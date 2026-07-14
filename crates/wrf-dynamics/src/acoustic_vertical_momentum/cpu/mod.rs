use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, ParallelExecutionError};

use crate::{
    AcousticVerticalAdvection, AcousticVerticalBoundaryPolicy, AcousticVerticalDamping,
    AcousticVerticalError, AcousticVerticalInputs, AcousticVerticalKernels,
    AcousticVerticalParameters, AcousticVerticalRegion, AcousticVerticalResult,
    AcousticVerticalState, AcousticVerticalWorkspace, VerticalAcousticTopBoundary,
};

use self::geopotential::GeopotentialUpdate;
use self::momentum::VerticalMomentumAdvance;
use self::right_hand_side::RightHandSideCalculation;
use self::validation::OperationValidation;

mod geopotential;
mod momentum;
mod right_hand_side;
mod validation;

impl AcousticVerticalKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn advance_acoustic_vertical_momentum(
        &self,
        state: AcousticVerticalState<'_, Self::Field>,
        workspace: AcousticVerticalWorkspace<'_, Self::Field>,
        inputs: AcousticVerticalInputs<'_, Self::Field>,
        parameters: AcousticVerticalParameters,
        boundary_policy: AcousticVerticalBoundaryPolicy,
        top_boundary: VerticalAcousticTopBoundary,
        vertical_advection: AcousticVerticalAdvection,
        damping: AcousticVerticalDamping,
        region: &AcousticVerticalRegion,
    ) -> AcousticVerticalResult<()> {
        let ranges =
            OperationValidation::validate(&state, &workspace, inputs, boundary_policy, region)?;
        let AcousticVerticalState {
            vertical_momentum,
            perturbation_geopotential,
            time_averaged_thermodynamics,
        } = state;

        RightHandSideCalculation::new(
            inputs,
            parameters,
            top_boundary,
            vertical_advection,
            region,
            &ranges,
        )
        .calculate(
            self,
            time_averaged_thermodynamics,
            workspace.geopotential_right_hand_side,
            perturbation_geopotential,
            vertical_momentum,
        )?;
        VerticalMomentumAdvance::new(
            inputs,
            parameters,
            top_boundary,
            damping,
            region,
            &ranges,
            workspace.geopotential_right_hand_side,
            perturbation_geopotential,
            time_averaged_thermodynamics,
        )
        .advance(self, vertical_momentum)?;
        GeopotentialUpdate::new(
            inputs,
            parameters,
            region,
            &ranges,
            workspace.geopotential_right_hand_side,
            vertical_momentum,
        )
        .update(self, perturbation_geopotential)
    }
}

pub(crate) fn validate_acoustic_vertical_momentum(
    state: &AcousticVerticalState<'_, CpuField<f32>>,
    workspace: &AcousticVerticalWorkspace<'_, CpuField<f32>>,
    inputs: AcousticVerticalInputs<'_, CpuField<f32>>,
    boundary_policy: AcousticVerticalBoundaryPolicy,
    region: &AcousticVerticalRegion,
) -> AcousticVerticalResult<()> {
    OperationValidation::validate(state, workspace, inputs, boundary_policy, region).map(|_| ())
}

