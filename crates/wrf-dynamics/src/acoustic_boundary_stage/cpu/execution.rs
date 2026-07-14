use crate::acoustic_trajectory::AcousticTrajectoryExecution;
use crate::{
    AcousticBoundaryStageControls, AcousticBoundaryStageRegions, AcousticBoundaryStageResult,
    AcousticPressureDampingPhase, PhysicalBoundaryKernels, PhysicalBoundaryVariable,
    SpecifiedBoundaryGeopotentialInputs, SpecifiedBoundaryGeopotentialKernels,
    SpecifiedBoundaryUpdateKernels, SpecifiedBoundaryUpdateParameters,
    SpecifiedBoundaryWestEastPeriodicity, SpecifiedBoundaryZeroGradientKernels,
    SpecifiedBoundaryZeroGradientParameters,
};

#[cfg(test)]
use std::cell::RefCell;
#[cfg(test)]
use wrf_compute::CpuField;

#[cfg(test)]
#[derive(Debug)]
pub(super) struct AcousticBoundaryStageSnapshot {
    pub(super) label: String,
    pub(super) bits: Vec<u32>,
}

#[cfg(test)]
thread_local! {
    static CAPTURED_STAGE_SNAPSHOTS: RefCell<Option<Vec<AcousticBoundaryStageSnapshot>>> =
        const { RefCell::new(None) };
}

#[cfg(test)]
pub(super) fn begin_stage_snapshot_capture() {
    CAPTURED_STAGE_SNAPSHOTS.with(|snapshots| {
        assert!(
            snapshots.borrow().is_none(),
            "stage snapshot capture is already active"
        );
        *snapshots.borrow_mut() = Some(Vec::new());
    });
}

#[cfg(test)]
pub(super) fn take_stage_snapshots() -> Vec<AcousticBoundaryStageSnapshot> {
    CAPTURED_STAGE_SNAPSHOTS.with(|snapshots| {
        snapshots
            .borrow_mut()
            .take()
            .expect("stage snapshot capture was not active")
    })
}

pub(super) struct AcousticBoundaryStageExecution<'backend, 'fields, 'coefficients, 'regions> {
    pub(super) trajectory: AcousticTrajectoryExecution<'backend, 'fields, 'coefficients, 'regions>,
    pub(super) controls: AcousticBoundaryStageControls,
    pub(super) regions: AcousticBoundaryStageRegions<'regions>,
}

