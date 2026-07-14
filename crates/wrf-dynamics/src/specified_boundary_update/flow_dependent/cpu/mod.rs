mod kernel;
mod south_north_side;
mod west_east_side;

use wrf_compute::{CpuBackend, CpuField};

use self::kernel::SpecifiedBoundaryFlowCpuKernel;
use crate::{
    SpecifiedBoundaryFlowInputs, SpecifiedBoundaryFlowKernels, SpecifiedBoundaryFlowParameters,
    SpecifiedBoundaryFlowRegion, SpecifiedBoundaryFlowResult, SpecifiedBoundaryWestEastPeriodicity,
};

impl SpecifiedBoundaryFlowKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn apply_flow_dependent_specified_boundary(
        &self,
        scalar: &mut Self::Field,
        inputs: SpecifiedBoundaryFlowInputs<'_, Self::Field>,
        parameters: SpecifiedBoundaryFlowParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryFlowRegion,
    ) -> SpecifiedBoundaryFlowResult<()> {
        SpecifiedBoundaryFlowCpuKernel::try_new(
            scalar,
            inputs,
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
        SpecifiedBoundaryFlowError, SpecifiedBoundaryFlowField, SpecifiedBoundaryUpdateAxis,
    };

    #[derive(Clone)]
    struct OracleCase {
        name: &'static str,
        periodicity: SpecifiedBoundaryWestEastPeriodicity,
        tile_west_east: Range<usize>,
        tile_south_north: Range<usize>,
        tile_bottom_top: Range<usize>,
        specified_zone_width: usize,
        exceptional_velocities: bool,
    }

    struct Fixture {
        scalar: CpuField<f32>,
        west_east_velocity: CpuField<f32>,
        south_north_velocity: CpuField<f32>,
    }

    #[test]
    fn matches_the_direct_pinned_fortran_oracle_for_flow_tiles_and_ieee_signs() {
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
    fn each_shape_role_fails_before_scalar_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = &oracle_cases()[0];
        for field_role in [
            SpecifiedBoundaryFlowField::Scalar,
            SpecifiedBoundaryFlowField::WestEastVelocity,
            SpecifiedBoundaryFlowField::SouthNorthVelocity,
        ] {
            let mut fixture = initialized_fixture(&backend, shape());
            let wrong_shape = GridShape::try_new(7, 8, 8).unwrap();
            let wrong = backend.create_field(wrong_shape, 1.0).unwrap();
            let original = fixture.scalar.clone();
            let result = match field_role {
                SpecifiedBoundaryFlowField::Scalar => backend
                    .apply_flow_dependent_specified_boundary(
                        &mut wrong.clone(),
                        SpecifiedBoundaryFlowInputs::new(
                            &fixture.west_east_velocity,
                            &fixture.south_north_velocity,
                        ),
                        SpecifiedBoundaryFlowParameters::new(2),
                        case.periodicity,
                        &region(case, shape()),
                    ),
                SpecifiedBoundaryFlowField::WestEastVelocity => backend
                    .apply_flow_dependent_specified_boundary(
                        &mut fixture.scalar,
                        SpecifiedBoundaryFlowInputs::new(&wrong, &fixture.south_north_velocity),
                        SpecifiedBoundaryFlowParameters::new(2),
                        case.periodicity,
                        &region(case, shape()),
                    ),
                SpecifiedBoundaryFlowField::SouthNorthVelocity => backend
                    .apply_flow_dependent_specified_boundary(
                        &mut fixture.scalar,
                        SpecifiedBoundaryFlowInputs::new(&fixture.west_east_velocity, &wrong),
                        SpecifiedBoundaryFlowParameters::new(2),
                        case.periodicity,
                        &region(case, shape()),
                    ),
            };
            assert!(matches!(
                result,
                Err(SpecifiedBoundaryFlowError::ShapeMismatch { field, .. })
                    if field == field_role
            ));
            if field_role != SpecifiedBoundaryFlowField::Scalar {
                assert_eq!(fixture.scalar, original);
            }
        }
    }

    #[test]
    fn missing_upper_velocity_neighbor_is_failure_atomic() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let compact_shape = GridShape::try_new(7, 7, 8).unwrap();
        let mut fixture = initialized_fixture(&backend, compact_shape);
        let original = fixture.scalar.clone();
        let region =
            SpecifiedBoundaryFlowRegion::try_new(compact_shape, 1..7, 1..7, 1..7, 1..7, 1..7, 1..7)
                .unwrap();

        let result = backend.apply_flow_dependent_specified_boundary(
            &mut fixture.scalar,
            SpecifiedBoundaryFlowInputs::new(
                &fixture.west_east_velocity,
                &fixture.south_north_velocity,
            ),
            SpecifiedBoundaryFlowParameters::new(2),
            SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
            &region,
        );

        assert!(matches!(
            result,
            Err(SpecifiedBoundaryFlowError::MissingVelocityNeighbor {
                axis: SpecifiedBoundaryUpdateAxis::SouthNorth,
                ..
            })
        ));
        assert_eq!(fixture.scalar, original);
    }

    #[test]
    fn missing_east_velocity_neighbor_is_failure_atomic() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let compact_shape = GridShape::try_new(7, 8, 8).unwrap();
        let mut fixture = initialized_fixture(&backend, compact_shape);
        let original = fixture.scalar.clone();
        let region =
            SpecifiedBoundaryFlowRegion::try_new(compact_shape, 1..7, 1..7, 1..7, 1..7, 4..5, 1..7)
                .unwrap();

        let result = backend.apply_flow_dependent_specified_boundary(
            &mut fixture.scalar,
            SpecifiedBoundaryFlowInputs::new(
                &fixture.west_east_velocity,
                &fixture.south_north_velocity,
            ),
            SpecifiedBoundaryFlowParameters::new(2),
            SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
            &region,
        );

        assert!(matches!(
            result,
            Err(SpecifiedBoundaryFlowError::MissingVelocityNeighbor {
                axis: SpecifiedBoundaryUpdateAxis::WestEast,
                ..
            })
        ));
        assert_eq!(fixture.scalar, original);
    }

    #[test]
    fn missing_interior_source_is_failure_atomic() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = &oracle_cases()[0];
        let mut fixture = initialized_fixture(&backend, shape());
        let original = fixture.scalar.clone();

        let result = backend.apply_flow_dependent_specified_boundary(
            &mut fixture.scalar,
            SpecifiedBoundaryFlowInputs::new(
                &fixture.west_east_velocity,
                &fixture.south_north_velocity,
            ),
            SpecifiedBoundaryFlowParameters::new(3),
            case.periodicity,
            &region(case, shape()),
        );

        assert!(matches!(
            result,
            Err(SpecifiedBoundaryFlowError::MissingInteriorSource {
                axis: SpecifiedBoundaryUpdateAxis::SouthNorth,
                ..
            })
        ));
        assert_eq!(fixture.scalar, original);
    }

    #[test]
    fn zero_width_zone_is_an_exact_no_op() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = &oracle_cases()[0];
        let mut fixture = initialized_fixture(&backend, shape());
        let original = fixture.scalar.clone();

        backend
            .apply_flow_dependent_specified_boundary(
                &mut fixture.scalar,
                SpecifiedBoundaryFlowInputs::new(
                    &fixture.west_east_velocity,
                    &fixture.south_north_velocity,
                ),
                SpecifiedBoundaryFlowParameters::new(0),
                case.periodicity,
                &region(case, shape()),
            )
            .unwrap();

        assert_eq!(fixture.scalar, original);
    }

    fn run_case(backend: &CpuBackend, case: &OracleCase) -> CpuField<f32> {
        let mut fixture = initialized_fixture(backend, shape());
        if case.exceptional_velocities {
            set_exceptional_velocities(&mut fixture);
        }
        backend
            .apply_flow_dependent_specified_boundary(
                &mut fixture.scalar,
                SpecifiedBoundaryFlowInputs::new(
                    &fixture.west_east_velocity,
                    &fixture.south_north_velocity,
                ),
                SpecifiedBoundaryFlowParameters::new(case.specified_zone_width),
                case.periodicity,
                &region(case, shape()),
            )
            .unwrap();
        fixture.scalar
    }

    fn initialized_fixture(backend: &CpuBackend, shape: GridShape) -> Fixture {
        let mut scalar = backend.create_field(shape, 0.0_f32).unwrap();
        let mut west_east_velocity = backend.create_field(shape, 0.0_f32).unwrap();
        let mut south_north_velocity = backend.create_field(shape, 0.0_f32).unwrap();
        for south_north in 0..shape.south_north_points() {
            for bottom_top in 0..shape.bottom_top_points() {
                for west_east in 0..shape.west_east_points() {
                    let index = volume_index(west_east, bottom_top, south_north, shape);
                    scalar.values_mut()[index] =
                        -40.0 + west_east as f32 * 0.7 + bottom_top as f32 * 0.11
                            - south_north as f32 * 0.3;
                    west_east_velocity.values_mut()[index] =
                        if (west_east + south_north + bottom_top) % 2 == 0 {
                            -1.0
                        } else {
                            1.0
                        };
                    south_north_velocity.values_mut()[index] =
                        if (west_east + 2 * south_north + bottom_top) % 3 == 0 {
                            -1.0
                        } else {
                            1.0
                        };
                }
            }
        }
        Fixture {
            scalar,
            west_east_velocity,
            south_north_velocity,
        }
    }

    fn set_exceptional_velocities(fixture: &mut Fixture) {
        let shape = fixture.scalar.shape();
        fixture.west_east_velocity.values_mut()[volume_index(1, 1, 3, shape)] = -0.0;
        fixture.west_east_velocity.values_mut()[volume_index(2, 1, 3, shape)] = f32::NAN;
        fixture.west_east_velocity.values_mut()[volume_index(6, 1, 3, shape)] = f32::INFINITY;
        fixture.west_east_velocity.values_mut()[volume_index(7, 1, 3, shape)] = f32::NEG_INFINITY;
        fixture.south_north_velocity.values_mut()[volume_index(1, 1, 1, shape)] = -0.0;
        fixture.south_north_velocity.values_mut()[volume_index(2, 1, 1, shape)] = f32::NEG_INFINITY;
        fixture.south_north_velocity.values_mut()[volume_index(3, 1, 1, shape)] = f32::NAN;
        fixture.south_north_velocity.values_mut()[volume_index(1, 1, 7, shape)] = f32::INFINITY;
    }

    fn region(case: &OracleCase, shape: GridShape) -> SpecifiedBoundaryFlowRegion {
        SpecifiedBoundaryFlowRegion::try_new(
            shape,
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

    fn volume_index(
        west_east: usize,
        bottom_top: usize,
        south_north: usize,
        shape: GridShape,
    ) -> usize {
        west_east
            + shape.west_east_points() * (bottom_top + shape.bottom_top_points() * south_north)
    }

    fn assert_field_bits_eq(field: &CpuField<f32>, expected: &[u32], case_name: &str) {
        let actual: Vec<_> = field.values().iter().map(|value| value.to_bits()).collect();
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
        include_str!("../../../../test-data/flow_dependent_boundary.out.correct")
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

    fn oracle_cases() -> [OracleCase; 6] {
        [
            full_case("mixed_full"),
            OracleCase {
                name: "periodic_mixed",
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Periodic,
                ..full_case("periodic_mixed")
            },
            OracleCase {
                name: "partial_south_west",
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                tile_west_east: 1..5,
                tile_south_north: 1..5,
                tile_bottom_top: 2..5,
                specified_zone_width: 2,
                exceptional_velocities: false,
            },
            OracleCase {
                name: "partial_north_east",
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                tile_west_east: 4..8,
                tile_south_north: 4..8,
                tile_bottom_top: 2..5,
                specified_zone_width: 2,
                exceptional_velocities: false,
            },
            OracleCase {
                name: "interior",
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                tile_west_east: 4..5,
                tile_south_north: 4..5,
                tile_bottom_top: 2..5,
                specified_zone_width: 1,
                exceptional_velocities: false,
            },
            OracleCase {
                name: "exceptional_signs",
                exceptional_velocities: true,
                ..full_case("exceptional_signs")
            },
        ]
    }

    fn full_case(name: &'static str) -> OracleCase {
        OracleCase {
            name,
            periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
            tile_west_east: 1..8,
            tile_south_north: 1..8,
            tile_bottom_top: 1..7,
            specified_zone_width: 2,
            exceptional_velocities: false,
        }
    }
}
