use crate::acoustic_flux_accumulation::validate_acoustic_flux_accumulation;
use crate::acoustic_horizontal_momentum::validate_acoustic_horizontal_momentum;
use crate::acoustic_mass_theta::validate_acoustic_mass_and_theta;
use crate::acoustic_pressure::validate_acoustic_pressure;
use crate::acoustic_step_preparation::validate_acoustic_step_preparation;
use crate::acoustic_vertical_momentum::validate_acoustic_vertical_momentum;
use crate::vertical_acoustic_coefficients::validate_vertical_acoustic_coefficients;
use crate::{AcousticPressureDampingPhase, AcousticTrajectoryResult};

use super::execution::AcousticTrajectoryExecution;

pub(super) fn validate(
    execution: &mut AcousticTrajectoryExecution<'_, '_, '_, '_>,
) -> AcousticTrajectoryResult<()> {
    validate_preparation(execution)?;
    validate_pressure(execution)?;
    validate_vertical_coefficients(execution)?;
    validate_horizontal_momentum(execution)?;
    validate_mass_theta(execution)?;
    validate_vertical_momentum(execution)?;
    validate_flux_accumulation(execution)
}

fn validate_preparation(
    execution: &mut AcousticTrajectoryExecution<'_, '_, '_, '_>,
) -> AcousticTrajectoryResult<()> {
    let arguments = execution.preparation_arguments();
    validate_acoustic_step_preparation(
        &arguments.volume,
        &arguments.column_mass,
        &arguments.saved,
        &arguments.mass_outputs,
        &arguments.mass_inputs,
        &arguments.diagnostics,
        &arguments.maps,
        arguments.coefficients,
        arguments.phase,
        arguments.region,
    )?;
    Ok(())
}

fn validate_pressure(
    execution: &mut AcousticTrajectoryExecution<'_, '_, '_, '_>,
) -> AcousticTrajectoryResult<()> {
    let arguments = execution.pressure_arguments(AcousticPressureDampingPhase::Initialize);
    validate_acoustic_pressure(
        &arguments.state,
        &arguments.thermodynamics,
        &arguments.masses,
        arguments.coefficients,
        arguments.vertical_metrics,
        arguments.region,
    )?;
    Ok(())
}

fn validate_vertical_coefficients(
    execution: &mut AcousticTrajectoryExecution<'_, '_, '_, '_>,
) -> AcousticTrajectoryResult<()> {
    let arguments = execution.vertical_coefficient_arguments();
    validate_vertical_acoustic_coefficients(
        &arguments.outputs,
        arguments.inputs,
        arguments.mass_coefficients,
        arguments.metrics,
        arguments.region,
    )?;
    Ok(())
}

fn validate_horizontal_momentum(
    execution: &mut AcousticTrajectoryExecution<'_, '_, '_, '_>,
) -> AcousticTrajectoryResult<()> {
    let arguments = execution.horizontal_momentum_arguments();
    validate_acoustic_horizontal_momentum(
        &arguments.state,
        arguments.inputs,
        arguments.parameters,
        arguments.boundary_policy,
        arguments.region,
    )?;
    Ok(())
}

fn validate_mass_theta(
    execution: &mut AcousticTrajectoryExecution<'_, '_, '_, '_>,
) -> AcousticTrajectoryResult<()> {
    let arguments = execution.mass_theta_arguments();
    validate_acoustic_mass_and_theta(
        &arguments.state,
        &arguments.diagnostics,
        arguments.inputs,
        arguments.boundary_policy,
        arguments.region,
    )?;
    Ok(())
}

fn validate_vertical_momentum(
    execution: &mut AcousticTrajectoryExecution<'_, '_, '_, '_>,
) -> AcousticTrajectoryResult<()> {
    let arguments = execution.vertical_momentum_arguments();
    validate_acoustic_vertical_momentum(
        &arguments.state,
        &arguments.workspace,
        arguments.inputs,
        arguments.boundary_policy,
        arguments.region,
    )?;
    Ok(())
}

fn validate_flux_accumulation(
    execution: &mut AcousticTrajectoryExecution<'_, '_, '_, '_>,
) -> AcousticTrajectoryResult<()> {
    let arguments = execution.flux_accumulation_arguments(1)?;
    validate_acoustic_flux_accumulation(
        &arguments.averages,
        &arguments.current,
        &arguments.linear,
        &arguments.masses,
        &arguments.map_factors,
        arguments.column_mass_multiplier,
        arguments.column_mass_offset,
        arguments.region,
    )?;
    Ok(())
}
