use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, ParallelExecutionError};

use crate::{
    AcousticFluxAccumulationError, AcousticFluxAccumulationKernels, AcousticFluxAccumulationRegion,
    AcousticFluxAccumulationResult, AcousticFluxCurrentFields, AcousticFluxLinearFields,
    AcousticFluxMapFactors, AcousticFluxMassFields, AcousticFluxRunningAverages,
    AcousticSubstepPhase,
};

mod update;
mod validation;

impl AcousticFluxAccumulationKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn accumulate_acoustic_fluxes(
        &self,
        averages: AcousticFluxRunningAverages<'_, Self::Field>,
        current: AcousticFluxCurrentFields<'_, Self::Field>,
        linear: AcousticFluxLinearFields<'_, Self::Field>,
        masses: AcousticFluxMassFields<'_, Self::Field>,
        map_factors: AcousticFluxMapFactors<'_, Self::Field>,
        column_mass_multiplier: &[f32],
        column_mass_offset: &[f32],
        phase: AcousticSubstepPhase,
        region: &AcousticFluxAccumulationRegion,
    ) -> AcousticFluxAccumulationResult<()> {
        validation::validate_operation(
            &averages,
            &current,
            &linear,
            &masses,
            &map_factors,
            column_mass_multiplier,
            column_mass_offset,
            region,
        )?;
        update::apply(
            self,
            averages,
            current,
            linear,
            masses,
            map_factors,
            column_mass_multiplier,
            column_mass_offset,
            phase,
            region,
        )
    }
}

