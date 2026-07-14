use wrf_compute::{CpuField, FieldStorage, GridShape};

use crate::{
    AcousticHorizontalBoundaryPolicy, AcousticHorizontalMomentumCoefficient,
    AcousticHorizontalMomentumError, AcousticHorizontalMomentumField,
    AcousticHorizontalMomentumInputs, AcousticHorizontalMomentumParameters,
    AcousticHorizontalMomentumRegion, AcousticHorizontalMomentumResult,
    AcousticHorizontalMomentumState, AcousticPressureMode,
};

use super::super::domain::AcousticHorizontalMomentumActiveRanges;

pub(super) fn validate_operation(
    state: &AcousticHorizontalMomentumState<'_, CpuField<f32>>,
    inputs: AcousticHorizontalMomentumInputs<'_, CpuField<f32>>,
    parameters: AcousticHorizontalMomentumParameters,
    boundary_policy: AcousticHorizontalBoundaryPolicy,
    region: &AcousticHorizontalMomentumRegion,
) -> AcousticHorizontalMomentumResult<AcousticHorizontalMomentumActiveRanges> {
    validate_volume_fields(state, inputs, region.shape())?;
    validate_horizontal_fields(inputs, region.shape().horizontal_shape())?;
    validate_coefficients(inputs, region.shape().bottom_top_points())?;
    if parameters.mode == AcousticPressureMode::Nonhydrostatic
        && region.half_level_domain().len() < 3
    {
        return Err(
            AcousticHorizontalMomentumError::InsufficientNonhydrostaticLevels {
                available: region.half_level_domain().len(),
            },
        );
    }
    region.active_ranges(boundary_policy)
}

fn validate_volume_fields(
    state: &AcousticHorizontalMomentumState<'_, CpuField<f32>>,
    inputs: AcousticHorizontalMomentumInputs<'_, CpuField<f32>>,
    expected: GridShape,
) -> AcousticHorizontalMomentumResult<()> {
    for (field, role) in [
        (
            &*state.west_east,
            AcousticHorizontalMomentumField::WestEastMomentum,
        ),
        (
            &*state.south_north,
            AcousticHorizontalMomentumField::SouthNorthMomentum,
        ),
        (
            inputs.tendencies.west_east,
            AcousticHorizontalMomentumField::WestEastTendency,
        ),
        (
            inputs.tendencies.south_north,
            AcousticHorizontalMomentumField::SouthNorthTendency,
        ),
        (
            inputs.pressure.pressure_perturbation,
            AcousticHorizontalMomentumField::PressurePerturbation,
        ),
        (
            inputs.pressure.base_pressure,
            AcousticHorizontalMomentumField::BasePressure,
        ),
        (
            inputs.pressure.geopotential_perturbation,
            AcousticHorizontalMomentumField::GeopotentialPerturbation,
        ),
        (
            inputs.pressure.pressure_point_geopotential,
            AcousticHorizontalMomentumField::PressurePointGeopotential,
        ),
        (
            inputs.pressure.full_inverse_density,
            AcousticHorizontalMomentumField::FullInverseDensity,
        ),
        (
            inputs.pressure.inverse_density_perturbation,
            AcousticHorizontalMomentumField::InverseDensityPerturbation,
        ),
        (
            inputs.moisture.west_east,
            AcousticHorizontalMomentumField::WestEastMoistureCoefficient,
        ),
        (
            inputs.moisture.south_north,
            AcousticHorizontalMomentumField::SouthNorthMoistureCoefficient,
        ),
    ] {
        validate_field_shape(field, role, expected)?;
    }
    Ok(())
}

fn validate_horizontal_fields(
    inputs: AcousticHorizontalMomentumInputs<'_, CpuField<f32>>,
    expected: GridShape,
) -> AcousticHorizontalMomentumResult<()> {
    for (field, role) in [
        (
            inputs.masses.perturbation,
            AcousticHorizontalMomentumField::PerturbationColumnMass,
        ),
        (
            inputs.masses.west_east,
            AcousticHorizontalMomentumField::WestEastColumnMass,
        ),
        (
            inputs.masses.south_north,
            AcousticHorizontalMomentumField::SouthNorthColumnMass,
        ),
        (
            inputs.masses.divergence_damping,
            AcousticHorizontalMomentumField::DivergenceDampingMass,
        ),
        (
            inputs.map_factors.west_east_x,
            AcousticHorizontalMomentumField::WestEastXMapFactor,
        ),
        (
            inputs.map_factors.west_east_y,
            AcousticHorizontalMomentumField::WestEastYMapFactor,
        ),
        (
            inputs.map_factors.south_north_x,
            AcousticHorizontalMomentumField::SouthNorthXMapFactor,
        ),
        (
            inputs.map_factors.inverse_south_north_x,
            AcousticHorizontalMomentumField::InverseSouthNorthXMapFactor,
        ),
        (
            inputs.map_factors.south_north_y,
            AcousticHorizontalMomentumField::SouthNorthYMapFactor,
        ),
    ] {
        validate_field_shape(field, role, expected)?;
    }
    Ok(())
}

fn validate_field_shape(
    field: &CpuField<f32>,
    role: AcousticHorizontalMomentumField,
    expected: GridShape,
) -> AcousticHorizontalMomentumResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(AcousticHorizontalMomentumError::FieldShapeMismatch {
            field: role,
            expected,
            actual,
        });
    }
    Ok(())
}

fn validate_coefficients(
    inputs: AcousticHorizontalMomentumInputs<'_, CpuField<f32>>,
    expected: usize,
) -> AcousticHorizontalMomentumResult<()> {
    for (values, coefficient) in [
        (
            inputs.vertical.half_level_mass_multiplier,
            AcousticHorizontalMomentumCoefficient::HalfLevelMassMultiplier,
        ),
        (
            inputs.vertical.half_level_mass_offset,
            AcousticHorizontalMomentumCoefficient::HalfLevelMassOffset,
        ),
        (
            inputs.vertical.lower_interpolation_weight,
            AcousticHorizontalMomentumCoefficient::LowerInterpolationWeight,
        ),
        (
            inputs.vertical.upper_interpolation_weight,
            AcousticHorizontalMomentumCoefficient::UpperInterpolationWeight,
        ),
        (
            inputs.vertical.inverse_half_level_spacing,
            AcousticHorizontalMomentumCoefficient::InverseHalfLevelEtaSpacing,
        ),
    ] {
        if values.len() != expected {
            return Err(AcousticHorizontalMomentumError::CoefficientLengthMismatch {
                coefficient,
                expected,
                actual: values.len(),
            });
        }
    }
    Ok(())
}
