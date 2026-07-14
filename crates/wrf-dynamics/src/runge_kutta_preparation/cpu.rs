use wrf_compute::{CpuBackend, CpuField};

use crate::column_mass_staggering::{validate_big_step_operation, validate_full_mass_operation};
use crate::inverse_density::validate_operation as validate_inverse_density;
use crate::moisture_coefficients::validate_borrowed_operation as validate_moisture_coefficients;
use crate::momentum_coupling::validate_borrowed_operation as validate_momentum_coupling;
use crate::omega_diagnosis::validate_operation as validate_omega_diagnosis;
use crate::pressure_point_geopotential::validate_operation as validate_pressure_point_geopotential;
use crate::{
    ColumnMassStaggeringKernels, ColumnMassStaggeringPeriodicity, InverseDensityKernels,
    MoistureCoefficientKernels, MoistureCoefficientOutputs, MomentumCouplingKernels,
    MomentumCouplingMasses, MomentumCouplingOutputs, OmegaDiagnosisKernels, OmegaDiagnosisMasses,
    PressurePointGeopotentialKernels, RungeKuttaPreparationInputs, RungeKuttaPreparationKernels,
    RungeKuttaPreparationOutputs, RungeKuttaPreparationRegions, RungeKuttaPreparationResult,
    RungeKuttaPreparationStage,
};

impl RungeKuttaPreparationKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn prepare_runge_kutta_step(
        &self,
        outputs: RungeKuttaPreparationOutputs<'_, Self::Field>,
        inputs: RungeKuttaPreparationInputs<'_, Self::Field>,
        regions: &RungeKuttaPreparationRegions,
        periodicity: ColumnMassStaggeringPeriodicity,
    ) -> RungeKuttaPreparationResult<()> {
        let RungeKuttaPreparationOutputs {
            masses,
            momentum,
            diagnostics,
        } = outputs;

        validate_all(
            &masses,
            &momentum,
            &diagnostics,
            &inputs,
            regions,
            periodicity,
        )?;

        self.calculate_full_column_mass(
            masses.full,
            inputs.masses.perturbation,
            inputs.masses.base_state,
            &regions.column_mass,
        )?;
        self.stagger_column_mass_for_big_step(
            inputs.masses.perturbation,
            inputs.masses.base_state,
            masses.west_east_momentum,
            masses.south_north_momentum,
            &regions.column_mass,
            periodicity,
        )?;
        self.couple_momentum(
            MomentumCouplingOutputs::new(
                momentum.west_east,
                momentum.south_north,
                momentum.vertical,
            ),
            inputs.velocities.momentum(),
            MomentumCouplingMasses::new(
                masses.west_east_momentum,
                masses.south_north_momentum,
                masses.full,
            ),
            inputs.map_factors.momentum(),
            inputs.coefficients.momentum(),
            &regions.momentum,
        )?;
        self.diagnose_omega(
            diagnostics.omega,
            inputs.velocities.omega(),
            OmegaDiagnosisMasses::new(inputs.masses.perturbation, inputs.masses.base_state),
            inputs.map_factors.omega(),
            inputs.coefficients.omega(),
            inputs.grid_metrics,
            &regions.omega,
        )?;
        self.calculate_moisture_coefficients(
            MoistureCoefficientOutputs::new(
                diagnostics.west_east_moisture,
                diagnostics.south_north_moisture,
                diagnostics.vertical_moisture,
            ),
            inputs.moisture,
            &regions.moisture,
        )?;
        self.calculate_full_inverse_density(
            diagnostics.full_inverse_density,
            inputs.thermodynamics.perturbation_inverse_density,
            inputs.thermodynamics.base_state_inverse_density,
            &regions.inverse_density,
        )?;
        self.calculate_pressure_point_geopotential(
            diagnostics.pressure_point_geopotential,
            inputs.thermodynamics.perturbation_full_level_geopotential,
            inputs.thermodynamics.base_state_full_level_geopotential,
            &regions.pressure_point_geopotential,
        )?;
        Ok(())
    }
}

