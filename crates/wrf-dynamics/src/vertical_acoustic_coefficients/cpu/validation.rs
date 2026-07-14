use wrf_compute::{CpuField, FieldStorage, GridShape};

use crate::{
    VerticalAcousticCoefficient, VerticalAcousticCoefficientError,
    VerticalAcousticCoefficientField, VerticalAcousticCoefficientInputs,
    VerticalAcousticCoefficientRegion, VerticalAcousticCoefficientResult,
    VerticalAcousticMassCoefficients, VerticalAcousticMetrics, VerticalAcousticSolveCoefficients,
};

pub(super) fn validate_operation(
    outputs: &VerticalAcousticSolveCoefficients<'_, CpuField<f32>>,
    inputs: VerticalAcousticCoefficientInputs<'_, CpuField<f32>>,
    mass_coefficients: VerticalAcousticMassCoefficients<'_>,
    metrics: VerticalAcousticMetrics<'_>,
    region: &VerticalAcousticCoefficientRegion,
) -> VerticalAcousticCoefficientResult<()> {
    let volume_shape = region.shape();
    for (field, role) in [
        (
            &*outputs.lower_diagonal,
            VerticalAcousticCoefficientField::LowerDiagonal,
        ),
        (
            &*outputs.inverse_eliminated_diagonal,
            VerticalAcousticCoefficientField::InverseEliminatedDiagonal,
        ),
        (
            &*outputs.upper_elimination_factor,
            VerticalAcousticCoefficientField::UpperEliminationFactor,
        ),
        (
            inputs.vertical_moisture_coefficient,
            VerticalAcousticCoefficientField::VerticalMoistureCoefficient,
        ),
        (
            inputs.pressure_coefficient,
            VerticalAcousticCoefficientField::PressureCoefficient,
        ),
    ] {
        validate_shape(field, role, volume_shape)?;
    }
    validate_shape(
        inputs.full_column_mass,
        VerticalAcousticCoefficientField::FullColumnMass,
        volume_shape.horizontal_shape(),
    )?;
    let expected = volume_shape.bottom_top_points();
    for (values, coefficient) in [
        (
            mass_coefficients.half_level_multiplier,
            VerticalAcousticCoefficient::HalfLevelMassMultiplier,
        ),
        (
            mass_coefficients.half_level_offset,
            VerticalAcousticCoefficient::HalfLevelMassOffset,
        ),
        (
            mass_coefficients.full_level_multiplier,
            VerticalAcousticCoefficient::FullLevelMassMultiplier,
        ),
        (
            mass_coefficients.full_level_offset,
            VerticalAcousticCoefficient::FullLevelMassOffset,
        ),
        (
            metrics.inverse_full_level_spacing,
            VerticalAcousticCoefficient::InverseFullLevelEtaSpacing,
        ),
        (
            metrics.inverse_half_level_spacing,
            VerticalAcousticCoefficient::InverseHalfLevelEtaSpacing,
        ),
    ] {
        if values.len() != expected {
            return Err(
                VerticalAcousticCoefficientError::CoefficientLengthMismatch {
                    coefficient,
                    expected,
                    actual: values.len(),
                },
            );
        }
    }
    Ok(())
}

fn validate_shape(
    field: &CpuField<f32>,
    role: VerticalAcousticCoefficientField,
    expected: GridShape,
) -> VerticalAcousticCoefficientResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(VerticalAcousticCoefficientError::FieldShapeMismatch {
            field: role,
            expected,
            actual,
        });
    }
    Ok(())
}
