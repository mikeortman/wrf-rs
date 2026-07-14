use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, ParallelExecutionError};

use crate::{
    SpecifiedBoundaryUpdateError, SpecifiedBoundaryUpdateKernels,
    SpecifiedBoundaryUpdateParameters, SpecifiedBoundaryUpdateRegion,
    SpecifiedBoundaryUpdateResult, SpecifiedBoundaryWestEastPeriodicity,
};

mod validation;

use crate::specified_boundary_update::geometry::SpecifiedBoundaryRanges;

pub(crate) fn validate_specified_boundary_update(
    field: &CpuField<f32>,
    tendency: &CpuField<f32>,
    region: &SpecifiedBoundaryUpdateRegion,
) -> SpecifiedBoundaryUpdateResult<()> {
    validation::validate(field, tendency, region)
}

impl SpecifiedBoundaryUpdateKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn update_specified_boundary(
        &self,
        field: &mut Self::Field,
        tendency: &Self::Field,
        parameters: SpecifiedBoundaryUpdateParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryUpdateResult<()> {
        validation::validate(field, tendency, region)?;
        if parameters.specified_zone_width == 0 {
            return Ok(());
        }
        let shape = region.shape();
        let west_east_points = shape.west_east_points();
        let bottom_top_points = shape.bottom_top_points();
        let plane_length = west_east_points * bottom_top_points;
        let tendency_values = tendency.values();
        let ranges = region.active_ranges();
        let boundary_ranges = SpecifiedBoundaryRanges::new(
            ranges,
            parameters.specified_zone_width,
            west_east_periodicity,
        );
        self.try_for_each_output_block::<_, Infallible, _>(
            field.values_mut(),
            plane_length,
            |south_north, output_plane| {
                let row_ranges = boundary_ranges.ranges_for_row(south_north);
                let plane_start = south_north * plane_length;
                let tendency_plane = &tendency_values[plane_start..plane_start + plane_length];
                for west_east_range in [
                    row_ranges.south,
                    row_ranges.north,
                    row_ranges.west,
                    row_ranges.east,
                ]
                .into_iter()
                .flatten()
                {
                    update_range(
                        output_plane,
                        tendency_plane,
                        west_east_points,
                        boundary_ranges.bottom_top_range(),
                        west_east_range,
                        parameters.time_step,
                    );
                }
                Ok(())
            },
        )
        .map_err(|error| match error {
            ParallelExecutionError::WorkerPanicked => SpecifiedBoundaryUpdateError::WorkerPanicked,
            ParallelExecutionError::Kernel(infallible) => match infallible {},
            ParallelExecutionError::ZeroBlockLength
            | ParallelExecutionError::IncompleteOutputBlock { .. }
            | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
                unreachable!("validated field shape creates complete planes")
            }
        })
    }
}