fn map_parallel_error(error: ParallelExecutionError<Infallible>) -> AcousticFluxAccumulationError {
    match error {
        ParallelExecutionError::Kernel(never) => match never {},
        ParallelExecutionError::WorkerPanicked => AcousticFluxAccumulationError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. }
        | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
            AcousticFluxAccumulationError::SchedulerContractViolated
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;
    use crate::{
        AcousticFluxCoefficient, AcousticFluxField, AcousticFluxMapFactors, AcousticFluxMassFields,
    };

    #[test]
    fn three_substeps_match_direct_fortran_sumflux_bits() {
        let backend = CpuBackend::try_with_worker_count(4).unwrap();
        let mut fixture = Fixture::new(&backend);

        for iteration in 1..=3 {
            fixture.set_current_fluxes(iteration);
            fixture
                .apply(
                    &backend,
                    AcousticSubstepPhase::try_new(iteration, 3).unwrap(),
                )
                .unwrap();
        }

        let expected = oracle_bits();
        assert_bits("ru_m", fixture.average_west_east.values(), &expected);
        assert_bits("rv_m", fixture.average_south_north.values(), &expected);
        assert_bits("ww_m", fixture.average_vertical.values(), &expected);
    }

    #[test]
    fn invalid_coefficient_is_failure_atomic() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut fixture = Fixture::new(&backend);
        let before = fixture.average_west_east.values().to_vec();
        let region = Fixture::region();
        let error = backend
            .accumulate_acoustic_fluxes(
                AcousticFluxRunningAverages {
                    west_east: &mut fixture.average_west_east,
                    south_north: &mut fixture.average_south_north,
                    vertical: &mut fixture.average_vertical,
                },
                AcousticFluxCurrentFields {
                    west_east: &fixture.current_west_east,
                    south_north: &fixture.current_south_north,
                    vertical: &fixture.current_vertical,
                },
                AcousticFluxLinearFields {
                    west_east: &fixture.linear_west_east,
                    south_north: &fixture.linear_south_north,
                    vertical: &fixture.linear_vertical,
                },
                AcousticFluxMassFields {
                    west_east: &fixture.mass_west_east,
                    south_north: &fixture.mass_south_north,
                },
                AcousticFluxMapFactors {
                    west_east_y: &fixture.map_west_east_y,
                    inverse_south_north_x: &fixture.map_inverse_south_north_x,
                },
                &fixture.coefficients[..4],
                &fixture.offsets,
                AcousticSubstepPhase::try_new(1, 3).unwrap(),
                &region,
            )
            .unwrap_err();

        assert_eq!(
            error,
            AcousticFluxAccumulationError::CoefficientLengthMismatch {
                coefficient: AcousticFluxCoefficient::ColumnMassMultiplier,
                expected: 5,
                actual: 4,
            }
        );
        assert_eq!(fixture.average_west_east.values(), before);
    }

    #[test]
    fn invalid_field_shape_names_the_scientific_role() {
        let backend = CpuBackend::try_with_worker_count(1).unwrap();
        let mut fixture = Fixture::new(&backend);
        fixture.current_vertical = backend
            .create_field(GridShape::try_new(4, 5, 5).unwrap(), 0.0)
            .unwrap();

        let error = fixture
            .apply(&backend, AcousticSubstepPhase::try_new(1, 3).unwrap())
            .unwrap_err();

        assert!(matches!(
            error,
            AcousticFluxAccumulationError::FieldShapeMismatch {
                field: AcousticFluxField::CurrentVertical,
                ..
            }
        ));
    }

    #[test]
    fn one_and_four_workers_produce_identical_complete_storage_bits() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();
        let mut serial = Fixture::new(&one_worker);
        let mut parallel = Fixture::new(&four_workers);

        for iteration in 1..=3 {
            let phase = AcousticSubstepPhase::try_new(iteration, 3).unwrap();
            serial.set_current_fluxes(iteration);
            parallel.set_current_fluxes(iteration);
            serial.apply(&one_worker, phase).unwrap();
            parallel.apply(&four_workers, phase).unwrap();
        }

        for (serial, parallel) in [
            (&serial.average_west_east, &parallel.average_west_east),
            (&serial.average_south_north, &parallel.average_south_north),
            (&serial.average_vertical, &parallel.average_vertical),
        ] {
            assert_eq!(
                serial
                    .values()
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                parallel
                    .values()
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>()
            );
        }
    }

    struct Fixture {
        current_west_east: CpuField<f32>,
        current_south_north: CpuField<f32>,
        current_vertical: CpuField<f32>,
        linear_west_east: CpuField<f32>,
        linear_south_north: CpuField<f32>,
        linear_vertical: CpuField<f32>,
        average_west_east: CpuField<f32>,
        average_south_north: CpuField<f32>,
        average_vertical: CpuField<f32>,
        mass_west_east: CpuField<f32>,
        mass_south_north: CpuField<f32>,
        map_west_east_y: CpuField<f32>,
        map_inverse_south_north_x: CpuField<f32>,
        coefficients: Vec<f32>,
        offsets: Vec<f32>,
    }

    impl Fixture {
        fn new(backend: &CpuBackend) -> Self {
            let shape = GridShape::try_new(5, 5, 5).unwrap();
            let volume = |value| backend.create_field(shape, value).unwrap();
            let horizontal = |value| {
                backend
                    .create_field(shape.horizontal_shape(), value)
                    .unwrap()
            };
            let mut fixture = Self {
                current_west_east: volume(0.0),
                current_south_north: volume(0.0),
                current_vertical: volume(0.0),
                linear_west_east: volume(0.0),
                linear_south_north: volume(0.0),
                linear_vertical: volume(0.0),
                average_west_east: volume(-999.0),
                average_south_north: volume(-999.0),
                average_vertical: volume(-999.0),
                mass_west_east: horizontal(0.0),
                mass_south_north: horizontal(0.0),
                map_west_east_y: horizontal(0.0),
                map_inverse_south_north_x: horizontal(0.0),
                coefficients: vec![0.0; 5],
                offsets: vec![0.0; 5],
            };
            fixture.initialize();
            fixture
        }

        fn initialize(&mut self) {
            for level in 0..5 {
                let level_value = level as f32;
                self.coefficients[level] = 0.45 + level_value * 0.013;
                self.offsets[level] = 0.17 - level_value * 0.006;
            }
            for south_north in 0..5 {
                let south_north_value = south_north as f32;
                for west_east in 0..5 {
                    let west_east_value = west_east as f32;
                    let horizontal = south_north * 5 + west_east;
                    self.mass_west_east.values_mut()[horizontal] =
                        10.0 + west_east_value * 0.2 - south_north_value * 0.1;
                    self.mass_south_north.values_mut()[horizontal] =
                        11.0 - west_east_value * 0.1 + south_north_value * 0.15;
                    self.map_west_east_y.values_mut()[horizontal] =
                        0.95 + west_east_value * 0.01 + south_north_value * 0.005;
                    self.map_inverse_south_north_x.values_mut()[horizontal] =
                        1.04 - west_east_value * 0.004 + south_north_value * 0.003;
                    for level in 0..5 {
                        let level_value = level as f32;
                        let index = volume_index(west_east, level, south_north);
                        self.linear_west_east.values_mut()[index] =
                            0.21 + west_east_value * 0.011 + level_value * 0.017
                                - south_north_value * 0.009;
                        self.linear_south_north.values_mut()[index] = 0.31
                            - west_east_value * 0.007
                            + level_value * 0.013
                            + south_north_value * 0.015;
                        self.linear_vertical.values_mut()[index] = 0.41 + west_east_value * 0.005
                            - level_value * 0.003
                            + south_north_value * 0.019;
                    }
                }
            }
        }

        fn set_current_fluxes(&mut self, iteration: usize) {
            let iteration_value = iteration as f32;
            for south_north in 0..5 {
                let south_north_value = south_north as f32;
                for level in 0..5 {
                    let level_value = level as f32;
                    for west_east in 0..5 {
                        let west_east_value = west_east as f32;
                        let index = volume_index(west_east, level, south_north);
                        self.current_west_east.values_mut()[index] =
                            iteration_value * 0.1 + west_east_value * 0.013 + level_value * 0.007
                                - south_north_value * 0.005;
                        self.current_south_north.values_mut()[index] = iteration_value * 0.2
                            - west_east_value * 0.009
                            + level_value * 0.011
                            + south_north_value * 0.004;
                        self.current_vertical.values_mut()[index] =
                            iteration_value * 0.3 + west_east_value * 0.003 - level_value * 0.008
                                + south_north_value * 0.006;
                    }
                }
            }
        }

        fn apply(
            &mut self,
            backend: &CpuBackend,
            phase: AcousticSubstepPhase,
        ) -> AcousticFluxAccumulationResult<()> {
            backend.accumulate_acoustic_fluxes(
                AcousticFluxRunningAverages {
                    west_east: &mut self.average_west_east,
                    south_north: &mut self.average_south_north,
                    vertical: &mut self.average_vertical,
                },
                AcousticFluxCurrentFields {
                    west_east: &self.current_west_east,
                    south_north: &self.current_south_north,
                    vertical: &self.current_vertical,
                },
                AcousticFluxLinearFields {
                    west_east: &self.linear_west_east,
                    south_north: &self.linear_south_north,
                    vertical: &self.linear_vertical,
                },
                AcousticFluxMassFields {
                    west_east: &self.mass_west_east,
                    south_north: &self.mass_south_north,
                },
                AcousticFluxMapFactors {
                    west_east_y: &self.map_west_east_y,
                    inverse_south_north_x: &self.map_inverse_south_north_x,
                },
                &self.coefficients,
                &self.offsets,
                phase,
                &Self::region(),
            )
        }

        fn region() -> AcousticFluxAccumulationRegion {
            AcousticFluxAccumulationRegion::try_new(
                GridShape::try_new(5, 5, 5).unwrap(),
                1..4,
                1..4,
                1..4,
                1..5,
                1..5,
                1..5,
            )
            .unwrap()
        }
    }

    fn oracle_bits() -> HashMap<&'static str, Vec<u32>> {
        let mut output = HashMap::new();
        for line in
            include_str!("../../../test-data/acoustic_flux_accumulation.out.correct").lines()
        {
            let mut parts = line.split_whitespace();
            let name = parts.next().unwrap();
            let _west_east = parts.next().unwrap();
            let _bottom_top = parts.next().unwrap();
            let _south_north = parts.next().unwrap();
            let bits = u32::from_str_radix(parts.next().unwrap(), 16).unwrap();
            output.entry(name).or_insert_with(Vec::new).push(bits);
        }
        output
    }

    fn assert_bits(name: &str, actual: &[f32], expected: &HashMap<&str, Vec<u32>>) {
        let actual = actual
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected[name], "{name} differs from direct Fortran");
    }

    const fn volume_index(west_east: usize, level: usize, south_north: usize) -> usize {
        west_east + 5 * (level + 5 * south_north)
    }
}
