use wrf_compute::{CpuBackend, CpuField};

use crate::{
    AcousticFluxAccumulationKernels, AcousticFluxCurrentFields, AcousticFluxLinearFields,
    AcousticFluxMapFactors, AcousticFluxMassFields, AcousticFluxRunningAverages,
    AcousticHorizontalMapFactors, AcousticHorizontalMassFields,
    AcousticHorizontalMoistureCoefficients, AcousticHorizontalMomentumInputs,
    AcousticHorizontalMomentumKernels, AcousticHorizontalMomentumParameters,
    AcousticHorizontalMomentumState, AcousticHorizontalMomentumTendencies,
    AcousticHorizontalPressureFields, AcousticHorizontalVerticalCoefficients,
    AcousticMassThetaDiagnostics, AcousticMassThetaInputs, AcousticMassThetaKernels,
    AcousticMassThetaMapFactors, AcousticMassThetaMassInputs, AcousticMassThetaMomentumInputs,
    AcousticMassThetaParameters, AcousticMassThetaState, AcousticMassThetaThermodynamicInputs,
    AcousticMassThetaVerticalCoefficients, AcousticPressureCoefficients,
    AcousticPressureDampingPhase, AcousticPressureKernels, AcousticPressureMasses,
    AcousticPressureParameters, AcousticPressureState, AcousticPressureThermodynamics,
    AcousticPressureVerticalMetrics, AcousticStepPreparationCoefficients,
    AcousticStepPreparationColumnMassTimeLevels, AcousticStepPreparationDiagnosticInputs,
    AcousticStepPreparationKernels, AcousticStepPreparationMapFactors,
    AcousticStepPreparationMassInputs, AcousticStepPreparationMassOutputs,
    AcousticStepPreparationSavedOutputs, AcousticStepPreparationVolumeTimeLevels,
    AcousticSubstepPhase, AcousticTrajectoryCoefficients, AcousticTrajectoryControls,
    AcousticTrajectoryDiagnostics, AcousticTrajectoryInputs, AcousticTrajectoryRegions,
    AcousticTrajectoryResult, AcousticTrajectorySavedState, AcousticTrajectoryTimeLevels,
    AcousticTrajectoryWorkspace, AcousticVerticalGeopotentialInputs, AcousticVerticalInputs,
    AcousticVerticalKernels, AcousticVerticalLevelCoefficients, AcousticVerticalMapFactors,
    AcousticVerticalMassInputs, AcousticVerticalMomentumInputs, AcousticVerticalParameters,
    AcousticVerticalSolveInputs, AcousticVerticalState, AcousticVerticalThermodynamicInputs,
    AcousticVerticalWorkspace, VerticalAcousticCoefficientInputs,
    VerticalAcousticCoefficientKernels, VerticalAcousticCoefficientParameters,
    VerticalAcousticMassCoefficients, VerticalAcousticMetrics, VerticalAcousticSolveCoefficients,
};

use super::arguments::{
    FluxAccumulationArguments, HorizontalMomentumArguments, MassThetaArguments,
    PreparationArguments, PressureArguments, VerticalCoefficientArguments,
    VerticalMomentumArguments,
};

pub(crate) struct AcousticTrajectoryExecution<'backend, 'fields, 'coefficients, 'regions> {
    pub(crate) backend: &'backend CpuBackend,
    pub(crate) time_levels: AcousticTrajectoryTimeLevels<'fields, CpuField<f32>>,
    pub(crate) saved_state: AcousticTrajectorySavedState<'fields, CpuField<f32>>,
    pub(crate) diagnostics: AcousticTrajectoryDiagnostics<'fields, CpuField<f32>>,
    pub(crate) workspace: AcousticTrajectoryWorkspace<'fields, CpuField<f32>>,
    pub(crate) inputs: AcousticTrajectoryInputs<'fields, CpuField<f32>>,
    pub(crate) coefficients: AcousticTrajectoryCoefficients<'coefficients>,
    pub(crate) controls: AcousticTrajectoryControls,
    pub(crate) regions: AcousticTrajectoryRegions<'regions>,
}

