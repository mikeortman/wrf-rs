mod horizontal_boundary_side;
mod kernel;

use wrf_compute::{CpuBackend, CpuField};

use self::kernel::SpecifiedBoundaryZeroGradientCpuKernel;
use crate::{
    SpecifiedBoundaryUpdateRegion, SpecifiedBoundaryWestEastPeriodicity,
    SpecifiedBoundaryZeroGradientKernels, SpecifiedBoundaryZeroGradientParameters,
    SpecifiedBoundaryZeroGradientResult,
};

pub(crate) fn validate_zero_gradient_specified_boundary(
    field: &mut CpuField<f32>,
    parameters: SpecifiedBoundaryZeroGradientParameters,
    west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
    region: &SpecifiedBoundaryUpdateRegion,
) -> SpecifiedBoundaryZeroGradientResult<()> {
    let _kernel = SpecifiedBoundaryZeroGradientCpuKernel::try_new(
        field,
        parameters,
        west_east_periodicity,
        region,
    )?;
    Ok(())
}

impl SpecifiedBoundaryZeroGradientKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn apply_zero_gradient_specified_boundary(
        &self,
        field: &mut Self::Field,
        parameters: SpecifiedBoundaryZeroGradientParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryZeroGradientResult<()> {
        SpecifiedBoundaryZeroGradientCpuKernel::try_new(
            field,
            parameters,
            west_east_periodicity,
            region,
        )?
        .execute(self)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use wrf_compute::{ComputeBackend, FieldStorage, GridShape};

    use super::*;
    use crate::{
        SpecifiedBoundaryFieldLocation, SpecifiedBoundaryUpdateAxis,
        SpecifiedBoundaryZeroGradientError,
    };

    #[derive(Clone)]
    struct OracleCase {
        name: &'static str,
        location: SpecifiedBoundaryFieldLocation,
        periodicity: SpecifiedBoundaryWestEastPeriodicity,
        tile_west_east: Range<usize>,
        tile_south_north: Range<usize>,
        tile_bottom_top: Range<usize>,
        specified_zone_width: usize,
    }

    #[test]
    fn matches_the_direct_pinned_fortran_oracle_for_staggers_periodicity_and_tiles() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let expected = parse_oracle();
        for case in oracle_cases() {
            let actual = run_case(&backend, &case);
            let expected_case: Vec<_> = expected
                .iter()
                .filter_map(|(name, bits)| (*name == case.name).then_some(*bits))
                .collect();
            assert_field_bits_eq(&actual, &expected_case, case.name);
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
    fn missing_interior_source_is_failure_atomic() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = &oracle_cases()[0];
        let mut field = initialized_field(&backend, shape());
        let original = field.clone();

        let result = backend.apply_zero_gradient_specified_boundary(
            &mut field,
            SpecifiedBoundaryZeroGradientParameters::new(3),
            case.periodicity,
            &region(case),
        );

        assert!(matches!(
            result,
            Err(SpecifiedBoundaryZeroGradientError::MissingInteriorSource {
                axis: SpecifiedBoundaryUpdateAxis::SouthNorth,
                ..
            })
        ));
        assert_eq!(field, original);
    }

    #[test]
    fn shape_failure_is_atomic() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = &oracle_cases()[0];
        let wrong_shape = GridShape::try_new(7, 8, 8).unwrap();
        let mut field = initialized_field(&backend, wrong_shape);
        let original = field.clone();

        let result = backend.apply_zero_gradient_specified_boundary(
            &mut field,
            SpecifiedBoundaryZeroGradientParameters::new(2),
            case.periodicity,
            &region(case),
        );

        assert!(matches!(
            result,
            Err(SpecifiedBoundaryZeroGradientError::ShapeMismatch { .. })
        ));
        assert_eq!(field, original);
    }

    #[test]
    fn zero_width_zone_is_an_exact_no_op() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = &oracle_cases()[0];
        let mut field = initialized_field(&backend, shape());
        let original = field.clone();
        backend
            .apply_zero_gradient_specified_boundary(
                &mut field,
                SpecifiedBoundaryZeroGradientParameters::new(0),
                case.periodicity,
                &region(case),
            )
            .unwrap();
        assert_eq!(field, original);
    }

    fn run_case(backend: &CpuBackend, case: &OracleCase) -> CpuField<f32> {
        let mut field = initialized_field(backend, shape());
        backend
            .apply_zero_gradient_specified_boundary(
                &mut field,
                SpecifiedBoundaryZeroGradientParameters::new(case.specified_zone_width),
                case.periodicity,
                &region(case),
            )
            .unwrap();
        field
    }

    fn initialized_field(backend: &CpuBackend, shape: GridShape) -> CpuField<f32> {
        let mut field = backend.create_field(shape, 0.0_f32).unwrap();
        for south_north in 0..shape.south_north_points() {
            for bottom_top in 0..shape.bottom_top_points() {
                for west_east in 0..shape.west_east_points() {
                    let index = west_east
                        + shape.west_east_points()
                            * (bottom_top + shape.bottom_top_points() * south_north);
                    field.values_mut()[index] =
                        -50.0 + west_east as f32 * 0.7 + bottom_top as f32 * 0.11
                            - south_north as f32 * 0.3;
                }
            }
        }
        field
    }

    fn region(case: &OracleCase) -> SpecifiedBoundaryUpdateRegion {
        SpecifiedBoundaryUpdateRegion::try_new(
            shape(),
            case.location,
            1..7,
            1..7,
            1..7,
            case.tile_west_east.clone(),
            case.tile_south_north.clone(),
            case.tile_bottom_top.clone(),
        )
        .unwrap()
    }

    fn shape() -> GridShape {
        GridShape::try_new(8, 8, 8).unwrap()
    }

    fn field_bits(field: &CpuField<f32>) -> Vec<u32> {
        field.values().iter().map(|value| value.to_bits()).collect()
    }

    fn assert_field_bits_eq(field: &CpuField<f32>, expected: &[u32], case_name: &str) {
        let actual = field_bits(field);
        assert_eq!(actual.len(), expected.len(), "case {case_name}");
        for (index, (actual_bits, expected_bits)) in actual.iter().zip(expected.iter()).enumerate()
        {
            if actual_bits == expected_bits {
                continue;
            }
            let shape = field.shape();
            let plane_length = shape.west_east_points() * shape.bottom_top_points();
            let south_north = index / plane_length;
            let within_plane = index % plane_length;
            let bottom_top = within_plane / shape.west_east_points();
            let west_east = within_plane % shape.west_east_points();
            panic!(
                "case {case_name}: mismatch at ({west_east}, {bottom_top}, {south_north}): actual={actual_bits:08X}, expected={expected_bits:08X}",
            );
        }
    }

    fn parse_oracle() -> Vec<(&'static str, u32)> {
        include_str!("../../../../test-data/zero_gradient_boundary.out.correct")
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

    fn oracle_cases() -> [OracleCase; 7] {
        [
            full_case("vertical_full", SpecifiedBoundaryFieldLocation::FullLevel),
            full_case("mass_half", SpecifiedBoundaryFieldLocation::MassHalfLevel),
            full_case("west_east", SpecifiedBoundaryFieldLocation::WestEastFace),
            full_case(
                "south_north",
                SpecifiedBoundaryFieldLocation::SouthNorthFace,
            ),
            OracleCase {
                name: "periodic_vertical",
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Periodic,
                ..full_case(
                    "periodic_vertical",
                    SpecifiedBoundaryFieldLocation::FullLevel,
                )
            },
            OracleCase {
                name: "partial_south_west",
                location: SpecifiedBoundaryFieldLocation::FullLevel,
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                tile_west_east: 1..5,
                tile_south_north: 1..5,
                tile_bottom_top: 2..7,
                specified_zone_width: 2,
            },
            OracleCase {
                name: "interior",
                location: SpecifiedBoundaryFieldLocation::FullLevel,
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                tile_west_east: 4..5,
                tile_south_north: 4..5,
                tile_bottom_top: 2..7,
                specified_zone_width: 1,
            },
        ]
    }

    fn full_case(name: &'static str, location: SpecifiedBoundaryFieldLocation) -> OracleCase {
        OracleCase {
            name,
            location,
            periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
            tile_west_east: 1..8,
            tile_south_north: 1..8,
            tile_bottom_top: 1..8,
            specified_zone_width: 2,
        }
    }
}
