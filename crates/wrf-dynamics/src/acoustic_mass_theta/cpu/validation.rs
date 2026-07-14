use wrf_compute::{CpuField, FieldStorage, GridShape};

use crate::{
    AcousticMassThetaBoundaryPolicy, AcousticMassThetaCoefficient, AcousticMassThetaDiagnostics,
    AcousticMassThetaError, AcousticMassThetaField, AcousticMassThetaInputs,
    AcousticMassThetaRegion, AcousticMassThetaResult, AcousticMassThetaState,
};

use super::super::domain::AcousticMassThetaActiveRanges;

pub(super) fn validate_operation(
    state: &AcousticMassThetaState<'_, CpuField<f32>>,
    diagnostics: &AcousticMassThetaDiagnostics<'_, CpuField<f32>>,
    inputs: AcousticMassThetaInputs<'_, CpuField<f32>>,
    boundary_policy: AcousticMassThetaBoundaryPolicy,
    region: &AcousticMassThetaRegion,
) -> AcousticMassThetaResult<AcousticMassThetaActiveRanges> {
    validate_volume_fields(state, diagnostics, inputs, region.shape())?;
    validate_horizontal_fields(
        state,
        diagnostics,
        inputs,
        region.shape().horizontal_shape(),
    )?;
    validate_coefficients(inputs, region.shape().bottom_top_points())?;
    region.active_ranges(boundary_policy)
}

fn validate_volume_fields(
    state: &AcousticMassThetaState<'_, CpuField<f32>>,
    diagnostics: &AcousticMassThetaDiagnostics<'_, CpuField<f32>>,
    inputs: AcousticMassThetaInputs<'_, CpuField<f32>>,
    expected: GridShape,
) -> AcousticMassThetaResult<()> {
    for (field, role) in [
        (
            &*state.vertical_mass_flux,
            AcousticMassThetaField::VerticalMassFlux,
        ),
        (
            &*state.potential_temperature,
            AcousticMassThetaField::PotentialTemperature,
        ),
        (
            &*diagnostics.previous_potential_temperature,
            AcousticMassThetaField::PreviousPotentialTemperature,
        ),
        (
            inputs.momentum.saved_vertical_mass_flux,
            AcousticMassThetaField::SavedVerticalMassFlux,
        ),
        (
            inputs.momentum.west_east,
            AcousticMassThetaField::WestEastMomentum,
        ),
        (
            inputs.momentum.saved_west_east,
            AcousticMassThetaField::SavedWestEastMomentum,
        ),
        (
            inputs.momentum.south_north,
            AcousticMassThetaField::SouthNorthMomentum,
        ),
        (
            inputs.momentum.saved_south_north,
            AcousticMassThetaField::SavedSouthNorthMomentum,
        ),
        (
            inputs.thermodynamics.saved_potential_temperature,
            AcousticMassThetaField::SavedPotentialTemperature,
        ),
        (
            inputs.thermodynamics.potential_temperature_tendency,
            AcousticMassThetaField::PotentialTemperatureTendency,
        ),
    ] {
        validate_field_shape(field, role, expected)?;
    }
    Ok(())
}

fn validate_horizontal_fields(
    state: &AcousticMassThetaState<'_, CpuField<f32>>,
    diagnostics: &AcousticMassThetaDiagnostics<'_, CpuField<f32>>,
    inputs: AcousticMassThetaInputs<'_, CpuField<f32>>,
    expected: GridShape,
) -> AcousticMassThetaResult<()> {
    for (field, role) in [
        (&*state.column_mass, AcousticMassThetaField::ColumnMass),
        (
            &*diagnostics.time_centered_column_mass,
            AcousticMassThetaField::TimeCenteredColumnMass,
        ),
        (
            &*diagnostics.coupled_column_mass,
            AcousticMassThetaField::CoupledColumnMass,
        ),
        (
            &*diagnostics.divergence_damping_mass_tendency,
            AcousticMassThetaField::DivergenceDampingMassTendency,
        ),
        (
            inputs.mass.base_full,
            AcousticMassThetaField::BaseFullColumnMass,
        ),
        (
            inputs.mass.west_east,
            AcousticMassThetaField::WestEastColumnMass,
        ),
        (
            inputs.mass.south_north,
            AcousticMassThetaField::SouthNorthColumnMass,
        ),
        (
            inputs.mass.tendency,
            AcousticMassThetaField::ColumnMassTendency,
        ),
        (
            inputs.map_factors.west_east_y,
            AcousticMassThetaField::WestEastYMapFactor,
        ),
        (
            inputs.map_factors.inverse_south_north_x,
            AcousticMassThetaField::InverseSouthNorthXMapFactor,
        ),
        (
            inputs.map_factors.mass_point_x,
            AcousticMassThetaField::MassPointXMapFactor,
        ),
        (
            inputs.map_factors.mass_point_y,
            AcousticMassThetaField::MassPointYMapFactor,
        ),
    ] {
        validate_field_shape(field, role, expected)?;
    }
    Ok(())
}

fn validate_field_shape(
    field: &CpuField<f32>,
    role: AcousticMassThetaField,
    expected: GridShape,
) -> AcousticMassThetaResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(AcousticMassThetaError::FieldShapeMismatch {
            field: role,
            expected,
            actual,
        });
    }
    Ok(())
}

fn validate_coefficients(
    inputs: AcousticMassThetaInputs<'_, CpuField<f32>>,
    expected: usize,
) -> AcousticMassThetaResult<()> {
    for (values, coefficient) in [
        (
            inputs.vertical.half_level_mass_multiplier,
            AcousticMassThetaCoefficient::HalfLevelMassMultiplier,
        ),
        (
            inputs.vertical.half_level_mass_offset,
            AcousticMassThetaCoefficient::HalfLevelMassOffset,
        ),
        (
            inputs.vertical.half_level_eta_thickness,
            AcousticMassThetaCoefficient::HalfLevelEtaThickness,
        ),
        (
            inputs.vertical.lower_interpolation_weight,
            AcousticMassThetaCoefficient::LowerInterpolationWeight,
        ),
        (
            inputs.vertical.upper_interpolation_weight,
            AcousticMassThetaCoefficient::UpperInterpolationWeight,
        ),
        (
            inputs.vertical.inverse_half_level_spacing,
            AcousticMassThetaCoefficient::InverseHalfLevelEtaSpacing,
        ),
    ] {
        if values.len() != expected {
            return Err(AcousticMassThetaError::CoefficientLengthMismatch {
                coefficient,
                expected,
                actual: values.len(),
            });
        }
    }
    Ok(())
}