fn validate_all(
    masses: &super::RungeKuttaPreparationMassOutputs<'_, CpuField<f32>>,
    momentum: &super::RungeKuttaPreparationMomentumOutputs<'_, CpuField<f32>>,
    diagnostics: &super::RungeKuttaPreparationDiagnosticOutputs<'_, CpuField<f32>>,
    inputs: &RungeKuttaPreparationInputs<'_, CpuField<f32>>,
    regions: &RungeKuttaPreparationRegions,
    periodicity: ColumnMassStaggeringPeriodicity,
) -> RungeKuttaPreparationResult<()> {
    validate_region_shapes(regions)?;
    validate_full_mass_operation(
        masses.full,
        inputs.masses.perturbation,
        inputs.masses.base_state,
        &regions.column_mass,
    )?;
    validate_big_step_operation(
        inputs.masses.perturbation,
        inputs.masses.base_state,
        masses.west_east_momentum,
        masses.south_north_momentum,
        &regions.column_mass,
        periodicity,
    )?;
    validate_momentum_coupling(
        momentum.west_east,
        momentum.south_north,
        momentum.vertical,
        &inputs.velocities.momentum(),
        &MomentumCouplingMasses::new(
            masses.west_east_momentum,
            masses.south_north_momentum,
            masses.full,
        ),
        &inputs.map_factors.momentum(),
        inputs.coefficients.momentum(),
        &regions.momentum,
    )?;
    validate_omega_diagnosis(
        diagnostics.omega,
        &inputs.velocities.omega(),
        &OmegaDiagnosisMasses::new(inputs.masses.perturbation, inputs.masses.base_state),
        &inputs.map_factors.omega(),
        inputs.coefficients.omega(),
        &regions.omega,
    )?;
    validate_moisture_coefficients(
        diagnostics.west_east_moisture,
        diagnostics.south_north_moisture,
        diagnostics.vertical_moisture,
        inputs.moisture,
        &regions.moisture,
    )?;
    validate_inverse_density(
        diagnostics.full_inverse_density,
        inputs.thermodynamics.perturbation_inverse_density,
        inputs.thermodynamics.base_state_inverse_density,
        &regions.inverse_density,
    )?;
    validate_pressure_point_geopotential(
        diagnostics.pressure_point_geopotential,
        inputs.thermodynamics.perturbation_full_level_geopotential,
        inputs.thermodynamics.base_state_full_level_geopotential,
        &regions.pressure_point_geopotential,
    )?;
    Ok(())
}

