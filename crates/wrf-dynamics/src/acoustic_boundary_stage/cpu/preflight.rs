use crate::acoustic_trajectory::validate_acoustic_trajectory;
use crate::physical_boundary::{validate_horizontal_boundary, validate_volume_boundary};
use crate::specified_boundary_update::{
    validate_specified_boundary_geopotential, validate_specified_boundary_update,
    validate_zero_gradient_specified_boundary,
};
use crate::{
    AcousticBoundaryRegionRole, AcousticBoundaryStageError, AcousticBoundaryStageResult,
    AcousticPressureMode, SpecifiedBoundaryFieldLocation, SpecifiedBoundaryGeopotentialInputs,
    SpecifiedBoundaryZeroGradientParameters,
};

use super::execution::AcousticBoundaryStageExecution;

pub(super) fn validate(
    execution: &mut AcousticBoundaryStageExecution<'_, '_, '_, '_>,
) -> AcousticBoundaryStageResult<()> {
    validate_supported_configuration(execution)?;
    validate_acoustic_trajectory(&mut execution.trajectory)?;
    validate_physical_boundaries(execution)?;
    validate_specified_boundaries(execution)
}

fn validate_supported_configuration(
    execution: &AcousticBoundaryStageExecution<'_, '_, '_, '_>,
) -> AcousticBoundaryStageResult<()> {
    if execution.controls.physical_boundaries.polar {
        return Err(AcousticBoundaryStageError::PolarFilteringUnsupported);
    }
    if execution.trajectory.controls.pressure_mode != AcousticPressureMode::Nonhydrostatic {
        return Err(AcousticBoundaryStageError::HydrostaticModeUnsupported);
    }
    Ok(())
}

fn validate_physical_boundaries(
    execution: &AcousticBoundaryStageExecution<'_, '_, '_, '_>,
) -> AcousticBoundaryStageResult<()> {
    let trajectory = &execution.trajectory;
    let region = execution.regions.physical;
    for field in [
        &*trajectory.inputs.tendencies.west_east_momentum,
        &*trajectory.inputs.tendencies.south_north_momentum,
        &*trajectory.time_levels.current_perturbation_geopotential,
        &*trajectory.diagnostics.inverse_density_perturbation,
        &*trajectory.diagnostics.pressure_perturbation,
        &*trajectory.time_levels.previous_potential_temperature,
        &*trajectory.saved_state.potential_temperature,
    ] {
        validate_volume_boundary(field, region)?;
    }
    for field in [
        &*trajectory.time_levels.previous_perturbation_column_mass,
        &*trajectory.time_levels.current_perturbation_column_mass,
        &*trajectory.diagnostics.divergence_damping_column_mass,
        &*trajectory.diagnostics.coupled_column_mass,
    ] {
        validate_horizontal_boundary(field, region)?;
    }
    Ok(())
}

fn validate_specified_boundaries(
    execution: &mut AcousticBoundaryStageExecution<'_, '_, '_, '_>,
) -> AcousticBoundaryStageResult<()> {
    if !execution.controls.has_specified_updates() {
        return Ok(());
    }
    validate_region_locations(execution)?;
    let parameters = execution.update_parameters();
    let periodicity = execution.west_east_periodicity();
    let trajectory = &mut execution.trajectory;
    validate_specified_boundary_update(
        &*trajectory.time_levels.current_west_east_momentum,
        &*trajectory.inputs.tendencies.west_east_momentum,
        execution.regions.west_east_momentum,
    )?;
    validate_specified_boundary_update(
        &*trajectory.time_levels.current_south_north_momentum,
        &*trajectory.inputs.tendencies.south_north_momentum,
        execution.regions.south_north_momentum,
    )?;
    validate_specified_boundary_update(
        &*trajectory.time_levels.current_potential_temperature,
        trajectory.inputs.tendencies.potential_temperature,
        execution.regions.potential_temperature,
    )?;
    validate_specified_boundary_update(
        &*trajectory.time_levels.current_perturbation_column_mass,
        trajectory.inputs.masses.tendency,
        execution.regions.column_mass,
    )?;
    validate_specified_boundary_update(
        &*trajectory.diagnostics.coupled_column_mass,
        trajectory.inputs.masses.tendency,
        execution.regions.column_mass,
    )?;
    validate_specified_boundary_geopotential(
        trajectory.time_levels.current_perturbation_geopotential,
        SpecifiedBoundaryGeopotentialInputs::new(
            &*trajectory.saved_state.perturbation_geopotential,
            trajectory.inputs.tendencies.perturbation_geopotential,
            trajectory.inputs.masses.tendency,
            &*trajectory.diagnostics.coupled_column_mass,
            trajectory.coefficients.full_level_mass_multiplier,
            trajectory.coefficients.full_level_mass_offset,
        ),
        parameters,
        periodicity,
        execution.regions.full_level,
    )?;
    if execution.controls.physical_boundaries.specified {
        validate_zero_gradient_specified_boundary(
            trajectory.time_levels.current_vertical_momentum,
            SpecifiedBoundaryZeroGradientParameters::new(execution.controls.specified_zone_width),
            periodicity,
            execution.regions.full_level,
        )?;
    } else {
        validate_specified_boundary_update(
            &*trajectory.time_levels.current_vertical_momentum,
            trajectory.inputs.tendencies.vertical_momentum,
            execution.regions.full_level,
        )?;
    }
    Ok(())
}

fn validate_region_locations(
    execution: &AcousticBoundaryStageExecution<'_, '_, '_, '_>,
) -> AcousticBoundaryStageResult<()> {
    for (role, region, expected) in [
        (
            AcousticBoundaryRegionRole::WestEastMomentum,
            execution.regions.west_east_momentum,
            SpecifiedBoundaryFieldLocation::WestEastFace,
        ),
        (
            AcousticBoundaryRegionRole::SouthNorthMomentum,
            execution.regions.south_north_momentum,
            SpecifiedBoundaryFieldLocation::SouthNorthFace,
        ),
        (
            AcousticBoundaryRegionRole::PotentialTemperature,
            execution.regions.potential_temperature,
            SpecifiedBoundaryFieldLocation::MassHalfLevel,
        ),
        (
            AcousticBoundaryRegionRole::ColumnMass,
            execution.regions.column_mass,
            SpecifiedBoundaryFieldLocation::HorizontalMass,
        ),
        (
            AcousticBoundaryRegionRole::FullLevel,
            execution.regions.full_level,
            SpecifiedBoundaryFieldLocation::FullLevel,
        ),
    ] {
        if region.location() != expected {
            return Err(AcousticBoundaryStageError::RegionLocationMismatch {
                role,
                expected,
                actual: region.location(),
            });
        }
    }
    Ok(())
}
