use wrf_compute::{CpuField, FieldStorage, GridShape};

use crate::{
    AcousticPressureCoefficient, AcousticPressureCoefficients, AcousticPressureError,
    AcousticPressureField, AcousticPressureMasses, AcousticPressureRegion, AcousticPressureResult,
    AcousticPressureState, AcousticPressureThermodynamics, AcousticPressureVerticalMetrics,
};

pub(super) fn validate_operation(
    state: &AcousticPressureState<'_, CpuField<f32>>,
    thermodynamics: &AcousticPressureThermodynamics<'_, CpuField<f32>>,
    masses: &AcousticPressureMasses<'_, CpuField<f32>>,
    coefficients: AcousticPressureCoefficients<'_>,
    vertical_metrics: AcousticPressureVerticalMetrics<'_>,
    region: &AcousticPressureRegion,
) -> AcousticPressureResult<()> {
    let volume_shape = region.shape();
    for (field, role) in [
        (
            &*state.inverse_density_perturbation,
            AcousticPressureField::InverseDensityPerturbation,
        ),
        (
            &*state.pressure_perturbation,
            AcousticPressureField::PressurePerturbation,
        ),
        (
            &*state.geopotential_perturbation,
            AcousticPressureField::GeopotentialPerturbation,
        ),
        (
            &*state.previous_pressure_perturbation,
            AcousticPressureField::PreviousPressurePerturbation,
        ),
        (
            thermodynamics.full_inverse_density,
            AcousticPressureField::FullInverseDensity,
        ),
        (
            thermodynamics.current_temperature,
            AcousticPressureField::CurrentTemperature,
        ),
        (
            thermodynamics.reference_temperature_perturbation,
            AcousticPressureField::ReferenceTemperaturePerturbation,
        ),
        (
            thermodynamics.pressure_coefficient,
            AcousticPressureField::PressureCoefficient,
        ),
    ] {
        validate_shape(field, role, volume_shape)?;
    }
    let horizontal_shape = volume_shape.horizontal_shape();
    for (field, role) in [
        (
            masses.perturbation,
            AcousticPressureField::PerturbationColumnMass,
        ),
        (masses.full, AcousticPressureField::FullColumnMass),
    ] {
        validate_shape(field, role, horizontal_shape)?;
    }
    let expected = volume_shape.bottom_top_points();
    for (values, coefficient) in [
        (
            coefficients.half_level_mass_multiplier,
            AcousticPressureCoefficient::HalfLevelMassMultiplier,
        ),
        (
            coefficients.half_level_mass_offset,
            AcousticPressureCoefficient::HalfLevelMassOffset,
        ),
        (
            coefficients.hydrostatic_pressure_multiplier,
            AcousticPressureCoefficient::HydrostaticPressureMultiplier,
        ),
        (
            vertical_metrics.inverse_eta_thickness,
            AcousticPressureCoefficient::InverseEtaThickness,
        ),
        (
            vertical_metrics.eta_thickness,
            AcousticPressureCoefficient::EtaThickness,
        ),
    ] {
        if values.len() != expected {
            return Err(AcousticPressureError::CoefficientLengthMismatch {
                coefficient,
                expected,
                actual: values.len(),
            });
        }
    }
    Ok(())
}

fn validate_shape(
    field: &CpuField<f32>,
    role: AcousticPressureField,
    expected: GridShape,
) -> AcousticPressureResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(AcousticPressureError::FieldShapeMismatch {
            field: role,
            expected,
            actual,
        });
    }
    Ok(())
}
