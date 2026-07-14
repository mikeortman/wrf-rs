mod boundary_side;
mod kernel;
mod update_context;

use wrf_compute::{CpuBackend, CpuField};

use super::{
    SpecifiedBoundaryTendencies, SpecifiedBoundaryTendencyKernels,
    SpecifiedBoundaryTendencyParameters, SpecifiedBoundaryTendencyResult,
};
use crate::{SpecifiedBoundaryUpdateRegion, SpecifiedBoundaryWestEastPeriodicity};
use kernel::SpecifiedBoundaryTendencyCpuKernel;

impl SpecifiedBoundaryTendencyKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn assign_specified_boundary_tendencies(
        &self,
        tendency: &mut Self::Field,
        boundary_tendencies: SpecifiedBoundaryTendencies<'_, Self::Field>,
        parameters: SpecifiedBoundaryTendencyParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryTendencyResult<()> {
        SpecifiedBoundaryTendencyCpuKernel::try_new(
            tendency,
            boundary_tendencies,
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

    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;
    use crate::{SpecifiedBoundaryFieldLocation, SpecifiedBoundaryTendencyError};

    const BOUNDARY_WIDTH: usize = 3;

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
        exceptional: bool,
    }

    #[test]
    fn matches_the_direct_pinned_fortran_oracle_for_staggers_tiles_and_ieee_values() {
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
    fn every_boundary_shape_role_is_validated_before_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = oracle_cases()[0].clone();
        let region = region(&case);
        let valid_shape = boundary_shape(&case);
        let valid = [
            backend.create_field(valid_shape, 1.0).unwrap(),
            backend.create_field(valid_shape, 2.0).unwrap(),
            backend.create_field(valid_shape, 3.0).unwrap(),
            backend.create_field(valid_shape, 4.0).unwrap(),
        ];
        let wrong = backend
            .create_field(GridShape::try_new(5, BOUNDARY_WIDTH, 5).unwrap(), 9.0)
            .unwrap();
        let expected_roles = [
            "west boundary tendencies",
            "east boundary tendencies",
            "south boundary tendencies",
            "north boundary tendencies",
        ];

        for (wrong_index, expected_role) in expected_roles.into_iter().enumerate() {
            let mut tendency = initialized_tendency(&backend);
            let original = tendency.clone();
            let side = |index| {
                if index == wrong_index {
                    &wrong
                } else {
                    &valid[index]
                }
            };
            let result = backend.assign_specified_boundary_tendencies(
                &mut tendency,
                SpecifiedBoundaryTendencies::new(side(0), side(1), side(2), side(3)),
                SpecifiedBoundaryTendencyParameters::new(BOUNDARY_WIDTH, 2),
                case.periodicity,
                &region,
            );

            assert!(matches!(
                result,
                Err(SpecifiedBoundaryTendencyError::BoundaryShapeMismatch {
                    field,
                    ..
                }) if field == expected_role
            ));
            assert_eq!(tendency, original);
        }
    }

    #[test]
    fn output_shape_and_width_failures_are_atomic() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = oracle_cases()[0].clone();
        let boundaries = boundary_fields(&backend, &case);
        let boundary_tendencies = boundary_references(&boundaries);
        let region = region(&case);
        let wrong_shape = GridShape::try_new(5, 6, 6).unwrap();
        let mut wrong_output = backend.create_field(wrong_shape, 8.0).unwrap();
        let wrong_original = wrong_output.clone();

        let shape_result = backend.assign_specified_boundary_tendencies(
            &mut wrong_output,
            boundary_tendencies,
            SpecifiedBoundaryTendencyParameters::new(BOUNDARY_WIDTH, 2),
            case.periodicity,
            &region,
        );
        assert!(matches!(
            shape_result,
            Err(SpecifiedBoundaryTendencyError::ShapeMismatch { .. })
        ));
        assert_eq!(wrong_output, wrong_original);

        for (parameters, expected) in [
            (
                SpecifiedBoundaryTendencyParameters::new(0, 0),
                SpecifiedBoundaryTendencyError::InvalidBoundaryWidth,
            ),
            (
                SpecifiedBoundaryTendencyParameters::new(BOUNDARY_WIDTH, BOUNDARY_WIDTH + 1),
                SpecifiedBoundaryTendencyError::SpecifiedZoneExceedsBoundaryWidth {
                    boundary_width: BOUNDARY_WIDTH,
                    specified_zone_width: BOUNDARY_WIDTH + 1,
                },
            ),
        ] {
            let mut tendency = initialized_tendency(&backend);
            let original = tendency.clone();
            let result = backend.assign_specified_boundary_tendencies(
                &mut tendency,
                boundary_tendencies,
                parameters,
                case.periodicity,
                &region,
            );

            assert_eq!(result, Err(expected));
            assert_eq!(tendency, original);
        }
    }

    #[test]
    fn zero_width_zone_is_validated_then_remains_an_exact_no_op() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = oracle_cases()
            .into_iter()
            .find(|case| case.name == "zero_zone")
            .unwrap();
        let boundaries = boundary_fields(&backend, &case);
        let mut tendency = initialized_tendency(&backend);
        let original = tendency.clone();

        backend
            .assign_specified_boundary_tendencies(
                &mut tendency,
                boundary_references(&boundaries),
                SpecifiedBoundaryTendencyParameters::new(BOUNDARY_WIDTH, 0),
                case.periodicity,
                &region(&case),
            )
            .unwrap();

        assert_eq!(tendency, original);
    }

    fn run_case(backend: &CpuBackend, case: &OracleCase) -> CpuField<f32> {
        let mut tendency = initialized_tendency(backend);
        let boundaries = boundary_fields(backend, case);
        backend
            .assign_specified_boundary_tendencies(
                &mut tendency,
                boundary_references(&boundaries),
                SpecifiedBoundaryTendencyParameters::new(BOUNDARY_WIDTH, case.specified_zone_width),
                case.periodicity,
                &region(case),
            )
            .unwrap();
        tendency
    }

    fn initialized_tendency(backend: &CpuBackend) -> CpuField<f32> {
        initialized_field(backend, shape(), |west_east, bottom_top, south_north| {
            ((-7000.0 + west_east as f32 * 11.0) + bottom_top as f32 * 0.25)
                - south_north as f32 * 3.0
        })
    }

    fn boundary_fields(backend: &CpuBackend, case: &OracleCase) -> [CpuField<f32>; 4] {
        let shape = boundary_shape(case);
        let mut fields = [
            boundary_field(backend, shape, 1000.0),
            boundary_field(backend, shape, 2000.0),
            boundary_field(backend, shape, 3000.0),
            boundary_field(backend, shape, 4000.0),
        ];
        if case.exceptional {
            let index = boundary_index(2, 0, 0, shape);
            fields[0].values_mut()[index] = f32::NEG_INFINITY;
            fields[1].values_mut()[index] = f32::from_bits(1);
            fields[2].values_mut()[index] = -0.0;
            fields[3].values_mut()[index] = f32::INFINITY;
        }
        fields
    }

    fn boundary_field(backend: &CpuBackend, shape: GridShape, base: f32) -> CpuField<f32> {
        initialized_field(backend, shape, |line_point, vertical, distance| {
            ((base + line_point as f32 * 10.0) + (vertical + 1) as f32)
                + (distance + 1) as f32 * 0.01
        })
    }

    fn boundary_references(
        fields: &[CpuField<f32>; 4],
    ) -> SpecifiedBoundaryTendencies<'_, CpuField<f32>> {
        SpecifiedBoundaryTendencies::new(&fields[0], &fields[1], &fields[2], &fields[3])
    }

    fn initialized_field(
        backend: &CpuBackend,
        shape: GridShape,
        value: impl Fn(usize, usize, usize) -> f32,
    ) -> CpuField<f32> {
        let mut field = backend.create_field(shape, 0.0_f32).unwrap();
        for south_north in 0..shape.south_north_points() {
            for bottom_top in 0..shape.bottom_top_points() {
                for west_east in 0..shape.west_east_points() {
                    let index = west_east
                        + shape.west_east_points()
                            * (bottom_top + shape.bottom_top_points() * south_north);
                    field.values_mut()[index] = value(west_east, bottom_top, south_north);
                }
            }
        }
        field
    }

    fn boundary_index(
        line_point: usize,
        vertical: usize,
        distance: usize,
        shape: GridShape,
    ) -> usize {
        line_point + shape.west_east_points() * (vertical + shape.bottom_top_points() * distance)
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

    fn boundary_shape(case: &OracleCase) -> GridShape {
        let vertical_points = if case.location == SpecifiedBoundaryFieldLocation::HorizontalMass {
            1
        } else {
            case.half_level_domain.len() + 1
        };
        GridShape::try_new(6, BOUNDARY_WIDTH, vertical_points).unwrap()
    }

    fn shape() -> GridShape {
        GridShape::try_new(6, 6, 6).unwrap()
    }

    fn field_bits(field: &CpuField<f32>) -> Vec<u32> {
        field.values().iter().map(|value| value.to_bits()).collect()
    }

    fn parse_oracle() -> Vec<(&'static str, u32)> {
        include_str!("../../../../test-data/specified_boundary_tendencies.out.correct")
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

    fn oracle_cases() -> [OracleCase; 12] {
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
                exceptional: false,
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
                tile_west_east: 1..4,
                tile_south_north: 1..4,
                tile_bottom_top: 2..3,
                ..full_case(
                    "partial_south_west",
                    SpecifiedBoundaryFieldLocation::MassHalfLevel,
                )
            },
            OracleCase {
                name: "partial_north_east",
                tile_west_east: 3..6,
                tile_south_north: 3..6,
                tile_bottom_top: 2..3,
                ..full_case(
                    "partial_north_east",
                    SpecifiedBoundaryFieldLocation::MassHalfLevel,
                )
            },
            OracleCase {
                name: "inactive_interior",
                tile_west_east: 3..4,
                tile_south_north: 3..4,
                tile_bottom_top: 2..3,
                specified_zone_width: 1,
                ..full_case(
                    "inactive_interior",
                    SpecifiedBoundaryFieldLocation::MassHalfLevel,
                )
            },
            OracleCase {
                name: "full_level_partial_vertical",
                tile_bottom_top: 2..4,
                ..full_case(
                    "full_level_partial_vertical",
                    SpecifiedBoundaryFieldLocation::FullLevel,
                )
            },
            OracleCase {
                name: "exceptional",
                exceptional: true,
                ..full_case("exceptional", SpecifiedBoundaryFieldLocation::MassHalfLevel)
            },
            OracleCase {
                name: "zero_zone",
                specified_zone_width: 0,
                ..full_case("zero_zone", SpecifiedBoundaryFieldLocation::MassHalfLevel)
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
            exceptional: false,
        }
    }
}