impl<'backend, 'fields, 'coefficients, 'regions>
    AcousticTrajectoryExecution<'backend, 'fields, 'coefficients, 'regions>
{
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        backend: &'backend CpuBackend,
        time_levels: AcousticTrajectoryTimeLevels<'fields, CpuField<f32>>,
        saved_state: AcousticTrajectorySavedState<'fields, CpuField<f32>>,
        diagnostics: AcousticTrajectoryDiagnostics<'fields, CpuField<f32>>,
        workspace: AcousticTrajectoryWorkspace<'fields, CpuField<f32>>,
        inputs: AcousticTrajectoryInputs<'fields, CpuField<f32>>,
        coefficients: AcousticTrajectoryCoefficients<'coefficients>,
        controls: AcousticTrajectoryControls,
        regions: AcousticTrajectoryRegions<'regions>,
    ) -> Self {
        Self {
            backend,
            time_levels,
            saved_state,
            diagnostics,
            workspace,
            inputs,
            coefficients,
            controls,
            regions,
        }
    }

    pub(crate) fn run(&mut self) -> AcousticTrajectoryResult<()> {
        self.prepare()?;
        self.update_pressure(AcousticPressureDampingPhase::Initialize)?;
        self.calculate_vertical_coefficients()?;
        for iteration in 1..=self.controls.substep_count {
            self.advance_horizontal_momentum()?;
            self.advance_mass_and_theta()?;
            self.advance_vertical_momentum()?;
            self.accumulate_fluxes(iteration)?;
            self.update_pressure(AcousticPressureDampingPhase::Advance)?;
        }
        Ok(())
    }

    pub(crate) fn prepare(&mut self) -> AcousticTrajectoryResult<()> {
        let backend = self.backend;
        let arguments = self.preparation_arguments();
        backend.prepare_acoustic_step(
            arguments.volume,
            arguments.column_mass,
            arguments.saved,
            arguments.mass_outputs,
            arguments.mass_inputs,
            arguments.diagnostics,
            arguments.maps,
            arguments.coefficients,
            arguments.phase,
            arguments.region,
        )?;
        Ok(())
    }

    pub(crate) fn update_pressure(
        &mut self,
        damping_phase: AcousticPressureDampingPhase,
    ) -> AcousticTrajectoryResult<()> {
        let backend = self.backend;
        let arguments = self.pressure_arguments(damping_phase);
        backend.update_acoustic_pressure(
            arguments.state,
            arguments.thermodynamics,
            arguments.masses,
            arguments.coefficients,
            arguments.vertical_metrics,
            arguments.parameters,
            arguments.mode,
            arguments.damping_phase,
            arguments.region,
        )?;
        Ok(())
    }

    pub(crate) fn calculate_vertical_coefficients(&mut self) -> AcousticTrajectoryResult<()> {
        let backend = self.backend;
        let arguments = self.vertical_coefficient_arguments();
        backend.calculate_vertical_acoustic_coefficients(
            arguments.outputs,
            arguments.inputs,
            arguments.mass_coefficients,
            arguments.metrics,
            arguments.parameters,
            arguments.region,
        )?;
        Ok(())
    }

    pub(crate) fn advance_horizontal_momentum(&mut self) -> AcousticTrajectoryResult<()> {
        let backend = self.backend;
        let arguments = self.horizontal_momentum_arguments();
        backend.advance_acoustic_horizontal_momentum(
            arguments.state,
            arguments.inputs,
            arguments.parameters,
            arguments.boundary_policy,
            arguments.region,
        )?;
        Ok(())
    }

    pub(crate) fn advance_mass_and_theta(&mut self) -> AcousticTrajectoryResult<()> {
        let backend = self.backend;
        let arguments = self.mass_theta_arguments();
        backend.advance_acoustic_mass_and_theta(
            arguments.state,
            arguments.diagnostics,
            arguments.inputs,
            arguments.parameters,
            arguments.boundary_policy,
            arguments.region,
        )?;
        Ok(())
    }

    pub(crate) fn advance_vertical_momentum(&mut self) -> AcousticTrajectoryResult<()> {
        let backend = self.backend;
        let arguments = self.vertical_momentum_arguments();
        backend.advance_acoustic_vertical_momentum(
            arguments.state,
            arguments.workspace,
            arguments.inputs,
            arguments.parameters,
            arguments.boundary_policy,
            arguments.top_boundary,
            arguments.vertical_advection,
            arguments.damping,
            arguments.region,
        )?;
        Ok(())
    }

    pub(crate) fn accumulate_fluxes(&mut self, iteration: usize) -> AcousticTrajectoryResult<()> {
        let backend = self.backend;
        let arguments = self.flux_accumulation_arguments(iteration)?;
        backend.accumulate_acoustic_fluxes(
            arguments.averages,
            arguments.current,
            arguments.linear,
            arguments.masses,
            arguments.map_factors,
            arguments.column_mass_multiplier,
            arguments.column_mass_offset,
            arguments.phase,
            arguments.region,
        )?;
        Ok(())
    }

    pub(super) fn preparation_arguments(
        &mut self,
    ) -> PreparationArguments<'_, 'coefficients, 'regions> {
        PreparationArguments {
            volume: AcousticStepPreparationVolumeTimeLevels::new(
                &mut *self.time_levels.previous_west_east_momentum,
                &mut *self.time_levels.current_west_east_momentum,
                &mut *self.time_levels.previous_south_north_momentum,
                &mut *self.time_levels.current_south_north_momentum,
                &mut *self.time_levels.previous_vertical_momentum,
                &mut *self.time_levels.current_vertical_momentum,
                &mut *self.time_levels.previous_potential_temperature,
                &mut *self.time_levels.current_potential_temperature,
                &mut *self.time_levels.previous_perturbation_geopotential,
                &mut *self.time_levels.current_perturbation_geopotential,
            ),
            column_mass: AcousticStepPreparationColumnMassTimeLevels::new(
                &mut *self.time_levels.previous_perturbation_column_mass,
                &mut *self.time_levels.current_perturbation_column_mass,
            ),
            saved: AcousticStepPreparationSavedOutputs::new(
                &mut *self.saved_state.west_east_momentum,
                &mut *self.saved_state.south_north_momentum,
                &mut *self.saved_state.vertical_momentum,
                &mut *self.saved_state.potential_temperature,
                &mut *self.saved_state.perturbation_geopotential,
                &mut *self.saved_state.perturbation_column_mass,
                &mut *self.saved_state.vertical_mass_flux,
                &mut *self.saved_state.pressure_coefficient,
            ),
            mass_outputs: AcousticStepPreparationMassOutputs::new(
                &mut *self.diagnostics.saved_west_east_column_mass,
                &mut *self.diagnostics.saved_south_north_column_mass,
                &mut *self.diagnostics.coupled_column_mass,
                &mut *self.diagnostics.divergence_damping_column_mass,
            ),
            mass_inputs: AcousticStepPreparationMassInputs::new(
                self.inputs.masses.base,
                self.inputs.masses.west_east,
                self.inputs.masses.south_north,
                self.inputs.masses.full,
            ),
            diagnostics: AcousticStepPreparationDiagnosticInputs::new(
                self.inputs.pressure.base_pressure,
                &*self.diagnostics.pressure_perturbation,
                self.inputs.pressure.full_inverse_density,
                &*self.diagnostics.vertical_mass_flux,
            ),
            maps: AcousticStepPreparationMapFactors::new(
                self.inputs.map_factors.west_east_y,
                self.inputs.map_factors.inverse_south_north_x,
                self.inputs.map_factors.mass_point_y,
            ),
            coefficients: AcousticStepPreparationCoefficients::new(
                self.coefficients.half_level_mass_multiplier,
                self.coefficients.half_level_mass_offset,
                self.coefficients.full_level_mass_multiplier,
                self.coefficients.full_level_mass_offset,
            ),
            phase: self.controls.preparation_phase,
            region: self.regions.preparation,
        }
    }

    pub(super) fn pressure_arguments(
        &mut self,
        damping_phase: AcousticPressureDampingPhase,
    ) -> PressureArguments<'_, 'coefficients, 'regions> {
        PressureArguments {
            state: AcousticPressureState::new(
                &mut *self.diagnostics.inverse_density_perturbation,
                &mut *self.diagnostics.pressure_perturbation,
                &mut *self.time_levels.current_perturbation_geopotential,
                &mut *self.diagnostics.previous_pressure_perturbation,
            ),
            thermodynamics: AcousticPressureThermodynamics::new(
                self.inputs.pressure.full_inverse_density,
                &*self.time_levels.current_potential_temperature,
                &*self.saved_state.potential_temperature,
                &*self.saved_state.pressure_coefficient,
            ),
            masses: AcousticPressureMasses::new(
                &*self.time_levels.current_perturbation_column_mass,
                &*self.diagnostics.coupled_column_mass,
            ),
            coefficients: AcousticPressureCoefficients::new(
                self.coefficients.half_level_mass_multiplier,
                self.coefficients.half_level_mass_offset,
                self.coefficients.hydrostatic_pressure_multiplier,
            ),
            vertical_metrics: AcousticPressureVerticalMetrics::new(
                self.coefficients.inverse_half_level_spacing,
                self.coefficients.half_level_eta_thickness,
            ),
            parameters: AcousticPressureParameters::new(
                self.controls.base_potential_temperature,
                self.controls.pressure_divergence_damping,
            ),
            mode: self.controls.pressure_mode,
            damping_phase,
            region: self.regions.pressure,
        }
    }

    pub(super) fn vertical_coefficient_arguments(
        &mut self,
    ) -> VerticalCoefficientArguments<'_, 'coefficients, 'regions> {
        VerticalCoefficientArguments {
            outputs: VerticalAcousticSolveCoefficients::new(
                &mut *self.diagnostics.lower_diagonal,
                &mut *self.diagnostics.inverse_eliminated_diagonal,
                &mut *self.diagnostics.upper_elimination_factor,
            ),
            inputs: VerticalAcousticCoefficientInputs::new(
                self.inputs.masses.full,
                self.inputs.moisture.vertical,
                &*self.saved_state.pressure_coefficient,
            ),
            mass_coefficients: VerticalAcousticMassCoefficients::new(
                self.coefficients.half_level_mass_multiplier,
                self.coefficients.half_level_mass_offset,
                self.coefficients.full_level_mass_multiplier,
                self.coefficients.full_level_mass_offset,
            ),
            metrics: VerticalAcousticMetrics::new(
                self.coefficients.inverse_full_level_spacing,
                self.coefficients.inverse_half_level_spacing,
            ),
            parameters: VerticalAcousticCoefficientParameters::new(
                self.controls.acoustic_time_step,
                self.controls.gravitational_acceleration,
                self.controls.time_centering,
                self.controls.top_boundary,
            ),
            region: self.regions.vertical_coefficients,
        }
    }

    pub(super) fn horizontal_momentum_arguments(
        &mut self,
    ) -> HorizontalMomentumArguments<'_, 'regions> {
        HorizontalMomentumArguments {
            state: AcousticHorizontalMomentumState::new(
                &mut *self.time_levels.current_west_east_momentum,
                &mut *self.time_levels.current_south_north_momentum,
            ),
            inputs: AcousticHorizontalMomentumInputs::new(
                AcousticHorizontalMomentumTendencies::new(
                    &*self.inputs.tendencies.west_east_momentum,
                    &*self.inputs.tendencies.south_north_momentum,
                ),
                AcousticHorizontalPressureFields::new(
                    &*self.diagnostics.pressure_perturbation,
                    self.inputs.pressure.base_pressure,
                    &*self.time_levels.current_perturbation_geopotential,
                    self.inputs.pressure.pressure_point_geopotential,
                    self.inputs.pressure.full_inverse_density,
                    &*self.diagnostics.inverse_density_perturbation,
                ),
                AcousticHorizontalMassFields::new(
                    &*self.time_levels.current_perturbation_column_mass,
                    self.inputs.masses.west_east,
                    self.inputs.masses.south_north,
                    &*self.diagnostics.divergence_damping_column_mass,
                ),
                AcousticHorizontalMoistureCoefficients::new(
                    self.inputs.moisture.west_east,
                    self.inputs.moisture.south_north,
                ),
                AcousticHorizontalMapFactors::new(
                    self.inputs.map_factors.west_east_x,
                    self.inputs.map_factors.west_east_y,
                    self.inputs.map_factors.south_north_x,
                    self.inputs.map_factors.inverse_south_north_x,
                    self.inputs.map_factors.south_north_y,
                ),
                AcousticHorizontalVerticalCoefficients::new(
                    self.coefficients.half_level_mass_multiplier,
                    self.coefficients.half_level_mass_offset,
                    self.coefficients.upper_full_level_weight,
                    self.coefficients.lower_full_level_weight,
                    self.coefficients.inverse_half_level_spacing,
                ),
            ),
            parameters: AcousticHorizontalMomentumParameters::new(
                self.controls.inverse_west_east_grid_spacing,
                self.controls.inverse_south_north_grid_spacing,
                self.controls.acoustic_time_step,
                self.controls.boundary_velocity_weights,
                self.controls.horizontal_divergence_damping,
                self.controls.pressure_mode,
                self.controls.top_boundary,
            ),
            boundary_policy: self.controls.horizontal_boundary_policy,
            region: self.regions.horizontal_momentum,
        }
    }

    pub(super) fn mass_theta_arguments(&mut self) -> MassThetaArguments<'_, 'regions> {
        MassThetaArguments {
            state: AcousticMassThetaState::new(
                &mut *self.diagnostics.vertical_mass_flux,
                &mut *self.time_levels.current_perturbation_column_mass,
                &mut *self.time_levels.current_potential_temperature,
            ),
            diagnostics: AcousticMassThetaDiagnostics::new(
                &mut *self.diagnostics.time_centered_column_mass,
                &mut *self.diagnostics.coupled_column_mass,
                &mut *self.diagnostics.divergence_damping_column_mass,
                &mut *self.diagnostics.time_averaged_thermodynamics,
            ),
            inputs: AcousticMassThetaInputs::new(
                AcousticMassThetaMomentumInputs::new(
                    &*self.time_levels.current_west_east_momentum,
                    &*self.saved_state.west_east_momentum,
                    &*self.time_levels.current_south_north_momentum,
                    &*self.saved_state.south_north_momentum,
                    &*self.saved_state.vertical_mass_flux,
                ),
                AcousticMassThetaMassInputs::new(
                    self.inputs.masses.full,
                    self.inputs.masses.west_east,
                    self.inputs.masses.south_north,
                    self.inputs.masses.tendency,
                ),
                AcousticMassThetaThermodynamicInputs::new(
                    &*self.saved_state.potential_temperature,
                    self.inputs.tendencies.potential_temperature,
                ),
                AcousticMassThetaMapFactors::new(
                    self.inputs.map_factors.west_east_y,
                    self.inputs.map_factors.inverse_south_north_x,
                    self.inputs.map_factors.mass_point_x,
                    self.inputs.map_factors.mass_point_y,
                ),
                AcousticMassThetaVerticalCoefficients::new(
                    self.coefficients.half_level_mass_multiplier,
                    self.coefficients.half_level_mass_offset,
                    self.coefficients.half_level_eta_thickness,
                    self.coefficients.upper_full_level_weight,
                    self.coefficients.lower_full_level_weight,
                    self.coefficients.inverse_half_level_spacing,
                ),
            ),
            parameters: AcousticMassThetaParameters::new(
                self.controls.inverse_west_east_grid_spacing,
                self.controls.inverse_south_north_grid_spacing,
                self.controls.acoustic_time_step,
                self.controls.time_centering,
            ),
            boundary_policy: self.controls.mass_theta_boundary_policy,
            region: self.regions.mass_theta,
        }
    }

    pub(super) fn vertical_momentum_arguments(
        &mut self,
    ) -> VerticalMomentumArguments<'_, 'regions> {
        VerticalMomentumArguments {
            state: AcousticVerticalState::new(
                &mut *self.time_levels.current_vertical_momentum,
                &mut *self.time_levels.current_perturbation_geopotential,
                &mut *self.diagnostics.time_averaged_thermodynamics,
            ),
            workspace: AcousticVerticalWorkspace::new(
                &mut *self.workspace.geopotential_right_hand_side,
            ),
            inputs: AcousticVerticalInputs::new(
                AcousticVerticalMomentumInputs::new(
                    self.inputs.tendencies.vertical_momentum,
                    &*self.diagnostics.vertical_mass_flux,
                    &*self.saved_state.vertical_momentum,
                    &*self.time_levels.current_west_east_momentum,
                    &*self.time_levels.current_south_north_momentum,
                ),
                AcousticVerticalMassInputs::new(
                    self.inputs.masses.full,
                    &*self.diagnostics.time_centered_column_mass,
                    &*self.diagnostics.coupled_column_mass,
                ),
                AcousticVerticalThermodynamicInputs::new(
                    &*self.time_levels.current_potential_temperature,
                    &*self.saved_state.potential_temperature,
                ),
                AcousticVerticalGeopotentialInputs::new(
                    &*self.saved_state.perturbation_geopotential,
                    self.inputs.pressure.base_geopotential,
                    self.inputs.tendencies.perturbation_geopotential,
                ),
                AcousticVerticalMapFactors::new(
                    self.inputs.map_factors.terrain_height,
                    self.inputs.map_factors.mass_point_x,
                    self.inputs.map_factors.mass_point_y,
                ),
                AcousticVerticalSolveInputs::new(
                    &*self.saved_state.pressure_coefficient,
                    self.inputs.moisture.vertical,
                    self.inputs.pressure.full_inverse_density,
                    &*self.diagnostics.lower_diagonal,
                    &*self.diagnostics.inverse_eliminated_diagonal,
                    &*self.diagnostics.upper_elimination_factor,
                ),
                AcousticVerticalLevelCoefficients::new(
                    self.coefficients.half_level_mass_multiplier,
                    self.coefficients.half_level_mass_offset,
                    self.coefficients.full_level_mass_multiplier,
                    self.coefficients.full_level_mass_offset,
                    self.coefficients.upper_full_level_weight,
                    self.coefficients.lower_full_level_weight,
                    self.coefficients.inverse_half_level_spacing,
                    self.coefficients.inverse_full_level_spacing,
                ),
            ),
            parameters: AcousticVerticalParameters::new(
                self.controls.inverse_west_east_grid_spacing,
                self.controls.inverse_south_north_grid_spacing,
                self.controls.acoustic_time_step,
                self.controls.base_potential_temperature,
                self.controls.time_centering,
                self.controls.boundary_velocity_weights,
            ),
            boundary_policy: self.controls.vertical_boundary_policy,
            top_boundary: self.controls.top_boundary,
            vertical_advection: self.controls.vertical_advection,
            damping: self.controls.vertical_damping,
            region: self.regions.vertical_momentum,
        }
    }

    pub(super) fn flux_accumulation_arguments(
        &mut self,
        iteration: usize,
    ) -> AcousticTrajectoryResult<FluxAccumulationArguments<'_, 'coefficients, 'regions>> {
        Ok(FluxAccumulationArguments {
            averages: AcousticFluxRunningAverages {
                west_east: &mut *self.diagnostics.average_west_east_mass_flux,
                south_north: &mut *self.diagnostics.average_south_north_mass_flux,
                vertical: &mut *self.diagnostics.average_vertical_mass_flux,
            },
            current: AcousticFluxCurrentFields {
                west_east: &*self.time_levels.current_west_east_momentum,
                south_north: &*self.time_levels.current_south_north_momentum,
                vertical: &*self.diagnostics.vertical_mass_flux,
            },
            linear: AcousticFluxLinearFields {
                west_east: &*self.saved_state.west_east_momentum,
                south_north: &*self.saved_state.south_north_momentum,
                vertical: &*self.saved_state.vertical_mass_flux,
            },
            masses: AcousticFluxMassFields {
                west_east: self.inputs.masses.west_east,
                south_north: self.inputs.masses.south_north,
            },
            map_factors: AcousticFluxMapFactors {
                west_east_y: self.inputs.map_factors.west_east_y,
                inverse_south_north_x: self.inputs.map_factors.inverse_south_north_x,
            },
            column_mass_multiplier: self.coefficients.half_level_mass_multiplier,
            column_mass_offset: self.coefficients.half_level_mass_offset,
            phase: AcousticSubstepPhase::try_new(iteration, self.controls.substep_count)?,
            region: self.regions.flux_accumulation,
        })
    }
}
