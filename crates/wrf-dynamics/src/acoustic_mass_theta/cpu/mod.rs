use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, ParallelExecutionError};

use crate::{
    AcousticMassThetaBoundaryPolicy, AcousticMassThetaDiagnostics, AcousticMassThetaError,
    AcousticMassThetaInputs, AcousticMassThetaKernels, AcousticMassThetaParameters,
    AcousticMassThetaRegion, AcousticMassThetaResult, AcousticMassThetaState,
};

mod column_mass;
mod flux_divergence;
mod potential_temperature;
mod validation;
mod vertical_mass_flux;

impl AcousticMassThetaKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn advance_acoustic_mass_and_theta(
        &self,
        state: AcousticMassThetaState<'_, Self::Field>,
        diagnostics: AcousticMassThetaDiagnostics<'_, Self::Field>,
        inputs: AcousticMassThetaInputs<'_, Self::Field>,
        parameters: AcousticMassThetaParameters,
        boundary_policy: AcousticMassThetaBoundaryPolicy,
        region: &AcousticMassThetaRegion,
    ) -> AcousticMassThetaResult<()> {
        let ranges =
            validation::validate_operation(&state, &diagnostics, inputs, boundary_policy, region)?;
        let AcousticMassThetaState {
            vertical_mass_flux,
            column_mass,
            potential_temperature,
        } = state;
        let AcousticMassThetaDiagnostics {
            time_centered_column_mass,
            coupled_column_mass,
            divergence_damping_mass_tendency,
            previous_potential_temperature,
        } = diagnostics;

        flux_divergence::calculate(
            self,
            previous_potential_temperature,
            inputs,
            parameters,
            region,
            &ranges,
        )?;
        flux_divergence::integrate_column_tendency(
            self,
            time_centered_column_mass,
            previous_potential_temperature,
            inputs.vertical,
            region,
            &ranges,
        )?;
        column_mass::save_previous(self, coupled_column_mass, column_mass, region, &ranges)?;
        column_mass::advance(
            self,
            column_mass,
            time_centered_column_mass,
            inputs.mass.tendency,
            parameters,
            region,
            &ranges,
        )?;
        column_mass::save_damping_tendency(
            self,
            divergence_damping_mass_tendency,
            time_centered_column_mass,
            inputs.mass.tendency,
            region,
            &ranges,
        )?;
        vertical_mass_flux::advance(
            self,
            vertical_mass_flux,
            previous_potential_temperature,
            time_centered_column_mass,
            inputs,
            region,
            &ranges,
        )?;
        column_mass::center(
            self,
            time_centered_column_mass,
            column_mass,
            coupled_column_mass,
            parameters,
            region,
            &ranges,
        )?;
        column_mass::couple(
            self,
            coupled_column_mass,
            column_mass,
            inputs.mass.base_full,
            region,
            &ranges,
        )?;
        potential_temperature::add_tendency(
            self,
            potential_temperature,
            previous_potential_temperature,
            inputs,
            parameters,
            region,
            &ranges,
        )?;
        potential_temperature::transport(
            self,
            potential_temperature,
            vertical_mass_flux,
            inputs,
            parameters,
            region,
            &ranges,
        )
    }
}

pub(crate) fn validate_acoustic_mass_and_theta(
    state: &AcousticMassThetaState<'_, CpuField<f32>>,
    diagnostics: &AcousticMassThetaDiagnostics<'_, CpuField<f32>>,
    inputs: AcousticMassThetaInputs<'_, CpuField<f32>>,
    boundary_policy: AcousticMassThetaBoundaryPolicy,
    region: &AcousticMassThetaRegion,
) -> AcousticMassThetaResult<()> {
    validation::validate_operation(state, diagnostics, inputs, boundary_policy, region).map(|_| ())
}

