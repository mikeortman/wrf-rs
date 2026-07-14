use wrf_compute::{CpuField, FieldStorage, GridShape};

use crate::{
    AcousticStepPreparationCoefficient, AcousticStepPreparationCoefficients,
    AcousticStepPreparationColumnMassTimeLevels, AcousticStepPreparationDiagnosticInputs,
    AcousticStepPreparationError, AcousticStepPreparationField, AcousticStepPreparationMapFactors,
    AcousticStepPreparationMassInputs, AcousticStepPreparationMassOutputs,
    AcousticStepPreparationPhase, AcousticStepPreparationRegion, AcousticStepPreparationResult,
    AcousticStepPreparationSavedOutputs, AcousticStepPreparationVolumeTimeLevels,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn validate_operation(
    volume: &AcousticStepPreparationVolumeTimeLevels<'_, CpuField<f32>>,
    column_mass: &AcousticStepPreparationColumnMassTimeLevels<'_, CpuField<f32>>,
    saved: &AcousticStepPreparationSavedOutputs<'_, CpuField<f32>>,
    mass_outputs: &AcousticStepPreparationMassOutputs<'_, CpuField<f32>>,
    mass_inputs: &AcousticStepPreparationMassInputs<'_, CpuField<f32>>,
    diagnostics: &AcousticStepPreparationDiagnosticInputs<'_, CpuField<f32>>,
    maps: &AcousticStepPreparationMapFactors<'_, CpuField<f32>>,
    coefficients: AcousticStepPreparationCoefficients<'_>,
    phase: AcousticStepPreparationPhase,
    region: &AcousticStepPreparationRegion,
) -> AcousticStepPreparationResult<()> {
    let volume_shape = region.shape();
    let horizontal_shape = volume_shape.horizontal_shape();
    for (field, role) in [
        (
            &*volume.previous_west_east_velocity,
            AcousticStepPreparationField::PreviousWestEastVelocity,
        ),
        (
            &*volume.current_west_east_velocity,
            AcousticStepPreparationField::CurrentWestEastVelocity,
        ),
        (
            &*volume.previous_south_north_velocity,
            AcousticStepPreparationField::PreviousSouthNorthVelocity,
        ),
        (
            &*volume.current_south_north_velocity,
            AcousticStepPreparationField::CurrentSouthNorthVelocity,
        ),
        (
            &*volume.previous_vertical_velocity,
            AcousticStepPreparationField::PreviousVerticalVelocity,
        ),
        (
            &*volume.current_vertical_velocity,
            AcousticStepPreparationField::CurrentVerticalVelocity,
        ),
        (
            &*volume.previous_potential_temperature,
            AcousticStepPreparationField::PreviousPotentialTemperature,
        ),
        (
            &*volume.current_potential_temperature,
            AcousticStepPreparationField::CurrentPotentialTemperature,
        ),
        (
            &*volume.previous_geopotential,
            AcousticStepPreparationField::PreviousGeopotential,
        ),
        (
            &*volume.current_geopotential,
            AcousticStepPreparationField::CurrentGeopotential,
        ),
        (
            &*saved.west_east_velocity,
            AcousticStepPreparationField::SavedWestEastVelocity,
        ),
        (
            &*saved.south_north_velocity,
            AcousticStepPreparationField::SavedSouthNorthVelocity,
        ),
        (
            &*saved.vertical_velocity,
            AcousticStepPreparationField::SavedVerticalVelocity,
        ),
        (
            &*saved.potential_temperature,
            AcousticStepPreparationField::SavedPotentialTemperature,
        ),
        (
            &*saved.geopotential,
            AcousticStepPreparationField::SavedGeopotential,
        ),
        (&*saved.omega, AcousticStepPreparationField::SavedOmega),
        (
            &*saved.pressure_coefficient,
            AcousticStepPreparationField::PressureCoefficient,
        ),
        (
            diagnostics.base_pressure,
            AcousticStepPreparationField::BasePressure,
        ),
        (
            diagnostics.perturbation_pressure,
            AcousticStepPreparationField::PerturbationPressure,
        ),
        (
            diagnostics.inverse_density,
            AcousticStepPreparationField::InverseDensity,
        ),
        (diagnostics.omega, AcousticStepPreparationField::Omega),
    ] {
        validate_shape(field, role, volume_shape)?;
    }
    for (field, role) in [
        (
            &*column_mass.previous,
            AcousticStepPreparationField::PreviousColumnMass,
        ),
        (
            &*column_mass.current,
            AcousticStepPreparationField::CurrentColumnMass,
        ),
        (
            mass_inputs.base,
            AcousticStepPreparationField::BaseColumnMass,
        ),
        (
            mass_inputs.west_east,
            AcousticStepPreparationField::WestEastColumnMass,
        ),
        (
            mass_inputs.south_north,
            AcousticStepPreparationField::SouthNorthColumnMass,
        ),
        (
            mass_inputs.full,
            AcousticStepPreparationField::FullColumnMass,
        ),
        (
            &*mass_outputs.saved_west_east,
            AcousticStepPreparationField::SavedWestEastColumnMass,
        ),
        (
            &*mass_outputs.saved_south_north,
            AcousticStepPreparationField::SavedSouthNorthColumnMass,
        ),
        (
            &*mass_outputs.saved_mass_point,
            AcousticStepPreparationField::SavedMassPointColumnMass,
        ),
        (
            &*mass_outputs.divergence_damping,
            AcousticStepPreparationField::DivergenceDamping,
        ),
        (
            &*saved.column_mass,
            AcousticStepPreparationField::SavedColumnMass,
        ),
        (
            maps.west_east_momentum_south_north,
            AcousticStepPreparationField::WestEastMomentumMapFactor,
        ),
        (
            maps.inverse_south_north_momentum_west_east,
            AcousticStepPreparationField::InverseSouthNorthMomentumMapFactor,
        ),
        (
            maps.mass_point_south_north,
            AcousticStepPreparationField::MassPointMapFactor,
        ),
    ] {
        validate_shape(field, role, horizontal_shape)?;
    }
    let expected = volume_shape.bottom_top_points();
    for (values, coefficient) in [
        (
            coefficients.half_level_mass_multiplier,
            AcousticStepPreparationCoefficient::HalfLevelMassMultiplier,
        ),
        (
            coefficients.half_level_offset,
            AcousticStepPreparationCoefficient::HalfLevelOffset,
        ),
        (
            coefficients.full_level_mass_multiplier,
            AcousticStepPreparationCoefficient::FullLevelMassMultiplier,
        ),
        (
            coefficients.full_level_offset,
            AcousticStepPreparationCoefficient::FullLevelOffset,
        ),
    ] {
        if values.len() != expected {
            return Err(AcousticStepPreparationError::CoefficientLengthMismatch {
                coefficient,
                expected,
                actual: values.len(),
            });
        }
    }
    if !phase.switches_time_levels() {
        region.validate_later_substep_neighbors()?;
    }
    Ok(())
}

fn validate_shape(
    field: &CpuField<f32>,
    role: AcousticStepPreparationField,
    expected: GridShape,
) -> AcousticStepPreparationResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(AcousticStepPreparationError::FieldShapeMismatch {
            field: role,
            expected,
            actual,
        });
    }
    Ok(())
}
