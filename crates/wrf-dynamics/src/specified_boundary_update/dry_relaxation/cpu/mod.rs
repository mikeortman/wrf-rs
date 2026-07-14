mod execution;
mod inputs;
mod mass_weighting;
mod validation;

pub(crate) use validation::validate_cpu_dry_boundary_relaxation;

use wrf_compute::{CpuBackend, CpuField};

use super::{
    DryBoundaryRelaxationBoundaryFields, DryBoundaryRelaxationKernels,
    DryBoundaryRelaxationMassCoefficients, DryBoundaryRelaxationRegion,
    DryBoundaryRelaxationResult, DryBoundaryRelaxationState, DryBoundaryRelaxationTendencies,
    DryBoundaryRelaxationWorkspace, DryBoundaryVerticalRelaxation,
};
use crate::{
    SpecifiedBoundaryRelaxationCoefficients, SpecifiedBoundaryRelaxationParameters,
    SpecifiedBoundaryWestEastPeriodicity,
};
use execution::DryBoundaryRelaxationCpuExecution;

impl DryBoundaryRelaxationKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn add_dry_boundary_relaxation_tendencies(
        &self,
        tendencies: DryBoundaryRelaxationTendencies<'_, Self::Field>,
        state: DryBoundaryRelaxationState<'_, Self::Field>,
        boundaries: DryBoundaryRelaxationBoundaryFields<'_, Self::Field>,
        vertical: DryBoundaryVerticalRelaxation<'_, Self::Field>,
        workspace: DryBoundaryRelaxationWorkspace<'_, Self::Field>,
        mass_coefficients: DryBoundaryRelaxationMassCoefficients<'_>,
        relaxation_coefficients: SpecifiedBoundaryRelaxationCoefficients<'_>,
        parameters: SpecifiedBoundaryRelaxationParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &DryBoundaryRelaxationRegion,
    ) -> DryBoundaryRelaxationResult<()> {
        DryBoundaryRelaxationCpuExecution::new(
            self,
            tendencies,
            state,
            boundaries,
            vertical,
            workspace,
            mass_coefficients,
            relaxation_coefficients,
            parameters,
            west_east_periodicity,
            region,
        )
        .run()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use wrf_compute::{ComputeBackend, FieldStorage, GridShape};

    use super::*;
    use crate::{
        DryBoundaryRelaxationBoundaryData, DryBoundaryRelaxationBoundaryFields,
        DryBoundaryRelaxationError, DryBoundaryRelaxationField,
        DryBoundaryRelaxationMassCoefficients, DryBoundaryRelaxationState,
        DryBoundaryRelaxationTendencies, DryBoundaryRelaxationWorkspace,
        DryBoundaryVerticalRelaxation, SpecifiedBoundaryRelaxationBoundaryValues,
        SpecifiedBoundaryTendencies,
    };

    const BOUNDARY_WIDTH: usize = 4;
    const VOLUME_FIELD_NAMES: [&str; 5] = ["u", "v", "ph", "t", "w"];

    #[derive(Clone)]
    struct OracleCase {
        name: &'static str,
        periodicity: SpecifiedBoundaryWestEastPeriodicity,
        is_nested: bool,
        west_east_tile: Range<usize>,
        south_north_tile: Range<usize>,
        specified_zone_width: usize,
        relaxation_zone_width: usize,
        exceptional: bool,
    }

    struct BoundarySet {
        fields: [CpuField<f32>; 8],
    }

    impl BoundarySet {
        fn data(&self) -> DryBoundaryRelaxationBoundaryData<'_, CpuField<f32>> {
            DryBoundaryRelaxationBoundaryData::new(
                SpecifiedBoundaryRelaxationBoundaryValues::new(
                    &self.fields[0],
                    &self.fields[1],
                    &self.fields[2],
                    &self.fields[3],
                ),
                SpecifiedBoundaryTendencies::new(
                    &self.fields[4],
                    &self.fields[5],
                    &self.fields[6],
                    &self.fields[7],
                ),
            )
        }
    }

    struct Fixture {
        west_east_momentum: CpuField<f32>,
        south_north_momentum: CpuField<f32>,
        perturbation_geopotential: CpuField<f32>,
        potential_temperature: CpuField<f32>,
        vertical_velocity: CpuField<f32>,
        perturbation_column_mass: CpuField<f32>,
        full_column_mass: CpuField<f32>,
        west_east_tendency: CpuField<f32>,
        south_north_tendency: CpuField<f32>,
        geopotential_tendency: CpuField<f32>,
        temperature_tendency: CpuField<f32>,
        vertical_tendency: CpuField<f32>,
        column_mass_tendency: CpuField<f32>,
        boundaries: [BoundarySet; 6],
        half_level_multiplier: Vec<f32>,
        half_level_offset: Vec<f32>,
        full_level_multiplier: Vec<f32>,
        full_level_offset: Vec<f32>,
        forcing: Vec<f32>,
        smoothing: Vec<f32>,
        workspace: CpuField<f32>,
        region: DryBoundaryRelaxationRegion,
        case: OracleCase,
    }

    #[test]
    fn matches_direct_pinned_fortran_for_dry_fields_tiles_nesting_and_ieee_values() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let expected = parse_oracle();

        for case in oracle_cases() {
            let mut fixture = create_fixture(&backend, case.clone());
            apply_fixture(&backend, &mut fixture).unwrap();
            for (field_name, field) in VOLUME_FIELD_NAMES.into_iter().zip(fixture.volume_outputs())
            {
                assert_output(&expected, case.name, field_name, field);
            }
            assert_output(&expected, case.name, "mu", &fixture.column_mass_tendency);
        }
    }

    #[test]
    fn complete_outputs_are_bitwise_deterministic_across_worker_counts() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();

        for case in oracle_cases() {
            let mut serial = create_fixture(&one_worker, case.clone());
            let mut parallel = create_fixture(&four_workers, case.clone());
            apply_fixture(&one_worker, &mut serial).unwrap();
            apply_fixture(&four_workers, &mut parallel).unwrap();
            assert_eq!(
                output_bits(&serial),
                output_bits(&parallel),
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn inactive_and_empty_tiles_skip_mass_weighting_work() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        for case in [oracle_cases()[5].clone(), oracle_cases()[6].clone()] {
            let mut fixture = create_fixture(&backend, case);
            let workspace_before = fixture.workspace.clone();

            apply_fixture(&backend, &mut fixture).unwrap();

            assert_eq!(fixture.workspace, workspace_before);
        }
    }

    #[test]
    fn late_boundary_failure_is_atomic_across_every_tendency() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut fixture = create_fixture(&backend, oracle_cases()[1].clone());
        fixture.boundaries[4].fields[7] = backend
            .create_field(GridShape::try_new(9, BOUNDARY_WIDTH, 5).unwrap(), 0.0)
            .unwrap();
        let before = output_bits(&fixture);

        let error = apply_fixture(&backend, &mut fixture).unwrap_err();

        assert!(matches!(
            error,
            DryBoundaryRelaxationError::SpecifiedRelaxation {
                target: crate::DryBoundaryRelaxationTarget::VerticalMomentum,
                ..
            }
        ));
        assert_eq!(output_bits(&fixture), before);
    }

    #[test]
    fn workspace_shape_and_mass_coefficient_failures_are_atomic() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = oracle_cases()[0].clone();

        let mut wrong_workspace = create_fixture(&backend, case.clone());
        wrong_workspace.workspace = backend
            .create_field(GridShape::try_new(2, 2, 2).unwrap(), 0.0)
            .unwrap();
        let before = output_bits(&wrong_workspace);
        let error = apply_fixture(&backend, &mut wrong_workspace).unwrap_err();
        assert!(matches!(
            error,
            DryBoundaryRelaxationError::FieldShapeMismatch {
                field: DryBoundaryRelaxationField::MassWeightingWorkspace,
                ..
            }
        ));
        assert_eq!(output_bits(&wrong_workspace), before);

        let mut short_coefficient = create_fixture(&backend, case);
        short_coefficient.full_level_offset.pop();
        let before = output_bits(&short_coefficient);
        assert!(matches!(
            apply_fixture(&backend, &mut short_coefficient),
            Err(DryBoundaryRelaxationError::CoefficientLengthMismatch {
                coefficient: "full-level offset",
                expected: 6,
                actual: 5,
            })
        ));
        assert_eq!(output_bits(&short_coefficient), before);
    }

    #[test]
    fn every_state_and_tendency_role_is_validated_before_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let roles = [
            DryBoundaryRelaxationField::WestEastMomentum,
            DryBoundaryRelaxationField::SouthNorthMomentum,
            DryBoundaryRelaxationField::PerturbationGeopotential,
            DryBoundaryRelaxationField::PotentialTemperature,
            DryBoundaryRelaxationField::PerturbationColumnMass,
            DryBoundaryRelaxationField::FullColumnMass,
            DryBoundaryRelaxationField::VerticalVelocity,
            DryBoundaryRelaxationField::WestEastMomentumTendency,
            DryBoundaryRelaxationField::SouthNorthMomentumTendency,
            DryBoundaryRelaxationField::PerturbationGeopotentialTendency,
            DryBoundaryRelaxationField::PotentialTemperatureTendency,
            DryBoundaryRelaxationField::PerturbationColumnMassTendency,
            DryBoundaryRelaxationField::VerticalMomentumTendency,
        ];

        for role in roles {
            let mut fixture = create_fixture(&backend, oracle_cases()[1].clone());
            replace_with_wrong_shape(&backend, &mut fixture, role);
            let before = output_bits(&fixture);

            let error = apply_fixture(&backend, &mut fixture).unwrap_err();

            assert!(matches!(
                error,
                DryBoundaryRelaxationError::FieldShapeMismatch { field, .. }
                    if field == role
            ));
            assert_eq!(output_bits(&fixture), before, "{role}");
        }
    }

    fn apply_fixture(
        backend: &CpuBackend,
        fixture: &mut Fixture,
    ) -> DryBoundaryRelaxationResult<()> {
        let boundaries = DryBoundaryRelaxationBoundaryFields::new(
            fixture.boundaries[0].data(),
            fixture.boundaries[1].data(),
            fixture.boundaries[2].data(),
            fixture.boundaries[3].data(),
            fixture.boundaries[5].data(),
        );
        let vertical = if fixture.case.is_nested {
            DryBoundaryVerticalRelaxation::Nested {
                velocity: &fixture.vertical_velocity,
                tendency: &mut fixture.vertical_tendency,
                boundary: fixture.boundaries[4].data(),
            }
        } else {
            DryBoundaryVerticalRelaxation::Disabled
        };
        backend.add_dry_boundary_relaxation_tendencies(
            DryBoundaryRelaxationTendencies::new(
                &mut fixture.west_east_tendency,
                &mut fixture.south_north_tendency,
                &mut fixture.geopotential_tendency,
                &mut fixture.temperature_tendency,
                &mut fixture.column_mass_tendency,
            ),
            DryBoundaryRelaxationState::new(
                &fixture.west_east_momentum,
                &fixture.south_north_momentum,
                &fixture.perturbation_geopotential,
                &fixture.potential_temperature,
                &fixture.perturbation_column_mass,
                &fixture.full_column_mass,
            ),
            boundaries,
            vertical,
            DryBoundaryRelaxationWorkspace::new(&mut fixture.workspace),
            DryBoundaryRelaxationMassCoefficients::new(
                &fixture.half_level_multiplier,
                &fixture.half_level_offset,
                &fixture.full_level_multiplier,
                &fixture.full_level_offset,
            ),
            SpecifiedBoundaryRelaxationCoefficients::new(&fixture.forcing, &fixture.smoothing),
            SpecifiedBoundaryRelaxationParameters::new(
                BOUNDARY_WIDTH,
                fixture.case.specified_zone_width,
                fixture.case.relaxation_zone_width,
                0.25,
            ),
            fixture.case.periodicity,
            &fixture.region,
        )
    }

    fn create_fixture(backend: &CpuBackend, case: OracleCase) -> Fixture {
        let volume_shape = GridShape::try_new(10, 10, 6).unwrap();
        let horizontal_shape = volume_shape.horizontal_shape();
        let region = DryBoundaryRelaxationRegion::try_new(
            volume_shape,
            1..9,
            1..9,
            1..5,
            case.west_east_tile.clone(),
            case.south_north_tile.clone(),
            1..6,
        )
        .unwrap();
        let volume =
            |function: fn(f32, f32, f32) -> f32| initialized_field(backend, volume_shape, function);
        let mut fixture = Fixture {
            west_east_momentum: volume(|i, k, j| ((10.0 + i * 0.5) + k * 0.25) - j * 0.125),
            south_north_momentum: volume(|i, k, j| ((20.0 - i * 0.25) + k * 0.5) + j * 0.0625),
            perturbation_geopotential: volume(|i, k, j| ((30.0 + i * 0.125) + k * 0.75) - j * 0.25),
            potential_temperature: volume(|i, k, j| ((40.0 - i * 0.0625) + k * 0.375) + j * 0.5),
            vertical_velocity: volume(|i, k, j| ((50.0 + i * 0.375) - k * 0.125) + j * 0.25),
            perturbation_column_mass: initialized_field(backend, horizontal_shape, |i, _, j| {
                60.0 + i * 0.25 - j * 0.125
            }),
            full_column_mass: initialized_field(backend, horizontal_shape, |i, _, j| {
                10.0 + i * 0.125 + j * 0.0625
            }),
            west_east_tendency: volume(|i, k, j| ((-10.0 + i * 0.25) + k * 0.0625) - j * 0.5),
            south_north_tendency: volume(|i, k, j| ((-20.0 - i * 0.125) + k * 0.25) + j * 0.375),
            geopotential_tendency: volume(|i, k, j| ((-30.0 + i * 0.5) - k * 0.125) + j * 0.0625),
            temperature_tendency: volume(|i, k, j| ((-40.0 - i * 0.25) + k * 0.5) - j * 0.125),
            vertical_tendency: volume(|i, k, j| ((-50.0 + i * 0.0625) + k * 0.375) + j * 0.25),
            column_mass_tendency: initialized_field(backend, horizontal_shape, |i, _, j| {
                -60.0 + i * 0.5 + j * 0.25
            }),
            boundaries: [
                boundary_set(backend, 100.0, false),
                boundary_set(backend, 200.0, false),
                boundary_set(backend, 300.0, false),
                boundary_set(backend, 400.0, false),
                boundary_set(backend, 500.0, false),
                boundary_set(backend, 600.0, true),
            ],
            half_level_multiplier: (0..6).map(|k| 0.60 + k as f32 * 0.03125).collect(),
            half_level_offset: (0..6).map(|k| 0.40 - k as f32 * 0.015625).collect(),
            full_level_multiplier: (0..6).map(|k| 0.55 + k as f32 * 0.015625).collect(),
            full_level_offset: (0..6).map(|k| 0.45 - k as f32 * 0.0078125).collect(),
            forcing: vec![0.0, 0.7, 0.4, 0.0],
            smoothing: vec![0.0, 0.1, 0.05, 0.0],
            workspace: backend.create_field(region.workspace_shape(), 0.0).unwrap(),
            region,
            case,
        };
        if fixture.case.exceptional {
            set_boundary_bits(&mut fixture.boundaries[2].fields[2], 4, 0, 1, 0x7f80_0000);
            set_boundary_bits(&mut fixture.boundaries[2].fields[6], 4, 0, 1, 0xff80_0000);
            set_boundary_bits(&mut fixture.boundaries[3].fields[3], 4, 0, 1, 0x8000_0000);
            set_boundary_bits(&mut fixture.boundaries[4].fields[4], 4, 0, 1, 0x0000_0001);
            fixture.full_column_mass.values_mut()[14] = f32::from_bits(0x7f7f_ffff);
        }
        fixture
    }

    fn boundary_set(backend: &CpuBackend, base: f32, horizontal: bool) -> BoundarySet {
        let vertical_points = if horizontal { 1 } else { 5 };
        let shape = GridShape::try_new(10, BOUNDARY_WIDTH, vertical_points).unwrap();
        let fields = std::array::from_fn(|role| {
            initialized_boundary_field(backend, shape, role, base, horizontal)
        });
        BoundarySet { fields }
    }

    fn initialized_boundary_field(
        backend: &CpuBackend,
        shape: GridShape,
        role: usize,
        base: f32,
        horizontal: bool,
    ) -> CpuField<f32> {
        let mut field = backend.create_field(shape, 0.0).unwrap();
        for distance in 0..BOUNDARY_WIDTH {
            let source_distance = (distance + 1) as f32;
            for vertical in 0..shape.bottom_top_points() {
                let source_vertical = if horizontal {
                    0.0
                } else {
                    (vertical + 1) as f32
                };
                for line in 0..10 {
                    let line = line as f32;
                    let value = match role {
                        0 => {
                            base + 10.0
                                + line * 0.5
                                + source_vertical * 0.25
                                + source_distance * 0.03125
                        }
                        1 => {
                            base + 20.0
                                + line * 0.5
                                + source_vertical * 0.25
                                + source_distance * 0.03125
                        }
                        2 => {
                            base + 30.0
                                + line * 0.5
                                + source_vertical * 0.25
                                + source_distance * 0.03125
                        }
                        3 => {
                            base + 40.0
                                + line * 0.5
                                + source_vertical * 0.25
                                + source_distance * 0.03125
                        }
                        4 => {
                            -base * 0.01
                                + line * 0.125
                                + source_vertical * 0.0625
                                + source_distance * 0.015625
                        }
                        5 => {
                            base * 0.01
                                + line * 0.125
                                + source_vertical * 0.0625
                                + source_distance * 0.015625
                        }
                        6 => {
                            -base * 0.02
                                + line * 0.125
                                + source_vertical * 0.0625
                                + source_distance * 0.015625
                        }
                        7 => {
                            base * 0.02
                                + line * 0.125
                                + source_vertical * 0.0625
                                + source_distance * 0.015625
                        }
                        _ => unreachable!(),
                    };
                    let index = line as usize
                        + shape.west_east_points()
                            * (vertical + shape.bottom_top_points() * distance);
                    field.values_mut()[index] = value;
                }
            }
        }
        field
    }

    fn initialized_field(
        backend: &CpuBackend,
        shape: GridShape,
        value: impl Fn(f32, f32, f32) -> f32,
    ) -> CpuField<f32> {
        let mut field = backend.create_field(shape, 0.0).unwrap();
        for south_north in 0..shape.south_north_points() {
            for bottom_top in 0..shape.bottom_top_points() {
                for west_east in 0..shape.west_east_points() {
                    let index = west_east
                        + shape.west_east_points()
                            * (bottom_top + shape.bottom_top_points() * south_north);
                    field.values_mut()[index] =
                        value(west_east as f32, bottom_top as f32, south_north as f32);
                }
            }
        }
        field
    }

    fn set_boundary_bits(
        field: &mut CpuField<f32>,
        line: usize,
        vertical: usize,
        distance: usize,
        bits: u32,
    ) {
        let shape = field.shape();
        let index =
            line + shape.west_east_points() * (vertical + shape.bottom_top_points() * distance);
        field.values_mut()[index] = f32::from_bits(bits);
    }

    fn replace_with_wrong_shape(
        backend: &CpuBackend,
        fixture: &mut Fixture,
        role: DryBoundaryRelaxationField,
    ) {
        let wrong = || {
            backend
                .create_field(GridShape::try_new(2, 2, 2).unwrap(), 0.0)
                .unwrap()
        };
        match role {
            DryBoundaryRelaxationField::WestEastMomentum => fixture.west_east_momentum = wrong(),
            DryBoundaryRelaxationField::SouthNorthMomentum => {
                fixture.south_north_momentum = wrong()
            }
            DryBoundaryRelaxationField::PerturbationGeopotential => {
                fixture.perturbation_geopotential = wrong()
            }
            DryBoundaryRelaxationField::PotentialTemperature => {
                fixture.potential_temperature = wrong()
            }
            DryBoundaryRelaxationField::PerturbationColumnMass => {
                fixture.perturbation_column_mass = wrong()
            }
            DryBoundaryRelaxationField::FullColumnMass => fixture.full_column_mass = wrong(),
            DryBoundaryRelaxationField::VerticalVelocity => fixture.vertical_velocity = wrong(),
            DryBoundaryRelaxationField::WestEastMomentumTendency => {
                fixture.west_east_tendency = wrong()
            }
            DryBoundaryRelaxationField::SouthNorthMomentumTendency => {
                fixture.south_north_tendency = wrong()
            }
            DryBoundaryRelaxationField::PerturbationGeopotentialTendency => {
                fixture.geopotential_tendency = wrong()
            }
            DryBoundaryRelaxationField::PotentialTemperatureTendency => {
                fixture.temperature_tendency = wrong()
            }
            DryBoundaryRelaxationField::PerturbationColumnMassTendency => {
                fixture.column_mass_tendency = wrong()
            }
            DryBoundaryRelaxationField::VerticalMomentumTendency => {
                fixture.vertical_tendency = wrong()
            }
            DryBoundaryRelaxationField::MassWeightingWorkspace => fixture.workspace = wrong(),
        }
    }

    fn oracle_cases() -> [OracleCase; 8] {
        [
            oracle_case("full_global", false, false, 1..9, 1..9, 1, 3, false),
            oracle_case("full_nested", false, true, 1..9, 1..9, 1, 3, false),
            oracle_case("periodic_nested", true, true, 1..9, 1..9, 1, 3, false),
            oracle_case("south_west", false, true, 1..6, 1..6, 1, 3, false),
            oracle_case("north_east", false, true, 4..9, 4..9, 1, 3, false),
            oracle_case("inactive", false, true, 4..6, 4..6, 1, 3, false),
            oracle_case("empty_band", false, true, 1..9, 1..9, 2, 2, false),
            oracle_case("exceptional", false, true, 1..9, 1..9, 1, 3, true),
        ]
    }

    #[allow(clippy::too_many_arguments)]
    fn oracle_case(
        name: &'static str,
        periodic: bool,
        is_nested: bool,
        west_east_tile: Range<usize>,
        south_north_tile: Range<usize>,
        specified_zone_width: usize,
        relaxation_zone_width: usize,
        exceptional: bool,
    ) -> OracleCase {
        OracleCase {
            name,
            periodicity: if periodic {
                SpecifiedBoundaryWestEastPeriodicity::Periodic
            } else {
                SpecifiedBoundaryWestEastPeriodicity::Nonperiodic
            },
            is_nested,
            west_east_tile,
            south_north_tile,
            specified_zone_width,
            relaxation_zone_width,
            exceptional,
        }
    }

    fn parse_oracle() -> Vec<(&'static str, &'static str, u32)> {
        include_str!("../../../../test-data/dry_boundary_relaxation.out.correct")
            .lines()
            .map(|line| {
                let mut parts = line.split_whitespace();
                let case = intern_case(parts.next().unwrap());
                let field = intern_field(parts.next().unwrap());
                let bits = u32::from_str_radix(parts.next().unwrap(), 16).unwrap();
                assert!(parts.next().is_none());
                (case, field, bits)
            })
            .collect()
    }

    fn intern_case(name: &str) -> &'static str {
        oracle_cases()
            .into_iter()
            .find(|case| case.name == name)
            .map(|case| case.name)
            .unwrap_or_else(|| panic!("unexpected oracle case {name}"))
    }

    fn intern_field(name: &str) -> &'static str {
        match name {
            "u" => "u",
            "v" => "v",
            "ph" => "ph",
            "t" => "t",
            "w" => "w",
            "mu" => "mu",
            _ => panic!("unexpected oracle field {name}"),
        }
    }

    fn assert_output(
        expected: &[(&str, &str, u32)],
        case: &str,
        field_name: &str,
        field: &CpuField<f32>,
    ) {
        let expected_bits: Vec<_> = expected
            .iter()
            .filter_map(|(expected_case, expected_field, bits)| {
                (*expected_case == case && *expected_field == field_name).then_some(*bits)
            })
            .collect();
        assert_eq!(
            field.values().len(),
            expected_bits.len(),
            "{case} {field_name}"
        );
        for (index, (actual, expected)) in field.values().iter().zip(expected_bits).enumerate() {
            if f32::from_bits(expected).is_nan() {
                assert!(actual.is_nan(), "{case} {field_name} index {index}");
            } else {
                assert_eq!(
                    actual.to_bits(),
                    expected,
                    "{case} {field_name} index {index}"
                );
            }
        }
    }

    fn output_bits(fixture: &Fixture) -> Vec<u32> {
        fixture
            .volume_outputs()
            .into_iter()
            .chain([&fixture.column_mass_tendency])
            .flat_map(|field| field.values().iter().map(|value| value.to_bits()))
            .collect()
    }

    impl Fixture {
        fn volume_outputs(&self) -> [&CpuField<f32>; 5] {
            [
                &self.west_east_tendency,
                &self.south_north_tendency,
                &self.geopotential_tendency,
                &self.temperature_tendency,
                &self.vertical_tendency,
            ]
        }
    }
}