fn validate_region_shapes(
    regions: &RungeKuttaPreparationRegions,
) -> RungeKuttaPreparationResult<()> {
    let expected = regions.momentum.shape();
    for (stage, actual) in [
        (
            RungeKuttaPreparationStage::ColumnMass,
            regions.column_mass.shape(),
        ),
        (
            RungeKuttaPreparationStage::OmegaDiagnosis,
            regions.omega.shape(),
        ),
        (
            RungeKuttaPreparationStage::MoistureCoefficients,
            regions.moisture.shape(),
        ),
        (
            RungeKuttaPreparationStage::InverseDensity,
            regions.inverse_density.shape(),
        ),
        (
            RungeKuttaPreparationStage::PressurePointGeopotential,
            regions.pressure_point_geopotential.shape(),
        ),
    ] {
        let expected_for_stage = if stage == RungeKuttaPreparationStage::ColumnMass {
            expected.horizontal_shape()
        } else {
            expected
        };
        if actual != expected_for_stage {
            return Err(crate::RungeKuttaPreparationError::RegionShapeMismatch {
                stage,
                expected: expected_for_stage,
                actual,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use wrf_compute::{ComputeBackend, CpuField, GridShape};

    use super::*;
    use crate::{
        ColumnMassStaggeringRegion, InverseDensityRegion, MoistureCoefficientRegion,
        MomentumCouplingRegion, OmegaDiagnosisGridMetrics, OmegaDiagnosisRegion,
        PressurePointGeopotentialRegion, RungeKuttaPreparationCoefficients,
        RungeKuttaPreparationDiagnosticOutputs, RungeKuttaPreparationMapFactors,
        RungeKuttaPreparationMassInputs, RungeKuttaPreparationMassOutputs,
        RungeKuttaPreparationMomentumOutputs, RungeKuttaPreparationThermodynamicInputs,
        RungeKuttaPreparationVelocities,
    };

    const SENTINEL: f32 = -9_999.0;

    struct Fixture {
        perturbation_mass: CpuField<f32>,
        base_mass: CpuField<f32>,
        west_east_velocity: CpuField<f32>,
        south_north_velocity: CpuField<f32>,
        vertical_velocity: CpuField<f32>,
        mass_point_west_east_map_factor: CpuField<f32>,
        mass_point_south_north_map_factor: CpuField<f32>,
        west_east_momentum_south_north_map_factor: CpuField<f32>,
        inverse_south_north_momentum_west_east_map_factor: CpuField<f32>,
        moisture: Vec<CpuField<f32>>,
        perturbation_inverse_density: CpuField<f32>,
        base_state_inverse_density: CpuField<f32>,
        perturbation_full_level_geopotential: CpuField<f32>,
        base_state_full_level_geopotential: CpuField<f32>,
        full_mass: CpuField<f32>,
        west_east_momentum_mass: CpuField<f32>,
        south_north_momentum_mass: CpuField<f32>,
        west_east_momentum: CpuField<f32>,
        south_north_momentum: CpuField<f32>,
        vertical_momentum: CpuField<f32>,
        omega: CpuField<f32>,
        west_east_moisture: CpuField<f32>,
        south_north_moisture: CpuField<f32>,
        vertical_moisture: CpuField<f32>,
        full_inverse_density: CpuField<f32>,
        pressure_point_geopotential: CpuField<f32>,
    }

    #[test]
    fn is_bitwise_deterministic_across_worker_counts() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();
        let mut one_worker_fixture = create_fixture(&one_worker);
        let mut four_worker_fixture = create_fixture(&four_workers);
        let coefficients = valid_coefficients();

        apply(&one_worker, &mut one_worker_fixture, &coefficients).unwrap();
        apply(&four_workers, &mut four_worker_fixture, &coefficients).unwrap();

        for (one, four) in output_fields(&one_worker_fixture)
            .into_iter()
            .zip(output_fields(&four_worker_fixture))
        {
            assert_eq!(one.values(), four.values());
        }
    }

    #[test]
    fn validates_the_final_stage_before_any_output_changes() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut fixture = create_fixture(&backend);
        fixture.pressure_point_geopotential = backend
            .create_field(GridShape::try_new(5, 6, 5).unwrap(), SENTINEL)
            .unwrap();
        let coefficients = valid_coefficients();

        let error = apply(&backend, &mut fixture, &coefficients).unwrap_err();

        assert!(matches!(
            error,
            crate::RungeKuttaPreparationError::PressurePointGeopotential(
                crate::PressurePointGeopotentialError::FieldShapeMismatch { .. }
            )
        ));
        for output in output_fields(&fixture) {
            assert!(output.values().iter().all(|value| *value == SENTINEL));
        }
    }

    #[test]
    fn rejects_inconsistent_component_grid_shapes() {
        let mut regions = regions();
        let inconsistent_shape = GridShape::try_new(7, 6, 5).unwrap();
        regions.inverse_density =
            InverseDensityRegion::try_new(inconsistent_shape, 1..6, 1..5, 1..4, 2..6, 2..5, 1..4)
                .unwrap();

        assert_eq!(
            validate_region_shapes(&regions),
            Err(crate::RungeKuttaPreparationError::RegionShapeMismatch {
                stage: RungeKuttaPreparationStage::InverseDensity,
                expected: GridShape::try_new(6, 6, 5).unwrap(),
                actual: inconsistent_shape,
            })
        );
    }

    #[test]
    fn matches_coupled_upstream_fortran_for_every_output_and_sentinel() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let mut fixture = create_oracle_fixture(&backend);
        let coefficient_values = OracleCoefficientValues::new();
        let coefficients = coefficient_values.borrowed();
        let mut expected = crate::test_support::ExpectedOutputReader::new(include_str!(
            "../../test-data/runge_kutta_preparation.out.correct"
        ));

        apply(&backend, &mut fixture, &coefficients).unwrap();

        for (field_name, field) in named_output_fields(&fixture) {
            for (index, value) in field.values().iter().copied().enumerate() {
                expected.assert_next(0, field_name, index, value);
            }
        }
        expected.finish();
    }

    fn create_fixture(backend: &CpuBackend) -> Fixture {
        let shape = GridShape::try_new(6, 6, 5).unwrap();
        let horizontal_shape = shape.horizontal_shape();
        let patterned_volume = |offset: f32| {
            let mut field = backend.create_field(shape, 0.0_f32).unwrap();
            for (index, value) in field.values_mut().iter_mut().enumerate() {
                *value = offset + index as f32 * 0.000_1;
            }
            field
        };
        let patterned_horizontal = |offset: f32| {
            let mut field = backend.create_field(horizontal_shape, 0.0_f32).unwrap();
            for (index, value) in field.values_mut().iter_mut().enumerate() {
                *value = offset + index as f32 * 0.01;
            }
            field
        };
        let output_volume = || backend.create_field(shape, SENTINEL).unwrap();
        let output_horizontal = || backend.create_field(horizontal_shape, SENTINEL).unwrap();

        Fixture {
            perturbation_mass: patterned_horizontal(10.0),
            base_mass: patterned_horizontal(90.0),
            west_east_velocity: patterned_volume(1.0),
            south_north_velocity: patterned_volume(-0.5),
            vertical_velocity: patterned_volume(0.25),
            mass_point_west_east_map_factor: patterned_horizontal(1.0),
            mass_point_south_north_map_factor: patterned_horizontal(1.1),
            west_east_momentum_south_north_map_factor: patterned_horizontal(0.9),
            inverse_south_north_momentum_west_east_map_factor: patterned_horizontal(0.8),
            moisture: vec![patterned_volume(0.001), patterned_volume(0.002)],
            perturbation_inverse_density: patterned_volume(0.2),
            base_state_inverse_density: patterned_volume(0.8),
            perturbation_full_level_geopotential: patterned_volume(100.0),
            base_state_full_level_geopotential: patterned_volume(1_000.0),
            full_mass: output_horizontal(),
            west_east_momentum_mass: output_horizontal(),
            south_north_momentum_mass: output_horizontal(),
            west_east_momentum: output_volume(),
            south_north_momentum: output_volume(),
            vertical_momentum: output_volume(),
            omega: output_volume(),
            west_east_moisture: output_volume(),
            south_north_moisture: output_volume(),
            vertical_moisture: output_volume(),
            full_inverse_density: output_volume(),
            pressure_point_geopotential: output_volume(),
        }
    }

    fn create_oracle_fixture(backend: &CpuBackend) -> Fixture {
        let shape = GridShape::try_new(6, 6, 5).unwrap();
        let horizontal_shape = shape.horizontal_shape();
        let mut perturbation_mass = backend.create_field(horizontal_shape, 0.0).unwrap();
        let mut base_mass = backend.create_field(horizontal_shape, 0.0).unwrap();
        let mut mass_point_west_east_map_factor =
            backend.create_field(horizontal_shape, 0.0).unwrap();
        let mut mass_point_south_north_map_factor =
            backend.create_field(horizontal_shape, 0.0).unwrap();
        let mut west_east_momentum_south_north_map_factor =
            backend.create_field(horizontal_shape, 0.0).unwrap();
        let mut inverse_south_north_momentum_west_east_map_factor =
            backend.create_field(horizontal_shape, 0.0).unwrap();

        for south_north in 0..shape.south_north_points() {
            let south_north = south_north as f32;
            for west_east in 0..shape.west_east_points() {
                let index = south_north as usize * shape.west_east_points() + west_east;
                let west_east = west_east as f32;
                perturbation_mass.values_mut()[index] =
                    (10.0 + west_east * 0.25) + south_north * 0.5;
                base_mass.values_mut()[index] = (90.0 - west_east * 0.125) + south_north * 0.375;
                mass_point_west_east_map_factor.values_mut()[index] =
                    (1.0 + west_east * 0.002) + south_north * 0.003;
                mass_point_south_north_map_factor.values_mut()[index] =
                    (1.1 - west_east * 0.001) + south_north * 0.002;
                west_east_momentum_south_north_map_factor.values_mut()[index] =
                    (0.9 + west_east * 0.0015) - south_north * 0.0005;
                inverse_south_north_momentum_west_east_map_factor.values_mut()[index] =
                    (0.8 - west_east * 0.001) + south_north * 0.001;
            }
        }

        let mut west_east_velocity = backend.create_field(shape, 0.0).unwrap();
        let mut south_north_velocity = backend.create_field(shape, 0.0).unwrap();
        let mut vertical_velocity = backend.create_field(shape, 0.0).unwrap();
        let mut perturbation_inverse_density = backend.create_field(shape, 0.0).unwrap();
        let mut base_state_inverse_density = backend.create_field(shape, 0.0).unwrap();
        let mut perturbation_full_level_geopotential = backend.create_field(shape, 0.0).unwrap();
        let mut base_state_full_level_geopotential = backend.create_field(shape, 0.0).unwrap();
        let mut moisture = vec![
            backend.create_field(shape, 0.0).unwrap(),
            backend.create_field(shape, 0.0).unwrap(),
        ];
        for south_north in 0..shape.south_north_points() {
            let south_north_value = south_north as f32;
            for bottom_top in 0..shape.bottom_top_points() {
                let bottom_top_value = bottom_top as f32;
                for west_east in 0..shape.west_east_points() {
                    let west_east_value = west_east as f32;
                    let index = (south_north * shape.bottom_top_points() + bottom_top)
                        * shape.west_east_points()
                        + west_east;
                    west_east_velocity.values_mut()[index] = ((1.0 + west_east_value * 0.01)
                        + bottom_top_value * 0.02)
                        - south_north_value * 0.03;
                    south_north_velocity.values_mut()[index] = ((-0.5 + west_east_value * 0.015)
                        - bottom_top_value * 0.01)
                        + south_north_value * 0.025;
                    vertical_velocity.values_mut()[index] = ((0.25 - west_east_value * 0.005)
                        + bottom_top_value * 0.03)
                        + south_north_value * 0.004;
                    perturbation_inverse_density.values_mut()[index] =
                        ((0.2 + west_east_value * 0.001) - bottom_top_value * 0.002)
                            + south_north_value * 0.0005;
                    base_state_inverse_density.values_mut()[index] =
                        ((0.8 - west_east_value * 0.0005) + bottom_top_value * 0.001)
                            - south_north_value * 0.00025;
                    perturbation_full_level_geopotential.values_mut()[index] =
                        ((100.0 + west_east_value * 2.0) + bottom_top_value * 11.0)
                            - south_north_value * 3.0;
                    base_state_full_level_geopotential.values_mut()[index] =
                        ((1000.0 - west_east_value * 1.5) + bottom_top_value * 17.0)
                            + south_north_value * 2.5;
                    moisture[0].values_mut()[index] = ((0.001 + west_east_value * 0.00001)
                        + bottom_top_value * 0.00002)
                        + south_north_value * 0.00003;
                    moisture[1].values_mut()[index] = ((0.002 - west_east_value * 0.000005)
                        + bottom_top_value * 0.00001)
                        - south_north_value * 0.000004;
                }
            }
        }
        let output_volume = || backend.create_field(shape, SENTINEL).unwrap();
        let output_horizontal = || backend.create_field(horizontal_shape, SENTINEL).unwrap();

        Fixture {
            perturbation_mass,
            base_mass,
            west_east_velocity,
            south_north_velocity,
            vertical_velocity,
            mass_point_west_east_map_factor,
            mass_point_south_north_map_factor,
            west_east_momentum_south_north_map_factor,
            inverse_south_north_momentum_west_east_map_factor,
            moisture,
            perturbation_inverse_density,
            base_state_inverse_density,
            perturbation_full_level_geopotential,
            base_state_full_level_geopotential,
            full_mass: output_horizontal(),
            west_east_momentum_mass: output_horizontal(),
            south_north_momentum_mass: output_horizontal(),
            west_east_momentum: output_volume(),
            south_north_momentum: output_volume(),
            vertical_momentum: output_volume(),
            omega: output_volume(),
            west_east_moisture: output_volume(),
            south_north_moisture: output_volume(),
            vertical_moisture: output_volume(),
            full_inverse_density: output_volume(),
            pressure_point_geopotential: output_volume(),
        }
    }

    struct OracleCoefficientValues {
        half_level_mass_multiplier: [f32; 5],
        half_level_mass_offset: [f32; 5],
        full_level_mass_multiplier: [f32; 5],
        full_level_mass_offset: [f32; 5],
        eta_layer_thickness: [f32; 5],
    }

    impl OracleCoefficientValues {
        fn new() -> Self {
            let mut values = Self {
                half_level_mass_multiplier: [0.0; 5],
                half_level_mass_offset: [0.0; 5],
                full_level_mass_multiplier: [0.0; 5],
                full_level_mass_offset: [0.0; 5],
                eta_layer_thickness: [0.0; 5],
            };
            for index in 0..5 {
                let index_value = index as f32;
                values.half_level_mass_multiplier[index] = 1.0 + index_value * 0.01;
                values.half_level_mass_offset[index] = 0.1 - index_value * 0.005;
                values.full_level_mass_multiplier[index] = 0.9 + index_value * 0.015;
                values.full_level_mass_offset[index] = 0.2 + index_value * 0.004;
                values.eta_layer_thickness[index] = -0.25 + index_value * 0.002;
            }
            values
        }

        fn borrowed(&self) -> RungeKuttaPreparationCoefficients<'_> {
            RungeKuttaPreparationCoefficients::new(
                &self.half_level_mass_multiplier,
                &self.half_level_mass_offset,
                &self.full_level_mass_multiplier,
                &self.full_level_mass_offset,
                &self.eta_layer_thickness,
            )
        }
    }

    fn valid_coefficients() -> RungeKuttaPreparationCoefficients<'static> {
        RungeKuttaPreparationCoefficients::new(
            &[1.0; 5],
            &[0.1; 5],
            &[0.9; 5],
            &[0.2; 5],
            &[-0.25; 5],
        )
    }

    fn regions() -> RungeKuttaPreparationRegions {
        let shape = GridShape::try_new(6, 6, 5).unwrap();
        RungeKuttaPreparationRegions::new(
            ColumnMassStaggeringRegion::try_new(shape.horizontal_shape(), 1..5, 1..5, 2..5, 2..5)
                .unwrap(),
            MomentumCouplingRegion::try_new(shape, 1..5, 1..5, 1..4, 2..5, 2..5, 1..5).unwrap(),
            OmegaDiagnosisRegion::try_new(shape, 1..5, 1..5, 1..4, 2..5, 2..5, 1..5).unwrap(),
            MoistureCoefficientRegion::try_new(shape, 1..5, 1..5, 1..4, 2..5, 2..5, 1..5).unwrap(),
            InverseDensityRegion::try_new(shape, 1..5, 1..5, 1..4, 2..5, 2..5, 1..4).unwrap(),
            PressurePointGeopotentialRegion::try_new(shape, 1..5, 1..5, 1..4, 2..5, 2..5, 1..4)
                .unwrap(),
        )
    }

    fn apply(
        backend: &CpuBackend,
        fixture: &mut Fixture,
        coefficients: &RungeKuttaPreparationCoefficients<'_>,
    ) -> RungeKuttaPreparationResult<()> {
        let inputs = RungeKuttaPreparationInputs::new(
            RungeKuttaPreparationMassInputs::new(&fixture.perturbation_mass, &fixture.base_mass),
            RungeKuttaPreparationVelocities::new(
                &fixture.west_east_velocity,
                &fixture.south_north_velocity,
                &fixture.vertical_velocity,
            ),
            RungeKuttaPreparationMapFactors::new(
                &fixture.mass_point_west_east_map_factor,
                &fixture.mass_point_south_north_map_factor,
                &fixture.west_east_momentum_south_north_map_factor,
                &fixture.inverse_south_north_momentum_west_east_map_factor,
            ),
            *coefficients,
            crate::MoistureSpecies::new(&fixture.moisture),
            RungeKuttaPreparationThermodynamicInputs::new(
                &fixture.perturbation_inverse_density,
                &fixture.base_state_inverse_density,
                &fixture.perturbation_full_level_geopotential,
                &fixture.base_state_full_level_geopotential,
            ),
            OmegaDiagnosisGridMetrics::new(1.0, 1.0),
        );
        let outputs = RungeKuttaPreparationOutputs::new(
            RungeKuttaPreparationMassOutputs::new(
                &mut fixture.full_mass,
                &mut fixture.west_east_momentum_mass,
                &mut fixture.south_north_momentum_mass,
            ),
            RungeKuttaPreparationMomentumOutputs::new(
                &mut fixture.west_east_momentum,
                &mut fixture.south_north_momentum,
                &mut fixture.vertical_momentum,
            ),
            RungeKuttaPreparationDiagnosticOutputs::new(
                &mut fixture.omega,
                &mut fixture.west_east_moisture,
                &mut fixture.south_north_moisture,
                &mut fixture.vertical_moisture,
                &mut fixture.full_inverse_density,
                &mut fixture.pressure_point_geopotential,
            ),
        );

        backend.prepare_runge_kutta_step(
            outputs,
            inputs,
            &regions(),
            ColumnMassStaggeringPeriodicity::None,
        )
    }

    fn output_fields(fixture: &Fixture) -> [&CpuField<f32>; 12] {
        [
            &fixture.full_mass,
            &fixture.west_east_momentum_mass,
            &fixture.south_north_momentum_mass,
            &fixture.west_east_momentum,
            &fixture.south_north_momentum,
            &fixture.vertical_momentum,
            &fixture.omega,
            &fixture.west_east_moisture,
            &fixture.south_north_moisture,
            &fixture.vertical_moisture,
            &fixture.full_inverse_density,
            &fixture.pressure_point_geopotential,
        ]
    }

    fn named_output_fields(fixture: &Fixture) -> [(&'static str, &CpuField<f32>); 12] {
        [
            ("mut", &fixture.full_mass),
            ("muu", &fixture.west_east_momentum_mass),
            ("muv", &fixture.south_north_momentum_mass),
            ("ru", &fixture.west_east_momentum),
            ("rv", &fixture.south_north_momentum),
            ("rw", &fixture.vertical_momentum),
            ("ww", &fixture.omega),
            ("cqu", &fixture.west_east_moisture),
            ("cqv", &fixture.south_north_moisture),
            ("cqw", &fixture.vertical_moisture),
            ("alt", &fixture.full_inverse_density),
            ("php", &fixture.pressure_point_geopotential),
        ]
    }
}