impl<'backend, 'fields, 'coefficients, 'regions>
    AcousticBoundaryStageExecution<'backend, 'fields, 'coefficients, 'regions>
{
    pub(super) const fn new(
        trajectory: AcousticTrajectoryExecution<'backend, 'fields, 'coefficients, 'regions>,
        controls: AcousticBoundaryStageControls,
        regions: AcousticBoundaryStageRegions<'regions>,
    ) -> Self {
        Self {
            trajectory,
            controls,
            regions,
        }
    }

    pub(super) fn run(&mut self) -> AcousticBoundaryStageResult<()> {
        self.trajectory.prepare()?;
        #[cfg(test)]
        {
            capture_volume_snapshot(
                "prepare:u1",
                self.trajectory.time_levels.previous_west_east_momentum,
            );
            capture_volume_snapshot(
                "prepare:u2",
                self.trajectory.time_levels.current_west_east_momentum,
            );
            capture_volume_snapshot(
                "prepare:v1",
                self.trajectory.time_levels.previous_south_north_momentum,
            );
            capture_volume_snapshot(
                "prepare:v2",
                self.trajectory.time_levels.current_south_north_momentum,
            );
            capture_volume_snapshot(
                "prepare:w1",
                self.trajectory.time_levels.previous_vertical_momentum,
            );
            capture_volume_snapshot(
                "prepare:w2",
                self.trajectory.time_levels.current_vertical_momentum,
            );
            capture_volume_snapshot(
                "prepare:t1",
                self.trajectory.time_levels.previous_potential_temperature,
            );
            capture_volume_snapshot(
                "prepare:t2",
                self.trajectory.time_levels.current_potential_temperature,
            );
            capture_volume_snapshot(
                "prepare:ph1",
                self.trajectory
                    .time_levels
                    .previous_perturbation_geopotential,
            );
            capture_volume_snapshot(
                "prepare:ph2",
                self.trajectory
                    .time_levels
                    .current_perturbation_geopotential,
            );
            capture_volume_snapshot("prepare:us", self.trajectory.saved_state.west_east_momentum);
            capture_volume_snapshot(
                "prepare:vs",
                self.trajectory.saved_state.south_north_momentum,
            );
            capture_volume_snapshot("prepare:ws", self.trajectory.saved_state.vertical_momentum);
            capture_volume_snapshot(
                "prepare:ts",
                self.trajectory.saved_state.potential_temperature,
            );
            capture_volume_snapshot(
                "prepare:phs",
                self.trajectory.saved_state.perturbation_geopotential,
            );
            capture_volume_snapshot(
                "prepare:c2a",
                self.trajectory.saved_state.pressure_coefficient,
            );
            capture_volume_snapshot(
                "prepare:ww1",
                self.trajectory.saved_state.vertical_mass_flux,
            );
            capture_horizontal_snapshot(
                "prepare:mu1",
                self.trajectory
                    .time_levels
                    .previous_perturbation_column_mass,
            );
            capture_horizontal_snapshot(
                "prepare:mu2",
                self.trajectory.time_levels.current_perturbation_column_mass,
            );
            capture_horizontal_snapshot(
                "prepare:muus",
                self.trajectory.diagnostics.saved_west_east_column_mass,
            );
            capture_horizontal_snapshot(
                "prepare:muvs",
                self.trajectory.diagnostics.saved_south_north_column_mass,
            );
            capture_horizontal_snapshot(
                "prepare:muts",
                self.trajectory.diagnostics.coupled_column_mass,
            );
            capture_horizontal_snapshot(
                "prepare:mudf",
                self.trajectory.diagnostics.divergence_damping_column_mass,
            );
            capture_horizontal_snapshot(
                "prepare:mus",
                self.trajectory.saved_state.perturbation_column_mass,
            );
        }
        self.trajectory
            .update_pressure(AcousticPressureDampingPhase::Initialize)?;
        #[cfg(test)]
        {
            capture_volume_snapshot(
                "pressure_initial:p",
                self.trajectory.diagnostics.pressure_perturbation,
            );
            capture_volume_snapshot(
                "pressure_initial:al",
                self.trajectory.diagnostics.inverse_density_perturbation,
            );
            capture_volume_snapshot(
                "pressure_initial:ph2",
                self.trajectory
                    .time_levels
                    .current_perturbation_geopotential,
            );
            capture_volume_snapshot(
                "pressure_initial:pm1",
                self.trajectory.diagnostics.previous_pressure_perturbation,
            );
        }
        self.trajectory.calculate_vertical_coefficients()?;
        #[cfg(test)]
        {
            capture_volume_snapshot("coefficients:a", self.trajectory.diagnostics.lower_diagonal);
            capture_volume_snapshot(
                "coefficients:alpha",
                self.trajectory.diagnostics.inverse_eliminated_diagonal,
            );
            capture_volume_snapshot(
                "coefficients:gamma",
                self.trajectory.diagnostics.upper_elimination_factor,
            );
        }
        self.apply_initial_physical_boundaries()?;
        #[cfg(test)]
        {
            capture_volume_snapshot(
                "physical_initial:ru_tend",
                self.trajectory.inputs.tendencies.west_east_momentum,
            );
            capture_volume_snapshot(
                "physical_initial:rv_tend",
                self.trajectory.inputs.tendencies.south_north_momentum,
            );
            capture_volume_snapshot(
                "physical_initial:ph2",
                self.trajectory
                    .time_levels
                    .current_perturbation_geopotential,
            );
            capture_volume_snapshot(
                "physical_initial:al",
                self.trajectory.diagnostics.inverse_density_perturbation,
            );
            capture_volume_snapshot(
                "physical_initial:p",
                self.trajectory.diagnostics.pressure_perturbation,
            );
            capture_volume_snapshot(
                "physical_initial:t1",
                self.trajectory.time_levels.previous_potential_temperature,
            );
            capture_volume_snapshot(
                "physical_initial:ts",
                self.trajectory.saved_state.potential_temperature,
            );
            capture_horizontal_snapshot(
                "physical_initial:mu1",
                self.trajectory
                    .time_levels
                    .previous_perturbation_column_mass,
            );
            capture_horizontal_snapshot(
                "physical_initial:mu2",
                self.trajectory.time_levels.current_perturbation_column_mass,
            );
            capture_horizontal_snapshot(
                "physical_initial:mudf",
                self.trajectory.diagnostics.divergence_damping_column_mass,
            );
        }

        for iteration in 1..=self.trajectory.controls.substep_count {
            self.trajectory.advance_horizontal_momentum()?;
            #[cfg(test)]
            {
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:uv:u2"),
                    self.trajectory.time_levels.current_west_east_momentum,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:uv:v2"),
                    self.trajectory.time_levels.current_south_north_momentum,
                );
            }
            self.update_horizontal_momentum_boundaries()?;
            #[cfg(test)]
            {
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:specified_uv:u2"),
                    self.trajectory.time_levels.current_west_east_momentum,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:specified_uv:v2"),
                    self.trajectory.time_levels.current_south_north_momentum,
                );
            }
            self.trajectory.advance_mass_and_theta()?;
            #[cfg(test)]
            {
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:mass_theta:ww"),
                    self.trajectory.diagnostics.vertical_mass_flux,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:mass_theta:ww1"),
                    self.trajectory.saved_state.vertical_mass_flux,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:mass_theta:t2"),
                    self.trajectory.time_levels.current_potential_temperature,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:mass_theta:t2save"),
                    self.trajectory.diagnostics.time_averaged_thermodynamics,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:mass_theta:ru_m"),
                    self.trajectory.diagnostics.average_west_east_mass_flux,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:mass_theta:rv_m"),
                    self.trajectory.diagnostics.average_south_north_mass_flux,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:mass_theta:ww_m"),
                    self.trajectory.diagnostics.average_vertical_mass_flux,
                );
                capture_horizontal_snapshot(
                    &format!("iteration_{iteration}:mass_theta:muave"),
                    self.trajectory.diagnostics.time_centered_column_mass,
                );
                capture_horizontal_snapshot(
                    &format!("iteration_{iteration}:mass_theta:muts"),
                    self.trajectory.diagnostics.coupled_column_mass,
                );
                capture_horizontal_snapshot(
                    &format!("iteration_{iteration}:mass_theta:mudf"),
                    self.trajectory.diagnostics.divergence_damping_column_mass,
                );
                capture_horizontal_snapshot(
                    &format!("iteration_{iteration}:mass_theta:mu2"),
                    self.trajectory.time_levels.current_perturbation_column_mass,
                );
            }
            self.update_mass_theta_boundaries()?;
            #[cfg(test)]
            {
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:specified_mass_theta:t2"),
                    self.trajectory.time_levels.current_potential_temperature,
                );
                capture_horizontal_snapshot(
                    &format!("iteration_{iteration}:specified_mass_theta:mu2"),
                    self.trajectory.time_levels.current_perturbation_column_mass,
                );
                capture_horizontal_snapshot(
                    &format!("iteration_{iteration}:specified_mass_theta:muts"),
                    self.trajectory.diagnostics.coupled_column_mass,
                );
            }
            self.trajectory.advance_vertical_momentum()?;
            #[cfg(test)]
            {
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:vertical:w2"),
                    self.trajectory.time_levels.current_vertical_momentum,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:vertical:ph2"),
                    self.trajectory
                        .time_levels
                        .current_perturbation_geopotential,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:vertical:t2save"),
                    self.trajectory.diagnostics.time_averaged_thermodynamics,
                );
            }
            self.trajectory.accumulate_fluxes(iteration)?;
            #[cfg(test)]
            {
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:flux:ru_m"),
                    self.trajectory.diagnostics.average_west_east_mass_flux,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:flux:rv_m"),
                    self.trajectory.diagnostics.average_south_north_mass_flux,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:flux:ww_m"),
                    self.trajectory.diagnostics.average_vertical_mass_flux,
                );
            }
            self.update_nonhydrostatic_boundaries()?;
            #[cfg(test)]
            {
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:specified_vertical:ph2"),
                    self.trajectory
                        .time_levels
                        .current_perturbation_geopotential,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:specified_vertical:w2"),
                    self.trajectory.time_levels.current_vertical_momentum,
                );
            }
            self.trajectory
                .update_pressure(AcousticPressureDampingPhase::Advance)?;
            #[cfg(test)]
            {
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:pressure:p"),
                    self.trajectory.diagnostics.pressure_perturbation,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:pressure:al"),
                    self.trajectory.diagnostics.inverse_density_perturbation,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:pressure:ph2"),
                    self.trajectory
                        .time_levels
                        .current_perturbation_geopotential,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:pressure:pm1"),
                    self.trajectory.diagnostics.previous_pressure_perturbation,
                );
            }
            self.apply_substep_physical_boundaries()?;
            #[cfg(test)]
            {
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:physical:ph2"),
                    self.trajectory
                        .time_levels
                        .current_perturbation_geopotential,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:physical:al"),
                    self.trajectory.diagnostics.inverse_density_perturbation,
                );
                capture_volume_snapshot(
                    &format!("iteration_{iteration}:physical:p"),
                    self.trajectory.diagnostics.pressure_perturbation,
                );
                capture_horizontal_snapshot(
                    &format!("iteration_{iteration}:physical:muts"),
                    self.trajectory.diagnostics.coupled_column_mass,
                );
                capture_horizontal_snapshot(
                    &format!("iteration_{iteration}:physical:mu2"),
                    self.trajectory.time_levels.current_perturbation_column_mass,
                );
                capture_horizontal_snapshot(
                    &format!("iteration_{iteration}:physical:mudf"),
                    self.trajectory.diagnostics.divergence_damping_column_mass,
                );
            }
        }
        Ok(())
    }

    fn apply_initial_physical_boundaries(&mut self) -> AcousticBoundaryStageResult<()> {
        let backend = self.trajectory.backend;
        let conditions = self.controls.physical_boundaries;
        let region = self.regions.physical;
        backend.apply_physical_boundary(
            self.trajectory.inputs.tendencies.west_east_momentum,
            PhysicalBoundaryVariable::WestEastFace,
            conditions,
            region,
        )?;
        backend.apply_physical_boundary(
            self.trajectory.inputs.tendencies.south_north_momentum,
            PhysicalBoundaryVariable::SouthNorthFace,
            conditions,
            region,
        )?;
        backend.apply_physical_boundary(
            self.trajectory
                .time_levels
                .current_perturbation_geopotential,
            PhysicalBoundaryVariable::FullLevel,
            conditions,
            region,
        )?;
        backend.apply_physical_boundary(
            self.trajectory.diagnostics.inverse_density_perturbation,
            PhysicalBoundaryVariable::MassHalfLevel,
            conditions,
            region,
        )?;
        backend.apply_physical_boundary(
            self.trajectory.diagnostics.pressure_perturbation,
            PhysicalBoundaryVariable::MassHalfLevel,
            conditions,
            region,
        )?;
        backend.apply_physical_boundary(
            self.trajectory.time_levels.previous_potential_temperature,
            PhysicalBoundaryVariable::MassHalfLevel,
            conditions,
            region,
        )?;
        backend.apply_physical_boundary(
            self.trajectory.saved_state.potential_temperature,
            PhysicalBoundaryVariable::MassHalfLevel,
            conditions,
            region,
        )?;
        backend.apply_physical_boundary_horizontal(
            self.trajectory
                .time_levels
                .previous_perturbation_column_mass,
            PhysicalBoundaryVariable::MassHalfLevel,
            conditions,
            region,
        )?;
        backend.apply_physical_boundary_horizontal(
            self.trajectory.time_levels.current_perturbation_column_mass,
            PhysicalBoundaryVariable::MassHalfLevel,
            conditions,
            region,
        )?;
        backend.apply_physical_boundary_horizontal(
            self.trajectory.diagnostics.divergence_damping_column_mass,
            PhysicalBoundaryVariable::MassHalfLevel,
            conditions,
            region,
        )?;
        Ok(())
    }

    fn update_horizontal_momentum_boundaries(&mut self) -> AcousticBoundaryStageResult<()> {
        if !self.controls.has_specified_updates() {
            return Ok(());
        }
        let backend = self.trajectory.backend;
        let parameters = self.update_parameters();
        let periodicity = self.west_east_periodicity();
        backend.update_specified_boundary(
            self.trajectory.time_levels.current_west_east_momentum,
            &*self.trajectory.inputs.tendencies.west_east_momentum,
            parameters,
            periodicity,
            self.regions.west_east_momentum,
        )?;
        backend.update_specified_boundary(
            self.trajectory.time_levels.current_south_north_momentum,
            &*self.trajectory.inputs.tendencies.south_north_momentum,
            parameters,
            periodicity,
            self.regions.south_north_momentum,
        )?;
        Ok(())
    }

    fn update_mass_theta_boundaries(&mut self) -> AcousticBoundaryStageResult<()> {
        if !self.controls.has_specified_updates() {
            return Ok(());
        }
        let backend = self.trajectory.backend;
        let parameters = self.update_parameters();
        let periodicity = self.west_east_periodicity();
        backend.update_specified_boundary(
            self.trajectory.time_levels.current_potential_temperature,
            self.trajectory.inputs.tendencies.potential_temperature,
            parameters,
            periodicity,
            self.regions.potential_temperature,
        )?;
        backend.update_specified_boundary(
            self.trajectory.time_levels.current_perturbation_column_mass,
            self.trajectory.inputs.masses.tendency,
            parameters,
            periodicity,
            self.regions.column_mass,
        )?;
        backend.update_specified_boundary(
            self.trajectory.diagnostics.coupled_column_mass,
            self.trajectory.inputs.masses.tendency,
            parameters,
            periodicity,
            self.regions.column_mass,
        )?;
        Ok(())
    }

    fn update_nonhydrostatic_boundaries(&mut self) -> AcousticBoundaryStageResult<()> {
        if !self.controls.has_specified_updates() {
            return Ok(());
        }
        let backend = self.trajectory.backend;
        let parameters = self.update_parameters();
        let periodicity = self.west_east_periodicity();
        backend.update_specified_boundary_geopotential(
            self.trajectory
                .time_levels
                .current_perturbation_geopotential,
            SpecifiedBoundaryGeopotentialInputs::new(
                &*self.trajectory.saved_state.perturbation_geopotential,
                self.trajectory.inputs.tendencies.perturbation_geopotential,
                self.trajectory.inputs.masses.tendency,
                &*self.trajectory.diagnostics.coupled_column_mass,
                self.trajectory.coefficients.full_level_mass_multiplier,
                self.trajectory.coefficients.full_level_mass_offset,
            ),
            parameters,
            periodicity,
            self.regions.full_level,
        )?;
        if self.controls.physical_boundaries.specified {
            backend.apply_zero_gradient_specified_boundary(
                self.trajectory.time_levels.current_vertical_momentum,
                SpecifiedBoundaryZeroGradientParameters::new(self.controls.specified_zone_width),
                periodicity,
                self.regions.full_level,
            )?;
        } else {
            backend.update_specified_boundary(
                self.trajectory.time_levels.current_vertical_momentum,
                self.trajectory.inputs.tendencies.vertical_momentum,
                parameters,
                periodicity,
                self.regions.full_level,
            )?;
        }
        Ok(())
    }

    fn apply_substep_physical_boundaries(&mut self) -> AcousticBoundaryStageResult<()> {
        let backend = self.trajectory.backend;
        let conditions = self.controls.physical_boundaries;
        let region = self.regions.physical;
        backend.apply_physical_boundary(
            self.trajectory
                .time_levels
                .current_perturbation_geopotential,
            PhysicalBoundaryVariable::FullLevel,
            conditions,
            region,
        )?;
        backend.apply_physical_boundary(
            self.trajectory.diagnostics.inverse_density_perturbation,
            PhysicalBoundaryVariable::MassHalfLevel,
            conditions,
            region,
        )?;
        backend.apply_physical_boundary(
            self.trajectory.diagnostics.pressure_perturbation,
            PhysicalBoundaryVariable::MassHalfLevel,
            conditions,
            region,
        )?;
        backend.apply_physical_boundary_horizontal(
            self.trajectory.diagnostics.coupled_column_mass,
            PhysicalBoundaryVariable::MassHalfLevel,
            conditions,
            region,
        )?;
        backend.apply_physical_boundary_horizontal(
            self.trajectory.time_levels.current_perturbation_column_mass,
            PhysicalBoundaryVariable::MassHalfLevel,
            conditions,
            region,
        )?;
        backend.apply_physical_boundary_horizontal(
            self.trajectory.diagnostics.divergence_damping_column_mass,
            PhysicalBoundaryVariable::MassHalfLevel,
            conditions,
            region,
        )?;
        Ok(())
    }

    pub(super) fn update_parameters(&self) -> SpecifiedBoundaryUpdateParameters {
        SpecifiedBoundaryUpdateParameters::new(
            self.trajectory.controls.acoustic_time_step,
            self.controls.specified_zone_width,
        )
    }

    pub(super) const fn west_east_periodicity(&self) -> SpecifiedBoundaryWestEastPeriodicity {
        if self.controls.physical_boundaries.periodic_x {
            SpecifiedBoundaryWestEastPeriodicity::Periodic
        } else {
            SpecifiedBoundaryWestEastPeriodicity::Nonperiodic
        }
    }
}

#[cfg(test)]
fn capture_volume_snapshot(label: &str, field: &CpuField<f32>) {
    CAPTURED_STAGE_SNAPSHOTS.with(|captured| {
        let mut captured = captured.borrow_mut();
        let Some(captured) = captured.as_mut() else {
            return;
        };
        let bits = field.values().iter().map(|value| value.to_bits()).collect();
        captured.push(AcousticBoundaryStageSnapshot {
            label: label.to_owned(),
            bits,
        });
    });
}

#[cfg(test)]
fn capture_horizontal_snapshot(label: &str, field: &CpuField<f32>) {
    CAPTURED_STAGE_SNAPSHOTS.with(|captured| {
        let mut captured = captured.borrow_mut();
        let Some(captured) = captured.as_mut() else {
            return;
        };
        let bits = field.values().iter().map(|value| value.to_bits()).collect();
        captured.push(AcousticBoundaryStageSnapshot {
            label: label.to_owned(),
            bits,
        });
    });
}