pub(super) fn map_parallel_error(
    error: ParallelExecutionError<Infallible>,
) -> AcousticVerticalError {
    match error {
        ParallelExecutionError::Kernel(never) => match never {},
        ParallelExecutionError::WorkerPanicked => AcousticVerticalError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. }
        | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
            AcousticVerticalError::SchedulerContractViolated
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;
    use crate::{
        AcousticVerticalGeopotentialInputs, AcousticVerticalLateralDomain,
        AcousticVerticalLevelCoefficients, AcousticVerticalMapFactors, AcousticVerticalMassInputs,
        AcousticVerticalMomentumInputs, AcousticVerticalSolveInputs,
        AcousticVerticalThermodynamicInputs, AcousticVerticalWestEastPeriodicity,
    };

    #[derive(Clone, Copy)]
    struct OracleCase {
        name: &'static str,
        lateral_domain: AcousticVerticalLateralDomain,
        periodicity: AcousticVerticalWestEastPeriodicity,
        advection: AcousticVerticalAdvection,
        top_boundary: VerticalAcousticTopBoundary,
        damping: AcousticVerticalDamping,
        west_east_tile: (usize, usize),
        south_north_tile: (usize, usize),
    }

    #[test]
    fn matches_fortran_for_advection_boundaries_damping_and_partial_tiles() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let expected = expected_oracle_bits();

        for case in oracle_cases() {
            let mut fixture = Fixture::new(&backend);
            fixture.apply(&backend, case).unwrap();
            for (field_name, field) in fixture.outputs() {
                let actual = field
                    .values()
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>();
                let expected = &expected[&(case.name.to_owned(), field_name.to_owned())];
                for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
                    assert_eq!(
                        actual, expected,
                        "{field_name} differs for {} at linear index {index}",
                        case.name
                    );
                }
            }
        }
    }

    #[test]
    fn one_and_four_workers_produce_identical_bits() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();

        for case in oracle_cases() {
            let mut serial = Fixture::new(&one_worker);
            let mut parallel = Fixture::new(&four_workers);
            serial.apply(&one_worker, case).unwrap();
            parallel.apply(&four_workers, case).unwrap();
            for ((field_name, serial_field), (_, parallel_field)) in
                serial.outputs().into_iter().zip(parallel.outputs())
            {
                assert_eq!(
                    serial_field, parallel_field,
                    "{field_name} differs for {}",
                    case.name
                );
            }
        }
    }

    #[test]
    fn coefficient_failure_is_atomic_across_all_state_fields() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut fixture = Fixture::new(&backend);
        fixture.vertical_coefficients[7].pop();
        let before = fixture.output_bits();

        assert_eq!(
            fixture.apply(&backend, oracle_cases()[0]),
            Err(AcousticVerticalError::CoefficientLengthMismatch {
                coefficient: crate::AcousticVerticalCoefficient::InverseFullLevelSpacing,
                expected: 6,
                actual: 5,
            })
        );
        assert_eq!(fixture.output_bits(), before);
    }

    #[test]
    fn workspace_shape_failure_is_atomic_across_all_state_fields() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut fixture = Fixture::new(&backend);
        fixture.workspace = backend
            .create_field(GridShape::try_new(6, 5, 6).unwrap(), 0.0)
            .unwrap();
        let before = fixture.output_bits();

        assert_eq!(
            fixture.apply(&backend, oracle_cases()[0]),
            Err(AcousticVerticalError::FieldShapeMismatch {
                field: crate::AcousticVerticalField::GeopotentialRightHandSide,
                expected: GridShape::try_new(6, 6, 6).unwrap(),
                actual: GridShape::try_new(6, 5, 6).unwrap(),
            })
        );
        assert_eq!(fixture.output_bits(), before);
    }

    #[test]
    fn zero_depth_damping_preserves_wrf_ieee_nan_behavior_at_the_top() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut fixture = Fixture::new(&backend);
        let mut case = oracle_cases()[0];
        case.damping = AcousticVerticalDamping::UpperLayer {
            coefficient: 0.15,
            depth: 0.0,
        };

        fixture.apply(&backend, case).unwrap();

        let top_index = volume_index(1, 5, 1);
        let lower_index = volume_index(1, 4, 1);
        assert!(fixture.vertical_momentum.values()[top_index].is_nan());
        assert!(fixture.perturbation_geopotential.values()[top_index].is_nan());
        assert!(fixture.vertical_momentum.values()[lower_index].is_finite());
    }

    struct Fixture {
        vertical_momentum: CpuField<f32>,
        perturbation_geopotential: CpuField<f32>,
        time_averaged_thermodynamics: CpuField<f32>,
        workspace: CpuField<f32>,
        vertical_momentum_tendency: CpuField<f32>,
        vertical_mass_flux: CpuField<f32>,
        saved_vertical_momentum: CpuField<f32>,
        west_east_momentum: CpuField<f32>,
        south_north_momentum: CpuField<f32>,
        potential_temperature: CpuField<f32>,
        saved_potential_temperature: CpuField<f32>,
        saved_perturbation_geopotential: CpuField<f32>,
        base_geopotential: CpuField<f32>,
        geopotential_tendency: CpuField<f32>,
        pressure_coefficient: CpuField<f32>,
        moisture_coefficient: CpuField<f32>,
        inverse_density: CpuField<f32>,
        lower_diagonal: CpuField<f32>,
        inverse_eliminated_diagonal: CpuField<f32>,
        upper_elimination_factor: CpuField<f32>,
        full_column_mass: CpuField<f32>,
        time_centered_column_mass: CpuField<f32>,
        coupled_column_mass: CpuField<f32>,
        terrain_height: CpuField<f32>,
        west_east_map_factor: CpuField<f32>,
        south_north_map_factor: CpuField<f32>,
        vertical_coefficients: [Vec<f32>; 8],
    }

    impl Fixture {
        fn new(backend: &CpuBackend) -> Self {
            let shape = GridShape::try_new(6, 6, 6).unwrap();
            let volume = |value| backend.create_field(shape, value).unwrap();
            let horizontal = |value| {
                backend
                    .create_field(shape.horizontal_shape(), value)
                    .unwrap()
            };
            let mut fixture = Self {
                vertical_momentum: volume(0.0),
                perturbation_geopotential: volume(0.0),
                time_averaged_thermodynamics: volume(0.0),
                workspace: volume(-999.0),
                vertical_momentum_tendency: volume(0.0),
                vertical_mass_flux: volume(0.0),
                saved_vertical_momentum: volume(0.0),
                west_east_momentum: volume(0.0),
                south_north_momentum: volume(0.0),
                potential_temperature: volume(0.0),
                saved_potential_temperature: volume(0.0),
                saved_perturbation_geopotential: volume(0.0),
                base_geopotential: volume(0.0),
                geopotential_tendency: volume(0.0),
                pressure_coefficient: volume(0.0),
                moisture_coefficient: volume(0.0),
                inverse_density: volume(0.0),
                lower_diagonal: volume(0.0),
                inverse_eliminated_diagonal: volume(0.0),
                upper_elimination_factor: volume(0.0),
                full_column_mass: horizontal(0.0),
                time_centered_column_mass: horizontal(0.0),
                coupled_column_mass: horizontal(0.0),
                terrain_height: horizontal(0.0),
                west_east_map_factor: horizontal(0.0),
                south_north_map_factor: horizontal(0.0),
                vertical_coefficients: std::array::from_fn(|_| vec![0.0; 6]),
            };
            fixture.initialize();
            fixture
        }

        fn initialize(&mut self) {
            for bottom_top in 0..6 {
                let level = bottom_top as f32;
                self.vertical_coefficients[0][bottom_top] = 0.42 + level * 0.011;
                self.vertical_coefficients[1][bottom_top] = 0.19 - level * 0.004;
                self.vertical_coefficients[2][bottom_top] = 0.37 + level * 0.009;
                self.vertical_coefficients[3][bottom_top] = 0.23 - level * 0.003;
                self.vertical_coefficients[4][bottom_top] = 0.58 + level * 0.006;
                self.vertical_coefficients[5][bottom_top] = 0.42 - level * 0.006;
                self.vertical_coefficients[6][bottom_top] = 1.05 + level * 0.025;
                self.vertical_coefficients[7][bottom_top] = 0.91 + level * 0.018;
            }
            for south_north in 0..6 {
                let south_north_value = south_north as f32;
                for west_east in 0..6 {
                    let west_east_value = west_east as f32;
                    let horizontal_index = south_north * 6 + west_east;
                    self.full_column_mass.values_mut()[horizontal_index] =
                        11.0 + west_east_value * 0.13 - south_north_value * 0.09;
                    self.time_centered_column_mass.values_mut()[horizontal_index] =
                        2.1 + west_east_value * 0.07 + south_north_value * 0.04;
                    self.coupled_column_mass.values_mut()[horizontal_index] =
                        12.7 - west_east_value * 0.03 + south_north_value * 0.08;
                    self.terrain_height.values_mut()[horizontal_index] = 140.0
                        + west_east_value * 9.0
                        + south_north_value * 13.0
                        + (west_east * south_north) as f32 * 0.7;
                    self.west_east_map_factor.values_mut()[horizontal_index] =
                        1.03 + west_east_value * 0.003 - south_north_value * 0.002;
                    self.south_north_map_factor.values_mut()[horizontal_index] =
                        0.97 - west_east_value * 0.002 + south_north_value * 0.004;
                    for bottom_top in 0..6 {
                        let level = bottom_top as f32;
                        let index = volume_index(west_east, bottom_top, south_north);
                        self.vertical_momentum.values_mut()[index] =
                            0.8 + west_east_value * 0.02 + level * 0.03 - south_north_value * 0.01;
                        self.vertical_momentum_tendency.values_mut()[index] =
                            0.012 + west_east_value * 0.0003 - level * 0.0002
                                + south_north_value * 0.0001;
                        self.vertical_mass_flux.values_mut()[index] = 0.35
                            - west_east_value * 0.004
                            + level * 0.006
                            + south_north_value * 0.003;
                        self.saved_vertical_momentum.values_mut()[index] =
                            0.31 + west_east_value * 0.005 - level * 0.004
                                + south_north_value * 0.002;
                        self.west_east_momentum.values_mut()[index] = 0.2
                            + west_east_value * 0.013
                            + level * 0.017
                            + south_north_value * 0.019;
                        self.south_north_momentum.values_mut()[index] = 0.3
                            - west_east_value * 0.009
                            + level * 0.014
                            + south_north_value * 0.021;
                        self.time_averaged_thermodynamics.values_mut()[index] =
                            294.0 + west_east_value * 0.2 + level * 0.7 - south_north_value * 0.15;
                        self.potential_temperature.values_mut()[index] =
                            300.0 + west_east_value * 0.7 + level * 1.1 - south_north_value * 0.4;
                        self.saved_potential_temperature.values_mut()[index] =
                            1.3 - west_east_value * 0.03 + level * 0.09 + south_north_value * 0.05;
                        self.perturbation_geopotential.values_mut()[index] =
                            20.0 + west_east_value * 0.8 + level * 2.1 - south_north_value * 0.6;
                        self.saved_perturbation_geopotential.values_mut()[index] =
                            18.0 - west_east_value * 0.4 + level * 1.7 + south_north_value * 0.3;
                        self.base_geopotential.values_mut()[index] = 500.0
                            + level * 1000.0
                            + west_east_value * 3.0
                            + south_north_value * 4.0;
                        self.geopotential_tendency.values_mut()[index] =
                            0.05 + west_east_value * 0.002 - level * 0.001
                                + south_north_value * 0.003;
                        self.pressure_coefficient.values_mut()[index] =
                            1.1 + west_east_value * 0.004 + level * 0.008
                                - south_north_value * 0.003;
                        self.moisture_coefficient.values_mut()[index] = 0.82
                            - west_east_value * 0.002
                            + level * 0.005
                            + south_north_value * 0.001;
                        self.inverse_density.values_mut()[index] = 0.9 + west_east_value * 0.006
                            - level * 0.004
                            + south_north_value * 0.002;
                        self.lower_diagonal.values_mut()[index] =
                            -0.03 + level * 0.0007 - west_east_value * 0.0002;
                        self.inverse_eliminated_diagonal.values_mut()[index] =
                            0.83 - level * 0.002 + south_north_value * 0.0003;
                        self.upper_elimination_factor.values_mut()[index] =
                            -0.02 + level * 0.0004 + west_east_value * 0.0001;
                    }
                }
            }
        }

        fn apply(&mut self, backend: &CpuBackend, case: OracleCase) -> AcousticVerticalResult<()> {
            let region = AcousticVerticalRegion::try_new(
                GridShape::try_new(6, 6, 6).unwrap(),
                1..5,
                1..5,
                1..5,
                case.west_east_tile.0..case.west_east_tile.1,
                case.south_north_tile.0..case.south_north_tile.1,
                1..6,
            )?;
            backend.advance_acoustic_vertical_momentum(
                AcousticVerticalState::new(
                    &mut self.vertical_momentum,
                    &mut self.perturbation_geopotential,
                    &mut self.time_averaged_thermodynamics,
                ),
                AcousticVerticalWorkspace::new(&mut self.workspace),
                AcousticVerticalInputs::new(
                    AcousticVerticalMomentumInputs::new(
                        &self.vertical_momentum_tendency,
                        &self.vertical_mass_flux,
                        &self.saved_vertical_momentum,
                        &self.west_east_momentum,
                        &self.south_north_momentum,
                    ),
                    AcousticVerticalMassInputs::new(
                        &self.full_column_mass,
                        &self.time_centered_column_mass,
                        &self.coupled_column_mass,
                    ),
                    AcousticVerticalThermodynamicInputs::new(
                        &self.potential_temperature,
                        &self.saved_potential_temperature,
                    ),
                    AcousticVerticalGeopotentialInputs::new(
                        &self.saved_perturbation_geopotential,
                        &self.base_geopotential,
                        &self.geopotential_tendency,
                    ),
                    AcousticVerticalMapFactors::new(
                        &self.terrain_height,
                        &self.west_east_map_factor,
                        &self.south_north_map_factor,
                    ),
                    AcousticVerticalSolveInputs::new(
                        &self.pressure_coefficient,
                        &self.moisture_coefficient,
                        &self.inverse_density,
                        &self.lower_diagonal,
                        &self.inverse_eliminated_diagonal,
                        &self.upper_elimination_factor,
                    ),
                    AcousticVerticalLevelCoefficients::new(
                        &self.vertical_coefficients[0],
                        &self.vertical_coefficients[1],
                        &self.vertical_coefficients[2],
                        &self.vertical_coefficients[3],
                        &self.vertical_coefficients[4],
                        &self.vertical_coefficients[5],
                        &self.vertical_coefficients[6],
                        &self.vertical_coefficients[7],
                    ),
                ),
                AcousticVerticalParameters::new(0.002, 0.003, 0.4, 300.0, 0.1, [0.5, 0.3, 0.2]),
                AcousticVerticalBoundaryPolicy::new(case.lateral_domain, case.periodicity),
                case.top_boundary,
                case.advection,
                case.damping,
                &region,
            )
        }

        fn outputs(&self) -> [(&'static str, &CpuField<f32>); 3] {
            [
                ("tave", &self.time_averaged_thermodynamics),
                ("w", &self.vertical_momentum),
                ("ph", &self.perturbation_geopotential),
            ]
        }

        fn output_bits(&self) -> Vec<Vec<u32>> {
            self.outputs()
                .iter()
                .map(|(_, field)| field.values().iter().map(|value| value.to_bits()).collect())
                .collect()
        }
    }

    fn oracle_cases() -> [OracleCase; 4] {
        [
            OracleCase {
                name: "global_gradient",
                lateral_domain: AcousticVerticalLateralDomain::Global,
                periodicity: AcousticVerticalWestEastPeriodicity::Nonperiodic,
                advection: AcousticVerticalAdvection::StaggeredGeopotentialGradient,
                top_boundary: VerticalAcousticTopBoundary::Nonrigid,
                damping: AcousticVerticalDamping::Disabled,
                west_east_tile: (1, 6),
                south_north_tile: (1, 6),
            },
            OracleCase {
                name: "nested_product_rigid_damped",
                lateral_domain: AcousticVerticalLateralDomain::SpecifiedOrNested,
                periodicity: AcousticVerticalWestEastPeriodicity::Nonperiodic,
                advection: AcousticVerticalAdvection::StaggeredTransportProduct,
                top_boundary: VerticalAcousticTopBoundary::RigidLid,
                damping: AcousticVerticalDamping::UpperLayer {
                    coefficient: 0.15,
                    depth: 220.0,
                },
                west_east_tile: (1, 6),
                south_north_tile: (1, 6),
            },
            OracleCase {
                name: "nested_periodic_gradient_damped",
                lateral_domain: AcousticVerticalLateralDomain::SpecifiedOrNested,
                periodicity: AcousticVerticalWestEastPeriodicity::Periodic,
                advection: AcousticVerticalAdvection::StaggeredGeopotentialGradient,
                top_boundary: VerticalAcousticTopBoundary::Nonrigid,
                damping: AcousticVerticalDamping::UpperLayer {
                    coefficient: 0.15,
                    depth: 220.0,
                },
                west_east_tile: (1, 6),
                south_north_tile: (1, 6),
            },
            OracleCase {
                name: "partial_product_rigid",
                lateral_domain: AcousticVerticalLateralDomain::Global,
                periodicity: AcousticVerticalWestEastPeriodicity::Nonperiodic,
                advection: AcousticVerticalAdvection::StaggeredTransportProduct,
                top_boundary: VerticalAcousticTopBoundary::RigidLid,
                damping: AcousticVerticalDamping::Disabled,
                west_east_tile: (2, 5),
                south_north_tile: (2, 5),
            },
        ]
    }

    fn volume_index(west_east: usize, bottom_top: usize, south_north: usize) -> usize {
        (south_north * 6 + bottom_top) * 6 + west_east
    }

    fn expected_oracle_bits() -> HashMap<(String, String), Vec<u32>> {
        let mut expected = HashMap::new();
        for line in
            include_str!("../../../test-data/acoustic_vertical_momentum.out.correct").lines()
        {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            expected
                .entry((parts[0].to_owned(), parts[1].to_owned()))
                .or_insert_with(Vec::new)
                .push(u32::from_str_radix(parts[5], 16).unwrap());
        }
        expected
    }
}
