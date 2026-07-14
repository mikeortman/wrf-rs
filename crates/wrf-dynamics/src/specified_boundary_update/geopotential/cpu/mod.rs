mod kernel;

use wrf_compute::{CpuBackend, CpuField};

use self::kernel::SpecifiedBoundaryGeopotentialCpuKernel;
use crate::{
    SpecifiedBoundaryGeopotentialInputs, SpecifiedBoundaryGeopotentialKernels,
    SpecifiedBoundaryGeopotentialResult, SpecifiedBoundaryUpdateParameters,
    SpecifiedBoundaryUpdateRegion, SpecifiedBoundaryWestEastPeriodicity,
};

impl SpecifiedBoundaryGeopotentialKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn update_specified_boundary_geopotential(
        &self,
        geopotential: &mut Self::Field,
        inputs: SpecifiedBoundaryGeopotentialInputs<'_, Self::Field>,
        parameters: SpecifiedBoundaryUpdateParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryGeopotentialResult<()> {
        SpecifiedBoundaryGeopotentialCpuKernel::try_new(
            geopotential,
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

    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;
    use crate::{
        SpecifiedBoundaryFieldLocation, SpecifiedBoundaryGeopotentialError,
        SpecifiedBoundaryGeopotentialInputs,
    };

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

    struct Fixture {
        geopotential: CpuField<f32>,
        saved_geopotential: CpuField<f32>,
        geopotential_tendency: CpuField<f32>,
        column_mass_tendency: CpuField<f32>,
        current_column_mass: CpuField<f32>,
        column_mass_multiplier: Vec<f32>,
        column_mass_offset: Vec<f32>,
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
            assert_bits_or_nan_class(actual.values(), &expected_case, case.name);
        }
    }

    #[test]
    fn complete_output_is_bitwise_deterministic_across_worker_counts() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();

        for case in oracle_cases() {
            assert_eq!(
                field_bits(&run_case(&one_worker, &case)),
                field_bits(&run_case(&four_workers, &case)),
                "case {}",
                case.name
            );
        }
    }

    #[test]
    fn shape_failure_is_atomic() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = &oracle_cases()[0];
        let mut fixture = Fixture::new(&backend, false);
        let original = fixture.geopotential.clone();
        let wrong_shape = GridShape::try_new(5, 6, 6).unwrap();
        let wrong_saved = backend.create_field(wrong_shape, 1.0_f32).unwrap();
        let inputs = SpecifiedBoundaryGeopotentialInputs::new(
            &wrong_saved,
            &fixture.geopotential_tendency,
            &fixture.column_mass_tendency,
            &fixture.current_column_mass,
            &fixture.column_mass_multiplier,
            &fixture.column_mass_offset,
        );

        let result = backend.update_specified_boundary_geopotential(
            &mut fixture.geopotential,
            inputs,
            SpecifiedBoundaryUpdateParameters::new(0.25, case.specified_zone_width),
            case.periodicity,
            &region(case),
        );

        assert!(matches!(
            result,
            Err(SpecifiedBoundaryGeopotentialError::ShapeMismatch {
                field: "saved geopotential",
                ..
            })
        ));
        assert_eq!(fixture.geopotential, original);
    }

    #[test]
    fn coefficient_failure_is_atomic() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = &oracle_cases()[0];
        let mut fixture = Fixture::new(&backend, false);
        fixture.column_mass_offset.pop();
        let original = fixture.geopotential.clone();

        let result = apply(&backend, &mut fixture, case);

        assert!(matches!(
            result,
            Err(
                SpecifiedBoundaryGeopotentialError::CoefficientLengthMismatch {
                    coefficient: "column mass offset",
                    ..
                }
            )
        ));
        assert_eq!(fixture.geopotential, original);
    }

    #[test]
    fn zero_width_zone_is_an_exact_no_op_after_validation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = &oracle_cases()[0];
        let mut fixture = Fixture::new(&backend, false);
        let original = fixture.geopotential.clone();
        let inputs = SpecifiedBoundaryGeopotentialInputs::new(
            &fixture.saved_geopotential,
            &fixture.geopotential_tendency,
            &fixture.column_mass_tendency,
            &fixture.current_column_mass,
            &fixture.column_mass_multiplier,
            &fixture.column_mass_offset,
        );

        backend
            .update_specified_boundary_geopotential(
                &mut fixture.geopotential,
                inputs,
                SpecifiedBoundaryUpdateParameters::new(f32::NAN, 0),
                case.periodicity,
                &region(case),
            )
            .unwrap();

        assert_eq!(fixture.geopotential, original);
    }

    impl Fixture {
        fn new(backend: &CpuBackend, exceptional: bool) -> Self {
            let shape = shape();
            let horizontal_shape = shape.horizontal_shape();
            let mut fixture = Self {
                geopotential: initialized_volume(backend, field_value),
                saved_geopotential: initialized_volume(backend, saved_value),
                geopotential_tendency: initialized_volume(backend, tendency_value),
                column_mass_tendency: initialized_horizontal(
                    backend,
                    horizontal_shape,
                    column_mass_tendency_value,
                ),
                current_column_mass: initialized_horizontal(
                    backend,
                    horizontal_shape,
                    current_column_mass_value,
                ),
                column_mass_multiplier: (0..shape.bottom_top_points())
                    .map(|bottom_top| 0.4 + bottom_top as f32 * 0.05)
                    .collect(),
                column_mass_offset: (0..shape.bottom_top_points())
                    .map(|bottom_top| 2.0 + bottom_top as f32 * 0.1)
                    .collect(),
            };
            if exceptional {
                fixture.column_mass_multiplier[2] = 0.0;
                fixture.column_mass_offset[2] = 0.0;
                fixture.current_column_mass.values_mut()[horizontal_index(1, 1)] = 0.0;
                fixture.column_mass_tendency.values_mut()[horizontal_index(1, 1)] = 0.0;
                fixture.column_mass_multiplier[3] = 1.0;
                fixture.column_mass_offset[3] = 0.0;
                fixture.geopotential_tendency.values_mut()[volume_index(1, 3, 1)] = 0.0;
            }
            fixture
        }
    }

    fn run_case(backend: &CpuBackend, case: &OracleCase) -> CpuField<f32> {
        let mut fixture = Fixture::new(backend, case.exceptional);
        apply(backend, &mut fixture, case).unwrap();
        fixture.geopotential
    }

    fn apply(
        backend: &CpuBackend,
        fixture: &mut Fixture,
        case: &OracleCase,
    ) -> SpecifiedBoundaryGeopotentialResult<()> {
        let inputs = SpecifiedBoundaryGeopotentialInputs::new(
            &fixture.saved_geopotential,
            &fixture.geopotential_tendency,
            &fixture.column_mass_tendency,
            &fixture.current_column_mass,
            &fixture.column_mass_multiplier,
            &fixture.column_mass_offset,
        );
        backend.update_specified_boundary_geopotential(
            &mut fixture.geopotential,
            inputs,
            SpecifiedBoundaryUpdateParameters::new(0.25, case.specified_zone_width),
            case.periodicity,
            &region(case),
        )
    }

    fn initialized_volume(
        backend: &CpuBackend,
        value: fn(usize, usize, usize) -> f32,
    ) -> CpuField<f32> {
        let shape = shape();
        let mut field = backend.create_field(shape, 0.0_f32).unwrap();
        for south_north in 0..shape.south_north_points() {
            for bottom_top in 0..shape.bottom_top_points() {
                for west_east in 0..shape.west_east_points() {
                    field.values_mut()[volume_index(west_east, bottom_top, south_north)] =
                        value(west_east, bottom_top, south_north);
                }
            }
        }
        field
    }

    fn initialized_horizontal(
        backend: &CpuBackend,
        shape: GridShape,
        value: fn(usize, usize) -> f32,
    ) -> CpuField<f32> {
        let mut field = backend.create_field(shape, 0.0_f32).unwrap();
        for south_north in 0..shape.south_north_points() {
            for west_east in 0..shape.west_east_points() {
                field.values_mut()[horizontal_index(west_east, south_north)] =
                    value(west_east, south_north);
            }
        }
        field
    }

    fn field_value(west_east: usize, bottom_top: usize, south_north: usize) -> f32 {
        -200.0 + west_east as f32 * 0.7 + bottom_top as f32 * 0.11 - south_north as f32 * 0.3
    }

    fn tendency_value(west_east: usize, bottom_top: usize, south_north: usize) -> f32 {
        1.5 + west_east as f32 * 0.02 - bottom_top as f32 * 0.03 + south_north as f32 * 0.04
    }

    fn saved_value(west_east: usize, bottom_top: usize, south_north: usize) -> f32 {
        100.0 + west_east as f32 * 0.5 + bottom_top as f32 * 0.13 - south_north as f32 * 0.2
    }

    fn current_column_mass_value(west_east: usize, south_north: usize) -> f32 {
        10.0 + west_east as f32 * 0.4 - south_north as f32 * 0.1
    }

    fn column_mass_tendency_value(west_east: usize, south_north: usize) -> f32 {
        0.3 + west_east as f32 * 0.02 + south_north as f32 * 0.01
    }

    fn shape() -> GridShape {
        GridShape::try_new(6, 6, 6).unwrap()
    }

    fn volume_index(west_east: usize, bottom_top: usize, south_north: usize) -> usize {
        west_east + 6 * (bottom_top + 6 * south_north)
    }

    fn horizontal_index(west_east: usize, south_north: usize) -> usize {
        west_east + 6 * south_north
    }

    fn field_bits(field: &CpuField<f32>) -> Vec<u32> {
        field.values().iter().map(|value| value.to_bits()).collect()
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

    fn parse_oracle() -> Vec<(&'static str, u32)> {
        include_str!("../../../../test-data/specified_boundary_geopotential.out.correct")
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

    fn assert_bits_or_nan_class(actual: &[f32], expected: &[u32], case_name: &str) {
        assert_eq!(actual.len(), expected.len());
        for (index, (actual_value, expected_bits)) in actual.iter().zip(expected.iter()).enumerate()
        {
            if f32::from_bits(*expected_bits).is_nan() {
                assert!(actual_value.is_nan(), "case {case_name}, index {index}");
            } else {
                assert_eq!(
                    actual_value.to_bits(),
                    *expected_bits,
                    "case {case_name}, index {index}"
                );
            }
        }
    }

    fn oracle_cases() -> [OracleCase; 9] {
        [
            full_case("full_level_full", SpecifiedBoundaryFieldLocation::FullLevel),
            full_case(
                "mass_half_full",
                SpecifiedBoundaryFieldLocation::MassHalfLevel,
            ),
            full_case(
                "west_east_full",
                SpecifiedBoundaryFieldLocation::WestEastFace,
            ),
            full_case(
                "south_north_full",
                SpecifiedBoundaryFieldLocation::SouthNorthFace,
            ),
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
                name: "periodic_full",
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Periodic,
                ..full_case("periodic_full", SpecifiedBoundaryFieldLocation::FullLevel)
            },
            OracleCase {
                name: "partial_south_west",
                location: SpecifiedBoundaryFieldLocation::FullLevel,
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                half_level_domain: 1..5,
                tile_west_east: 1..4,
                tile_south_north: 1..4,
                tile_bottom_top: 2..5,
                specified_zone_width: 2,
                exceptional: false,
            },
            OracleCase {
                name: "interior",
                location: SpecifiedBoundaryFieldLocation::FullLevel,
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                half_level_domain: 1..5,
                tile_west_east: 3..4,
                tile_south_north: 3..4,
                tile_bottom_top: 2..5,
                specified_zone_width: 1,
                exceptional: false,
            },
            OracleCase {
                name: "exceptional_full",
                exceptional: true,
                ..full_case(
                    "exceptional_full",
                    SpecifiedBoundaryFieldLocation::FullLevel,
                )
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