fn map_parallel_error(error: ParallelExecutionError<Infallible>) -> AcousticMassThetaError {
    match error {
        ParallelExecutionError::Kernel(never) => match never {},
        ParallelExecutionError::WorkerPanicked => AcousticMassThetaError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. }
        | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
            AcousticMassThetaError::SchedulerContractViolated
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;
    use crate::{
        AcousticMassThetaLateralDomain, AcousticMassThetaMapFactors, AcousticMassThetaMassInputs,
        AcousticMassThetaMomentumInputs, AcousticMassThetaThermodynamicInputs,
        AcousticMassThetaVerticalCoefficients, AcousticMassThetaWestEastPeriodicity,
    };

    #[derive(Clone, Copy)]
    struct OracleCase {
        name: &'static str,
        lateral_domain: AcousticMassThetaLateralDomain,
        periodicity: AcousticMassThetaWestEastPeriodicity,
        west_east_tile: (usize, usize),
        south_north_tile: (usize, usize),
    }

    struct Fixture {
        vertical_mass_flux: CpuField<f32>,
        column_mass: CpuField<f32>,
        potential_temperature: CpuField<f32>,
        time_centered_column_mass: CpuField<f32>,
        coupled_column_mass: CpuField<f32>,
        damping_tendency: CpuField<f32>,
        previous_potential_temperature: CpuField<f32>,
        west_east_momentum: CpuField<f32>,
        saved_west_east_momentum: CpuField<f32>,
        south_north_momentum: CpuField<f32>,
        saved_south_north_momentum: CpuField<f32>,
        saved_vertical_mass_flux: CpuField<f32>,
        saved_potential_temperature: CpuField<f32>,
        potential_temperature_tendency: CpuField<f32>,
        base_full_column_mass: CpuField<f32>,
        west_east_column_mass: CpuField<f32>,
        south_north_column_mass: CpuField<f32>,
        column_mass_tendency: CpuField<f32>,
        west_east_y_map: CpuField<f32>,
        inverse_south_north_x_map: CpuField<f32>,
        mass_point_x_map: CpuField<f32>,
        mass_point_y_map: CpuField<f32>,
        vertical_coefficients: [Vec<f32>; 6],
    }

    #[test]
    fn matches_direct_fortran_for_global_nested_periodic_and_partial_tiles() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let expected = expected_oracle_bits();

        for case in oracle_cases() {
            let mut fixture = Fixture::new(&backend);
            apply(&backend, &mut fixture, case).unwrap();

            for (field_name, actual) in output_fields(&fixture) {
                assert_eq!(
                    field_bits(actual),
                    expected[&(case.name, field_name)],
                    "{field_name} differs for {}",
                    case.name
                );
            }
        }
    }

    #[test]
    fn is_bitwise_deterministic_across_worker_counts() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();

        for case in oracle_cases() {
            let mut one = Fixture::new(&one_worker);
            let mut four = Fixture::new(&four_workers);
            apply(&one_worker, &mut one, case).unwrap();
            apply(&four_workers, &mut four, case).unwrap();

            for ((field_name, one_field), (_, four_field)) in
                output_fields(&one).into_iter().zip(output_fields(&four))
            {
                assert_eq!(
                    one_field, four_field,
                    "{field_name} differs for {}",
                    case.name
                );
            }
        }
    }

    #[test]
    fn rejects_a_coefficient_mismatch_before_mutating_any_output() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut fixture = Fixture::new(&backend);
        fixture.vertical_coefficients[5].pop();
        let before = output_bit_sets(&fixture);

        assert_eq!(
            apply(&backend, &mut fixture, oracle_cases()[0]),
            Err(AcousticMassThetaError::CoefficientLengthMismatch {
                coefficient: crate::AcousticMassThetaCoefficient::InverseHalfLevelEtaSpacing,
                expected: 6,
                actual: 5,
            })
        );
        assert_eq!(output_bit_sets(&fixture), before);
    }

    impl Fixture {
        fn new(backend: &CpuBackend) -> Self {
            let shape = GridShape::try_new(6, 6, 6).unwrap();
            let horizontal_shape = shape.horizontal_shape();
            let volume = |value| backend.create_field(shape, value).unwrap();
            let horizontal = |value| backend.create_field(horizontal_shape, value).unwrap();
            let mut fixture = Self {
                vertical_mass_flux: volume(0.0),
                column_mass: horizontal(0.0),
                potential_temperature: volume(0.0),
                time_centered_column_mass: horizontal(-902.0),
                coupled_column_mass: horizontal(-903.0),
                damping_tendency: horizontal(-904.0),
                previous_potential_temperature: volume(-901.0),
                west_east_momentum: volume(0.0),
                saved_west_east_momentum: volume(0.0),
                south_north_momentum: volume(0.0),
                saved_south_north_momentum: volume(0.0),
                saved_vertical_mass_flux: volume(0.0),
                saved_potential_temperature: volume(0.0),
                potential_temperature_tendency: volume(0.0),
                base_full_column_mass: horizontal(0.0),
                west_east_column_mass: horizontal(0.0),
                south_north_column_mass: horizontal(0.0),
                column_mass_tendency: horizontal(0.0),
                west_east_y_map: horizontal(0.0),
                inverse_south_north_x_map: horizontal(0.0),
                mass_point_x_map: horizontal(0.0),
                mass_point_y_map: horizontal(0.0),
                vertical_coefficients: std::array::from_fn(|_| vec![0.0; 6]),
            };
            fixture.initialize();
            fixture
        }

        fn initialize(&mut self) {
            for bottom_top in 0..6 {
                self.vertical_coefficients[0][bottom_top] = 0.45 + bottom_top as f32 * 0.01;
                self.vertical_coefficients[1][bottom_top] = 0.2 - bottom_top as f32 * 0.005;
                self.vertical_coefficients[2][bottom_top] = 0.18 + bottom_top as f32 * 0.007;
                self.vertical_coefficients[3][bottom_top] = 0.61 + bottom_top as f32 * 0.002;
                self.vertical_coefficients[4][bottom_top] = 0.39 - bottom_top as f32 * 0.002;
                self.vertical_coefficients[5][bottom_top] = 1.1 + bottom_top as f32 * 0.03;
            }
            for south_north in 0..6 {
                for west_east in 0..6 {
                    let horizontal_index = south_north * 6 + west_east;
                    self.column_mass.values_mut()[horizontal_index] =
                        2.0 + west_east as f32 * 0.11 + south_north as f32 * 0.17;
                    self.base_full_column_mass.values_mut()[horizontal_index] =
                        11.0 + west_east as f32 * 0.13 - south_north as f32 * 0.09;
                    self.west_east_column_mass.values_mut()[horizontal_index] =
                        3.0 + west_east as f32 * 0.07 + south_north as f32 * 0.02;
                    self.south_north_column_mass.values_mut()[horizontal_index] =
                        4.0 - west_east as f32 * 0.03 + south_north as f32 * 0.08;
                    self.column_mass_tendency.values_mut()[horizontal_index] =
                        0.03 + west_east as f32 * 0.002 - south_north as f32 * 0.001;
                    self.west_east_y_map.values_mut()[horizontal_index] =
                        0.92 + west_east as f32 * 0.006 + south_north as f32 * 0.003;
                    self.inverse_south_north_x_map.values_mut()[horizontal_index] =
                        1.08 - west_east as f32 * 0.004 + south_north as f32 * 0.002;
                    self.mass_point_x_map.values_mut()[horizontal_index] =
                        1.03 + west_east as f32 * 0.003 - south_north as f32 * 0.002;
                    self.mass_point_y_map.values_mut()[horizontal_index] =
                        0.97 - west_east as f32 * 0.002 + south_north as f32 * 0.004;
                }
                for bottom_top in 0..6 {
                    for west_east in 0..6 {
                        let index = volume_index(west_east, bottom_top, south_north);
                        self.west_east_momentum.values_mut()[index] = 0.2
                            + west_east as f32 * 0.013
                            + bottom_top as f32 * 0.017
                            + south_north as f32 * 0.019;
                        self.saved_west_east_momentum.values_mut()[index] = 0.15
                            - west_east as f32 * 0.006
                            + bottom_top as f32 * 0.011
                            + south_north as f32 * 0.004;
                        self.south_north_momentum.values_mut()[index] = 0.3
                            - west_east as f32 * 0.009
                            + bottom_top as f32 * 0.014
                            + south_north as f32 * 0.021;
                        self.saved_south_north_momentum.values_mut()[index] =
                            0.12 + west_east as f32 * 0.005 - bottom_top as f32 * 0.003
                                + south_north as f32 * 0.008;
                        self.vertical_mass_flux.values_mut()[index] =
                            0.8 + west_east as f32 * 0.02 + bottom_top as f32 * 0.03
                                - south_north as f32 * 0.01;
                        self.saved_vertical_mass_flux.values_mut()[index] = 0.35
                            - west_east as f32 * 0.004
                            + bottom_top as f32 * 0.006
                            + south_north as f32 * 0.003;
                        self.potential_temperature.values_mut()[index] =
                            300.0 + west_east as f32 * 0.7 + bottom_top as f32 * 1.1
                                - south_north as f32 * 0.4;
                        self.saved_potential_temperature.values_mut()[index] = 290.0
                            - west_east as f32 * 0.3
                            + bottom_top as f32 * 0.9
                            + south_north as f32 * 0.5;
                        self.potential_temperature_tendency.values_mut()[index] =
                            0.012 + west_east as f32 * 0.0003 - bottom_top as f32 * 0.0002
                                + south_north as f32 * 0.0001;
                    }
                }
            }
        }
    }

    fn apply(
        backend: &CpuBackend,
        fixture: &mut Fixture,
        case: OracleCase,
    ) -> AcousticMassThetaResult<()> {
        let region = AcousticMassThetaRegion::try_new(
            GridShape::try_new(6, 6, 6).unwrap(),
            1..5,
            1..5,
            1..5,
            case.west_east_tile.0..case.west_east_tile.1,
            case.south_north_tile.0..case.south_north_tile.1,
            1..6,
        )?;
        backend.advance_acoustic_mass_and_theta(
            AcousticMassThetaState::new(
                &mut fixture.vertical_mass_flux,
                &mut fixture.column_mass,
                &mut fixture.potential_temperature,
            ),
            AcousticMassThetaDiagnostics::new(
                &mut fixture.time_centered_column_mass,
                &mut fixture.coupled_column_mass,
                &mut fixture.damping_tendency,
                &mut fixture.previous_potential_temperature,
            ),
            AcousticMassThetaInputs::new(
                AcousticMassThetaMomentumInputs::new(
                    &fixture.west_east_momentum,
                    &fixture.saved_west_east_momentum,
                    &fixture.south_north_momentum,
                    &fixture.saved_south_north_momentum,
                    &fixture.saved_vertical_mass_flux,
                ),
                AcousticMassThetaMassInputs::new(
                    &fixture.base_full_column_mass,
                    &fixture.west_east_column_mass,
                    &fixture.south_north_column_mass,
                    &fixture.column_mass_tendency,
                ),
                AcousticMassThetaThermodynamicInputs::new(
                    &fixture.saved_potential_temperature,
                    &fixture.potential_temperature_tendency,
                ),
                AcousticMassThetaMapFactors::new(
                    &fixture.west_east_y_map,
                    &fixture.inverse_south_north_x_map,
                    &fixture.mass_point_x_map,
                    &fixture.mass_point_y_map,
                ),
                AcousticMassThetaVerticalCoefficients::new(
                    &fixture.vertical_coefficients[0],
                    &fixture.vertical_coefficients[1],
                    &fixture.vertical_coefficients[2],
                    &fixture.vertical_coefficients[3],
                    &fixture.vertical_coefficients[4],
                    &fixture.vertical_coefficients[5],
                ),
            ),
            AcousticMassThetaParameters::new(0.002, 0.003, 0.4, 0.1),
            AcousticMassThetaBoundaryPolicy::new(case.lateral_domain, case.periodicity),
            &region,
        )
    }

    fn oracle_cases() -> [OracleCase; 4] {
        [
            OracleCase {
                name: "global",
                lateral_domain: AcousticMassThetaLateralDomain::Global,
                periodicity: AcousticMassThetaWestEastPeriodicity::Nonperiodic,
                west_east_tile: (1, 6),
                south_north_tile: (1, 6),
            },
            OracleCase {
                name: "nested",
                lateral_domain: AcousticMassThetaLateralDomain::SpecifiedOrNested,
                periodicity: AcousticMassThetaWestEastPeriodicity::Nonperiodic,
                west_east_tile: (1, 6),
                south_north_tile: (1, 6),
            },
            OracleCase {
                name: "nested_periodic",
                lateral_domain: AcousticMassThetaLateralDomain::SpecifiedOrNested,
                periodicity: AcousticMassThetaWestEastPeriodicity::Periodic,
                west_east_tile: (1, 6),
                south_north_tile: (1, 6),
            },
            OracleCase {
                name: "partial",
                lateral_domain: AcousticMassThetaLateralDomain::Global,
                periodicity: AcousticMassThetaWestEastPeriodicity::Nonperiodic,
                west_east_tile: (2, 5),
                south_north_tile: (2, 5),
            },
        ]
    }

    fn output_fields(fixture: &Fixture) -> [(&'static str, &CpuField<f32>); 7] {
        [
            ("ww", &fixture.vertical_mass_flux),
            ("t", &fixture.potential_temperature),
            ("tave", &fixture.previous_potential_temperature),
            ("mu", &fixture.column_mass),
            ("muave", &fixture.time_centered_column_mass),
            ("muts", &fixture.coupled_column_mass),
            ("mudf", &fixture.damping_tendency),
        ]
    }

    fn output_bit_sets(fixture: &Fixture) -> Vec<Vec<u32>> {
        output_fields(fixture)
            .into_iter()
            .map(|(_, field)| field_bits(field))
            .collect()
    }

    fn expected_oracle_bits() -> HashMap<(&'static str, &'static str), Vec<u32>> {
        let mut expected = HashMap::new();
        for line in include_str!("../../../test-data/acoustic_mass_theta.out.correct").lines() {
            let mut parts = line.split_whitespace();
            let case = intern_case(parts.next().unwrap());
            let field = intern_field(parts.next().unwrap());
            let remaining: Vec<_> = parts.collect();
            let bits = u32::from_str_radix(remaining.last().unwrap(), 16).unwrap();
            expected
                .entry((case, field))
                .or_insert_with(Vec::new)
                .push(bits);
        }
        expected
    }

    fn intern_case(name: &str) -> &'static str {
        match name {
            "global" => "global",
            "nested" => "nested",
            "nested_periodic" => "nested_periodic",
            "partial" => "partial",
            _ => panic!("unknown oracle case {name}"),
        }
    }

    fn intern_field(name: &str) -> &'static str {
        match name {
            "ww" => "ww",
            "t" => "t",
            "tave" => "tave",
            "mu" => "mu",
            "muave" => "muave",
            "muts" => "muts",
            "mudf" => "mudf",
            _ => panic!("unknown oracle field {name}"),
        }
    }

    fn field_bits(field: &CpuField<f32>) -> Vec<u32> {
        field.values().iter().map(|value| value.to_bits()).collect()
    }

    const fn volume_index(west_east: usize, bottom_top: usize, south_north: usize) -> usize {
        (south_north * 6 + bottom_top) * 6 + west_east
    }
}
