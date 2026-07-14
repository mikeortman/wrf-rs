mod boundary_side;
mod kernel;
mod update_context;

use wrf_compute::{CpuBackend, CpuField};

use super::{
    SpecifiedBoundaryFinalizationInputs, SpecifiedBoundaryFinalizationKernels,
    SpecifiedBoundaryFinalizationParameters, SpecifiedBoundaryFinalizationRegion,
    SpecifiedBoundaryFinalizationResult,
};
use crate::SpecifiedBoundaryWestEastPeriodicity;

use kernel::SpecifiedBoundaryFinalizationCpuKernel;

impl SpecifiedBoundaryFinalizationKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn finalize_specified_boundary(
        &self,
        field: &mut Self::Field,
        inputs: SpecifiedBoundaryFinalizationInputs<'_, Self::Field>,
        parameters: SpecifiedBoundaryFinalizationParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryFinalizationRegion,
    ) -> SpecifiedBoundaryFinalizationResult<()> {
        SpecifiedBoundaryFinalizationCpuKernel::try_new(
            field,
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
        SpecifiedBoundaryFinalizationBoundaryFields, SpecifiedBoundaryFinalizationError,
        SpecifiedBoundaryFinalizationFieldLocation,
    };

    #[derive(Clone)]
    struct OracleCase {
        name: &'static str,
        location: SpecifiedBoundaryFinalizationFieldLocation,
        periodicity: SpecifiedBoundaryWestEastPeriodicity,
        tile_west_east: Range<usize>,
        tile_south_north: Range<usize>,
        tile_bottom_top: Range<usize>,
        is_horizontal_mass: bool,
        has_exceptional_values: bool,
    }

    struct Fixture {
        field: CpuField<f32>,
        column_mass: CpuField<f32>,
        map_factor: CpuField<f32>,
        column_mass_multiplier: Vec<f32>,
        column_mass_offset: Vec<f32>,
        west_values: CpuField<f32>,
        east_values: CpuField<f32>,
        south_values: CpuField<f32>,
        north_values: CpuField<f32>,
        west_tendencies: CpuField<f32>,
        east_tendencies: CpuField<f32>,
        south_tendencies: CpuField<f32>,
        north_tendencies: CpuField<f32>,
    }

    #[derive(Clone, Copy)]
    enum InvalidBoundaryRole {
        WestValues,
        EastValues,
        SouthValues,
        NorthValues,
        WestTendencies,
        EastTendencies,
        SouthTendencies,
        NorthTendencies,
    }

    #[test]
    fn matches_the_direct_pinned_fortran_oracle_for_every_coupling_and_tile() {
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
    fn every_boundary_array_shape_is_validated_before_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = oracle_cases()[0].clone();
        for (role, expected_name) in [
            (InvalidBoundaryRole::WestValues, "west boundary values"),
            (InvalidBoundaryRole::EastValues, "east boundary values"),
            (InvalidBoundaryRole::SouthValues, "south boundary values"),
            (InvalidBoundaryRole::NorthValues, "north boundary values"),
            (
                InvalidBoundaryRole::WestTendencies,
                "west boundary tendencies",
            ),
            (
                InvalidBoundaryRole::EastTendencies,
                "east boundary tendencies",
            ),
            (
                InvalidBoundaryRole::SouthTendencies,
                "south boundary tendencies",
            ),
            (
                InvalidBoundaryRole::NorthTendencies,
                "north boundary tendencies",
            ),
        ] {
            let mut fixture = Fixture::new(&backend, &case);
            let original = fixture.field.clone();
            let wrong = backend
                .create_field(GridShape::try_new(7, 3, 7).unwrap(), 0.0)
                .unwrap();
            let mut values = SpecifiedBoundaryFinalizationBoundaryFields::new(
                &fixture.west_values,
                &fixture.east_values,
                &fixture.south_values,
                &fixture.north_values,
            );
            let mut tendencies = SpecifiedBoundaryFinalizationBoundaryFields::new(
                &fixture.west_tendencies,
                &fixture.east_tendencies,
                &fixture.south_tendencies,
                &fixture.north_tendencies,
            );
            match role {
                InvalidBoundaryRole::WestValues => values.west = &wrong,
                InvalidBoundaryRole::EastValues => values.east = &wrong,
                InvalidBoundaryRole::SouthValues => values.south = &wrong,
                InvalidBoundaryRole::NorthValues => values.north = &wrong,
                InvalidBoundaryRole::WestTendencies => tendencies.west = &wrong,
                InvalidBoundaryRole::EastTendencies => tendencies.east = &wrong,
                InvalidBoundaryRole::SouthTendencies => tendencies.south = &wrong,
                InvalidBoundaryRole::NorthTendencies => tendencies.north = &wrong,
            }
            let result = backend.finalize_specified_boundary(
                &mut fixture.field,
                SpecifiedBoundaryFinalizationInputs::new(
                    &fixture.column_mass,
                    &fixture.map_factor,
                    &fixture.column_mass_multiplier,
                    &fixture.column_mass_offset,
                    values,
                    tendencies,
                ),
                SpecifiedBoundaryFinalizationParameters::new(0.25, 3, 2),
                case.periodicity,
                &region(&case),
            );
            assert!(matches!(
                result,
                Err(SpecifiedBoundaryFinalizationError::BoundaryShapeMismatch {
                    field,
                    ..
                }) if field == expected_name
            ));
            assert_eq!(fixture.field, original);
        }
    }

    #[test]
    fn normalization_shapes_and_coefficients_fail_before_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = oracle_cases()[0].clone();
        let wrong_horizontal = backend
            .create_field(GridShape::try_new(7, 8, 1).unwrap(), 0.0)
            .unwrap();

        for field_name in ["column mass", "map factor"] {
            let mut fixture = Fixture::new(&backend, &case);
            let original = fixture.field.clone();
            let column_mass = if field_name == "column mass" {
                &wrong_horizontal
            } else {
                &fixture.column_mass
            };
            let map_factor = if field_name == "map factor" {
                &wrong_horizontal
            } else {
                &fixture.map_factor
            };
            let result = backend.finalize_specified_boundary(
                &mut fixture.field,
                SpecifiedBoundaryFinalizationInputs::new(
                    column_mass,
                    map_factor,
                    &fixture.column_mass_multiplier,
                    &fixture.column_mass_offset,
                    SpecifiedBoundaryFinalizationBoundaryFields::new(
                        &fixture.west_values,
                        &fixture.east_values,
                        &fixture.south_values,
                        &fixture.north_values,
                    ),
                    SpecifiedBoundaryFinalizationBoundaryFields::new(
                        &fixture.west_tendencies,
                        &fixture.east_tendencies,
                        &fixture.south_tendencies,
                        &fixture.north_tendencies,
                    ),
                ),
                SpecifiedBoundaryFinalizationParameters::new(0.25, 3, 2),
                case.periodicity,
                &region(&case),
            );
            assert!(matches!(
                result,
                Err(SpecifiedBoundaryFinalizationError::ShapeMismatch { field, .. })
                    if field == field_name
            ));
            assert_eq!(fixture.field, original);
        }

        for coefficient_name in ["column mass multiplier", "column mass offset"] {
            let mut fixture = Fixture::new(&backend, &case);
            let original = fixture.field.clone();
            let short = vec![1.0; 7];
            let multiplier = if coefficient_name == "column mass multiplier" {
                &short
            } else {
                &fixture.column_mass_multiplier
            };
            let offset = if coefficient_name == "column mass offset" {
                &short
            } else {
                &fixture.column_mass_offset
            };
            let result = backend.finalize_specified_boundary(
                &mut fixture.field,
                SpecifiedBoundaryFinalizationInputs::new(
                    &fixture.column_mass,
                    &fixture.map_factor,
                    multiplier,
                    offset,
                    SpecifiedBoundaryFinalizationBoundaryFields::new(
                        &fixture.west_values,
                        &fixture.east_values,
                        &fixture.south_values,
                        &fixture.north_values,
                    ),
                    SpecifiedBoundaryFinalizationBoundaryFields::new(
                        &fixture.west_tendencies,
                        &fixture.east_tendencies,
                        &fixture.south_tendencies,
                        &fixture.north_tendencies,
                    ),
                ),
                SpecifiedBoundaryFinalizationParameters::new(0.25, 3, 2),
                case.periodicity,
                &region(&case),
            );
            assert!(matches!(
                result,
                Err(SpecifiedBoundaryFinalizationError::CoefficientLengthMismatch {
                    coefficient,
                    ..
                }) if coefficient == coefficient_name
            ));
            assert_eq!(fixture.field, original);
        }
    }

    #[test]
    fn invalid_widths_and_zero_zone_have_explicit_atomic_behavior() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = oracle_cases()[0].clone();

        for parameters in [
            SpecifiedBoundaryFinalizationParameters::new(0.25, 0, 0),
            SpecifiedBoundaryFinalizationParameters::new(0.25, 1, 2),
        ] {
            let mut fixture = Fixture::new(&backend, &case);
            let original = fixture.field.clone();
            let result = apply_fixture(&backend, &mut fixture, parameters, &case);
            assert!(result.is_err());
            assert_eq!(fixture.field, original);
        }

        let mut fixture = Fixture::new(&backend, &case);
        let original = fixture.field.clone();
        apply_fixture(
            &backend,
            &mut fixture,
            SpecifiedBoundaryFinalizationParameters::new(f32::NAN, 3, 0),
            &case,
        )
        .unwrap();
        assert_eq!(fixture.field, original);
    }

    fn run_case(backend: &CpuBackend, case: &OracleCase) -> CpuField<f32> {
        let mut fixture = Fixture::new(backend, case);
        apply_fixture(
            backend,
            &mut fixture,
            SpecifiedBoundaryFinalizationParameters::new(0.25, 3, 2),
            case,
        )
        .unwrap();
        fixture.field
    }

    fn apply_fixture(
        backend: &CpuBackend,
        fixture: &mut Fixture,
        parameters: SpecifiedBoundaryFinalizationParameters,
        case: &OracleCase,
    ) -> SpecifiedBoundaryFinalizationResult<()> {
        backend.finalize_specified_boundary(
            &mut fixture.field,
            SpecifiedBoundaryFinalizationInputs::new(
                &fixture.column_mass,
                &fixture.map_factor,
                &fixture.column_mass_multiplier,
                &fixture.column_mass_offset,
                SpecifiedBoundaryFinalizationBoundaryFields::new(
                    &fixture.west_values,
                    &fixture.east_values,
                    &fixture.south_values,
                    &fixture.north_values,
                ),
                SpecifiedBoundaryFinalizationBoundaryFields::new(
                    &fixture.west_tendencies,
                    &fixture.east_tendencies,
                    &fixture.south_tendencies,
                    &fixture.north_tendencies,
                ),
            ),
            parameters,
            case.periodicity,
            &region(case),
        )
    }

    fn region(case: &OracleCase) -> SpecifiedBoundaryFinalizationRegion {
        let half_level_domain = if case.is_horizontal_mass { 0..1 } else { 1..7 };
        SpecifiedBoundaryFinalizationRegion::try_new(
            case.shape(),
            case.location,
            1..7,
            1..7,
            half_level_domain,
            case.tile_west_east.clone(),
            case.tile_south_north.clone(),
            case.tile_bottom_top.clone(),
        )
        .unwrap()
    }

    impl OracleCase {
        fn shape(&self) -> GridShape {
            if self.is_horizontal_mass {
                GridShape::try_new(8, 8, 1).unwrap()
            } else {
                GridShape::try_new(8, 8, 8).unwrap()
            }
        }
    }

    impl Fixture {
        fn new(backend: &CpuBackend, case: &OracleCase) -> Self {
            let shape = case.shape();
            let horizontal_shape = shape.horizontal_shape();
            let boundary_vertical_points = if case.is_horizontal_mass { 1 } else { 7 };
            let boundary_shape = GridShape::try_new(8, 3, boundary_vertical_points).unwrap();
            let boundary_functions: [fn(usize, usize, usize) -> f32; 8] = if case.is_horizontal_mass
            {
                [
                    horizontal_west_value,
                    horizontal_east_value,
                    horizontal_south_value,
                    horizontal_north_value,
                    horizontal_west_tendency,
                    horizontal_east_tendency,
                    horizontal_south_tendency,
                    horizontal_north_tendency,
                ]
            } else {
                [
                    west_value,
                    east_value,
                    south_value,
                    north_value,
                    west_tendency,
                    east_tendency,
                    south_tendency,
                    north_tendency,
                ]
            };
            let mut fixture = Self {
                field: initialized_field(backend, shape),
                column_mass: initialized_horizontal_field(backend, horizontal_shape, column_mass),
                map_factor: initialized_horizontal_field(backend, horizontal_shape, map_factor),
                column_mass_multiplier: (0..shape.bottom_top_points())
                    .map(column_mass_multiplier)
                    .collect(),
                column_mass_offset: (0..shape.bottom_top_points())
                    .map(column_mass_offset)
                    .collect(),
                west_values: initialized_boundary_field(
                    backend,
                    boundary_shape,
                    boundary_functions[0],
                ),
                east_values: initialized_boundary_field(
                    backend,
                    boundary_shape,
                    boundary_functions[1],
                ),
                south_values: initialized_boundary_field(
                    backend,
                    boundary_shape,
                    boundary_functions[2],
                ),
                north_values: initialized_boundary_field(
                    backend,
                    boundary_shape,
                    boundary_functions[3],
                ),
                west_tendencies: initialized_boundary_field(
                    backend,
                    boundary_shape,
                    boundary_functions[4],
                ),
                east_tendencies: initialized_boundary_field(
                    backend,
                    boundary_shape,
                    boundary_functions[5],
                ),
                south_tendencies: initialized_boundary_field(
                    backend,
                    boundary_shape,
                    boundary_functions[6],
                ),
                north_tendencies: initialized_boundary_field(
                    backend,
                    boundary_shape,
                    boundary_functions[7],
                ),
            };
            if case.has_exceptional_values {
                fixture.west_values.values_mut()[boundary_index(3, 2, 1, 8, 7)] = f32::INFINITY;
                fixture.east_values.values_mut()[boundary_index(4, 1, 0, 8, 7)] = f32::NEG_INFINITY;
                fixture.south_values.values_mut()[boundary_index(3, 3, 0, 8, 7)] = -0.0;
                fixture.south_tendencies.values_mut()[boundary_index(3, 3, 0, 8, 7)] = -0.0;
                fixture.map_factor.values_mut()[3 + 8 * 6] = -0.0;
            }
            fixture
        }
    }

    fn initialized_field(backend: &CpuBackend, shape: GridShape) -> CpuField<f32> {
        let mut field = backend.create_field(shape, 0.0).unwrap();
        for south_north in 0..shape.south_north_points() {
            for bottom_top in 0..shape.bottom_top_points() {
                for west_east in 0..shape.west_east_points() {
                    let index = west_east
                        + shape.west_east_points()
                            * (bottom_top + shape.bottom_top_points() * south_north);
                    field.values_mut()[index] =
                        -30.0 + west_east as f32 * 0.7 + bottom_top as f32 * 0.11
                            - south_north as f32 * 0.2;
                }
            }
        }
        field
    }

    fn initialized_horizontal_field(
        backend: &CpuBackend,
        shape: GridShape,
        value: fn(usize, usize) -> f32,
    ) -> CpuField<f32> {
        let mut field = backend.create_field(shape, 0.0).unwrap();
        for south_north in 0..shape.south_north_points() {
            for west_east in 0..shape.west_east_points() {
                field.values_mut()[west_east + shape.west_east_points() * south_north] =
                    value(west_east, south_north);
            }
        }
        field
    }

    fn initialized_boundary_field(
        backend: &CpuBackend,
        shape: GridShape,
        value: fn(usize, usize, usize) -> f32,
    ) -> CpuField<f32> {
        let mut field = backend.create_field(shape, 0.0).unwrap();
        for distance in 0..shape.south_north_points() {
            for vertical in 0..shape.bottom_top_points() {
                for line in 0..shape.west_east_points() {
                    let index = boundary_index(
                        line,
                        vertical,
                        distance,
                        shape.west_east_points(),
                        shape.bottom_top_points(),
                    );
                    field.values_mut()[index] = value(line, vertical, distance);
                }
            }
        }
        field
    }

    fn boundary_index(
        line: usize,
        vertical: usize,
        distance: usize,
        line_points: usize,
        vertical_points: usize,
    ) -> usize {
        line + line_points * (vertical + vertical_points * distance)
    }

    fn column_mass(west_east: usize, south_north: usize) -> f32 {
        5.0 + west_east as f32 * 0.03 + south_north as f32 * 0.02
    }

    fn map_factor(west_east: usize, south_north: usize) -> f32 {
        0.9 + west_east as f32 * 0.004 - south_north as f32 * 0.003
    }

    fn column_mass_multiplier(bottom_top: usize) -> f32 {
        0.4 + bottom_top as f32 * 0.01
    }

    fn column_mass_offset(bottom_top: usize) -> f32 {
        1.3 - bottom_top as f32 * 0.015
    }

    fn physical_vertical(vertical: usize) -> f32 {
        (vertical + 1) as f32
    }

    fn physical_distance(distance: usize) -> f32 {
        (distance + 1) as f32
    }

    fn west_value(line: usize, vertical: usize, distance: usize) -> f32 {
        10.0 + line as f32 * 0.5
            + physical_vertical(vertical) * 0.07
            + 0.009 * physical_distance(distance)
    }

    fn east_value(line: usize, vertical: usize, distance: usize) -> f32 {
        -8.0 + line as f32 * 0.4 - physical_vertical(vertical) * 0.05
            + 0.008 * physical_distance(distance)
    }

    fn south_value(line: usize, vertical: usize, distance: usize) -> f32 {
        4.0 - line as f32 * 0.3 + physical_vertical(vertical) * 0.06
            - 0.007 * physical_distance(distance)
    }

    fn north_value(line: usize, vertical: usize, distance: usize) -> f32 {
        -2.0 + line as f32 * 0.2
            + physical_vertical(vertical) * 0.04
            + 0.006 * physical_distance(distance)
    }

    fn west_tendency(line: usize, vertical: usize, distance: usize) -> f32 {
        0.03 - line as f32 * 0.002
            + physical_vertical(vertical) * 0.0004
            + 0.0009 * physical_distance(distance)
    }

    fn east_tendency(line: usize, vertical: usize, distance: usize) -> f32 {
        -0.02 + line as f32 * 0.001 - physical_vertical(vertical) * 0.0003
            + 0.0008 * physical_distance(distance)
    }

    fn south_tendency(line: usize, vertical: usize, distance: usize) -> f32 {
        0.01 + line as f32 * 0.0015 + physical_vertical(vertical) * 0.0002
            - 0.0007 * physical_distance(distance)
    }

    fn north_tendency(line: usize, vertical: usize, distance: usize) -> f32 {
        -0.015 - line as f32 * 0.001
            + physical_vertical(vertical) * 0.0005
            + 0.0006 * physical_distance(distance)
    }

    fn horizontal_west_value(line: usize, _vertical: usize, distance: usize) -> f32 {
        10.0 + line as f32 * 0.5 + 0.009 * physical_distance(distance)
    }

    fn horizontal_east_value(line: usize, _vertical: usize, distance: usize) -> f32 {
        -8.0 + line as f32 * 0.4 + 0.008 * physical_distance(distance)
    }

    fn horizontal_south_value(line: usize, _vertical: usize, distance: usize) -> f32 {
        4.0 - line as f32 * 0.3 - 0.007 * physical_distance(distance)
    }

    fn horizontal_north_value(line: usize, _vertical: usize, distance: usize) -> f32 {
        -2.0 + line as f32 * 0.2 + 0.006 * physical_distance(distance)
    }

    fn horizontal_west_tendency(line: usize, _vertical: usize, distance: usize) -> f32 {
        0.03 - line as f32 * 0.002 + 0.0009 * physical_distance(distance)
    }

    fn horizontal_east_tendency(line: usize, _vertical: usize, distance: usize) -> f32 {
        -0.02 + line as f32 * 0.001 + 0.0008 * physical_distance(distance)
    }

    fn horizontal_south_tendency(line: usize, _vertical: usize, distance: usize) -> f32 {
        0.01 + line as f32 * 0.0015 - 0.0007 * physical_distance(distance)
    }

    fn horizontal_north_tendency(line: usize, _vertical: usize, distance: usize) -> f32 {
        -0.015 - line as f32 * 0.001 + 0.0006 * physical_distance(distance)
    }

    fn parse_oracle() -> Vec<(&'static str, u32)> {
        include_str!("../../../../test-data/specified_boundary_finalization.out.correct")
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

    fn field_bits(field: &CpuField<f32>) -> Vec<u32> {
        field.values().iter().map(|value| value.to_bits()).collect()
    }

    fn assert_field_bits_eq(field: &CpuField<f32>, expected: &[u32], case_name: &str) {
        let actual = field_bits(field);
        assert_eq!(actual.len(), expected.len(), "case {case_name} length");
        let shape = field.shape();
        for (index, (actual_bits, expected_bits)) in actual.iter().zip(expected).enumerate() {
            let west_east = index % shape.west_east_points();
            let remaining = index / shape.west_east_points();
            let bottom_top = remaining % shape.bottom_top_points();
            let south_north = remaining / shape.bottom_top_points();
            assert_eq!(
                actual_bits, expected_bits,
                "case {case_name} at ({west_east}, {bottom_top}, {south_north})"
            );
        }
    }

    fn oracle_cases() -> [OracleCase; 11] {
        [
            volume_case(
                "scalar_full",
                SpecifiedBoundaryFinalizationFieldLocation::MassHalfLevel,
            ),
            OracleCase {
                name: "west_east_momentum",
                location: SpecifiedBoundaryFinalizationFieldLocation::WestEastMomentum,
                tile_west_east: 1..8,
                ..volume_case(
                    "west_east_momentum",
                    SpecifiedBoundaryFinalizationFieldLocation::WestEastMomentum,
                )
            },
            OracleCase {
                name: "south_north_momentum",
                location: SpecifiedBoundaryFinalizationFieldLocation::SouthNorthMomentum,
                tile_south_north: 1..8,
                ..volume_case(
                    "south_north_momentum",
                    SpecifiedBoundaryFinalizationFieldLocation::SouthNorthMomentum,
                )
            },
            OracleCase {
                tile_bottom_top: 1..8,
                ..volume_case(
                    "vertical_momentum",
                    SpecifiedBoundaryFinalizationFieldLocation::VerticalMomentum,
                )
            },
            OracleCase {
                tile_bottom_top: 1..8,
                ..volume_case(
                    "full_level",
                    SpecifiedBoundaryFinalizationFieldLocation::FullLevel,
                )
            },
            OracleCase {
                name: "horizontal_mass",
                location: SpecifiedBoundaryFinalizationFieldLocation::HorizontalMass,
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                tile_west_east: 1..7,
                tile_south_north: 1..7,
                tile_bottom_top: 0..1,
                is_horizontal_mass: true,
                has_exceptional_values: false,
            },
            OracleCase {
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Periodic,
                ..volume_case(
                    "scalar_periodic",
                    SpecifiedBoundaryFinalizationFieldLocation::MassHalfLevel,
                )
            },
            OracleCase {
                tile_west_east: 1..5,
                tile_south_north: 1..5,
                tile_bottom_top: 2..5,
                ..volume_case(
                    "scalar_partial_south_west",
                    SpecifiedBoundaryFinalizationFieldLocation::MassHalfLevel,
                )
            },
            OracleCase {
                tile_west_east: 3..7,
                tile_south_north: 3..7,
                tile_bottom_top: 2..5,
                ..volume_case(
                    "full_partial_north_east",
                    SpecifiedBoundaryFinalizationFieldLocation::FullLevel,
                )
            },
            OracleCase {
                tile_west_east: 3..5,
                tile_south_north: 3..5,
                tile_bottom_top: 2..5,
                ..volume_case(
                    "scalar_interior",
                    SpecifiedBoundaryFinalizationFieldLocation::MassHalfLevel,
                )
            },
            OracleCase {
                tile_bottom_top: 1..8,
                has_exceptional_values: true,
                ..volume_case(
                    "momentum_exceptional",
                    SpecifiedBoundaryFinalizationFieldLocation::VerticalMomentum,
                )
            },
        ]
    }

    fn volume_case(
        name: &'static str,
        location: SpecifiedBoundaryFinalizationFieldLocation,
    ) -> OracleCase {
        OracleCase {
            name,
            location,
            periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
            tile_west_east: 1..7,
            tile_south_north: 1..7,
            tile_bottom_top: 1..7,
            is_horizontal_mass: false,
            has_exceptional_values: false,
        }
    }
}