fn update_range(
    output_plane: &mut [f32],
    tendency_plane: &[f32],
    west_east_points: usize,
    bottom_top_range: std::ops::Range<usize>,
    west_east_range: std::ops::Range<usize>,
    time_step: f32,
) {
    for bottom_top in bottom_top_range {
        let row_start = bottom_top * west_east_points;
        for west_east in west_east_range.clone() {
            let index = row_start + west_east;
            output_plane[index] += time_step * tendency_plane[index];
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};

    use super::*;
    use crate::SpecifiedBoundaryFieldLocation;

    #[derive(Clone)]
    struct OracleCase {
        name: &'static str,
        location: SpecifiedBoundaryFieldLocation,
        periodicity: SpecifiedBoundaryWestEastPeriodicity,
        half_level_domain: Range<usize>,
        tile_west_east: Range<usize>,
        tile_south_north: Range<usize>,
        tile_bottom_top: Range<usize>,
        specified_zone_width: usize,
    }

    #[test]
    fn matches_the_direct_pinned_fortran_oracle_for_all_field_locations_and_tiles() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let expected = parse_oracle();

        for case in oracle_cases() {
            let actual = run_case(&backend, &case);
            let expected_case: Vec<_> = expected
                .iter()
                .filter_map(|(name, bits)| (*name == case.name).then_some(*bits))
                .collect();

            assert_eq!(field_bits(&actual), expected_case, "case {}", case.name);
        }
    }

    #[test]
    fn complete_output_is_bitwise_deterministic_across_worker_counts() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();

        for case in oracle_cases() {
            assert_eq!(
                run_case(&one_worker, &case),
                run_case(&four_workers, &case),
                "case {}",
                case.name
            );
        }
    }

    #[test]
    fn shape_validation_is_failure_atomic() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut field = initialized_field(&backend, shape(), field_value);
        let original = field.clone();
        let wrong_shape = GridShape::try_new(5, 6, 6).unwrap();
        let tendency = initialized_field(&backend, wrong_shape, tendency_value);
        let region = region(&oracle_cases()[0]);

        let result = backend.update_specified_boundary(
            &mut field,
            &tendency,
            SpecifiedBoundaryUpdateParameters::new(0.25, 2),
            SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
            &region,
        );

        assert!(matches!(
            result,
            Err(SpecifiedBoundaryUpdateError::ShapeMismatch {
                field: "tendency",
                ..
            })
        ));
        assert_eq!(field, original);
    }

    #[test]
    fn zero_width_zone_is_an_exact_no_op() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = &oracle_cases()[0];
        let mut field = initialized_field(&backend, shape(), field_value);
        let tendency = initialized_field(&backend, shape(), tendency_value);
        let original = field.clone();

        backend
            .update_specified_boundary(
                &mut field,
                &tendency,
                SpecifiedBoundaryUpdateParameters::new(f32::NAN, 0),
                case.periodicity,
                &region(case),
            )
            .unwrap();

        assert_eq!(field, original);
    }

    fn run_case(backend: &CpuBackend, case: &OracleCase) -> CpuField<f32> {
        let mut field = initialized_field(backend, shape(), field_value);
        let tendency = initialized_field(backend, shape(), tendency_value);
        backend
            .update_specified_boundary(
                &mut field,
                &tendency,
                SpecifiedBoundaryUpdateParameters::new(0.25, case.specified_zone_width),
                case.periodicity,
                &region(case),
            )
            .unwrap();
        field
    }

    fn region(case: &OracleCase) -> SpecifiedBoundaryUpdateRegion {
        SpecifiedBoundaryUpdateRegion::try_new(
            shape(),
            case.location,
            1..5,
            1..5,
            case.half_level_domain.clone(),
            case.tile_west_east.clone(),
            case.tile_south_north.clone(),
            case.tile_bottom_top.clone(),
        )
        .unwrap()
    }

    fn initialized_field(
        backend: &CpuBackend,
        shape: GridShape,
        value: fn(usize, usize, usize) -> f32,
    ) -> CpuField<f32> {
        let mut field = backend.create_field(shape, 0.0_f32).unwrap();
        let west_east_points = shape.west_east_points();
        let bottom_top_points = shape.bottom_top_points();
        for south_north in 0..shape.south_north_points() {
            for bottom_top in 0..bottom_top_points {
                for west_east in 0..west_east_points {
                    let index = west_east
                        + west_east_points * (bottom_top + bottom_top_points * south_north);
                    field.values_mut()[index] = value(west_east, bottom_top, south_north);
                }
            }
        }
        field
    }

    fn shape() -> GridShape {
        GridShape::try_new(6, 6, 6).unwrap()
    }

    fn field_value(west_east: usize, bottom_top: usize, south_north: usize) -> f32 {
        -20.0 + west_east as f32 * 0.7 + bottom_top as f32 * 0.11 - south_north as f32 * 0.3
    }

    fn tendency_value(west_east: usize, bottom_top: usize, south_north: usize) -> f32 {
        0.5 + west_east as f32 * 0.02 - bottom_top as f32 * 0.03 + south_north as f32 * 0.04
    }

    fn field_bits(field: &CpuField<f32>) -> Vec<u32> {
        field.values().iter().map(|value| value.to_bits()).collect()
    }

    fn parse_oracle() -> Vec<(&'static str, u32)> {
        include_str!("../../../test-data/specified_boundary_update.out.correct")
            .lines()
            .map(|line| {
                let mut parts = line.split_whitespace();
                let name = parts.next().unwrap();
                let _coordinates = [parts.next(), parts.next(), parts.next()];
                let bits = u32::from_str_radix(parts.next().unwrap(), 16).unwrap();
                (name, bits)
            })
            .collect()
    }

    fn oracle_cases() -> [OracleCase; 9] {
        [
            full_case("mass_full", SpecifiedBoundaryFieldLocation::MassHalfLevel),
            full_case(
                "west_east_full",
                SpecifiedBoundaryFieldLocation::WestEastFace,
            ),
            full_case(
                "south_north_full",
                SpecifiedBoundaryFieldLocation::SouthNorthFace,
            ),
            full_case("full_level", SpecifiedBoundaryFieldLocation::FullLevel),
            OracleCase {
                name: "horizontal_mass",
                location: SpecifiedBoundaryFieldLocation::HorizontalMass,
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                half_level_domain: 1..2,
                tile_west_east: 1..6,
                tile_south_north: 1..6,
                tile_bottom_top: 1..2,
                specified_zone_width: 2,
            },
            OracleCase {
                name: "periodic_mass",
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Periodic,
                ..full_case(
                    "periodic_mass",
                    SpecifiedBoundaryFieldLocation::MassHalfLevel,
                )
            },
            OracleCase {
                name: "partial_south_west",
                location: SpecifiedBoundaryFieldLocation::MassHalfLevel,
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                half_level_domain: 1..5,
                tile_west_east: 1..4,
                tile_south_north: 1..4,
                tile_bottom_top: 2..5,
                specified_zone_width: 2,
            },
            OracleCase {
                name: "partial_vertical_ignored",
                location: SpecifiedBoundaryFieldLocation::MassHalfLevel,
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                half_level_domain: 1..5,
                tile_west_east: 1..6,
                tile_south_north: 1..6,
                tile_bottom_top: 2..3,
                specified_zone_width: 2,
            },
            OracleCase {
                name: "interior",
                location: SpecifiedBoundaryFieldLocation::MassHalfLevel,
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                half_level_domain: 1..5,
                tile_west_east: 3..4,
                tile_south_north: 3..4,
                tile_bottom_top: 2..5,
                specified_zone_width: 1,
            },
        ]
    }

    fn full_case(name: &'static str, location: SpecifiedBoundaryFieldLocation) -> OracleCase {
        OracleCase {
            name,
            location,
            periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
            half_level_domain: 1..5,
            tile_west_east: 1..6,
            tile_south_north: 1..6,
            tile_bottom_top: 1..6,
            specified_zone_width: 2,
        }
    }
}
