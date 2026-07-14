use wrf_compute::{FieldStorage, GridShape};

use crate::{
    AcousticFluxAccumulationError, AcousticFluxAccumulationRegion, AcousticFluxAccumulationResult,
    AcousticFluxCoefficient, AcousticFluxCurrentFields, AcousticFluxField,
    AcousticFluxLinearFields, AcousticFluxMapFactors, AcousticFluxMassFields,
    AcousticFluxRunningAverages,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn validate_operation<Field: FieldStorage<f32>>(
    averages: &AcousticFluxRunningAverages<'_, Field>,
    current: &AcousticFluxCurrentFields<'_, Field>,
    linear: &AcousticFluxLinearFields<'_, Field>,
    masses: &AcousticFluxMassFields<'_, Field>,
    map_factors: &AcousticFluxMapFactors<'_, Field>,
    column_mass_multiplier: &[f32],
    column_mass_offset: &[f32],
    region: &AcousticFluxAccumulationRegion,
) -> AcousticFluxAccumulationResult<()> {
    let shape = region.shape();
    for (field, actual) in [
        (
            AcousticFluxField::AverageWestEast,
            averages.west_east.shape(),
        ),
        (
            AcousticFluxField::AverageSouthNorth,
            averages.south_north.shape(),
        ),
        (
            AcousticFluxField::AverageVertical,
            averages.vertical.shape(),
        ),
        (
            AcousticFluxField::CurrentWestEast,
            current.west_east.shape(),
        ),
        (
            AcousticFluxField::CurrentSouthNorth,
            current.south_north.shape(),
        ),
        (AcousticFluxField::CurrentVertical, current.vertical.shape()),
        (AcousticFluxField::LinearWestEast, linear.west_east.shape()),
        (
            AcousticFluxField::LinearSouthNorth,
            linear.south_north.shape(),
        ),
        (AcousticFluxField::LinearVertical, linear.vertical.shape()),
    ] {
        require_shape(field, shape, actual)?;
    }
    let horizontal_shape = shape.horizontal_shape();
    for (field, actual) in [
        (
            AcousticFluxField::WestEastColumnMass,
            masses.west_east.shape(),
        ),
        (
            AcousticFluxField::SouthNorthColumnMass,
            masses.south_north.shape(),
        ),
        (
            AcousticFluxField::WestEastYMapFactor,
            map_factors.west_east_y.shape(),
        ),
        (
            AcousticFluxField::InverseSouthNorthXMapFactor,
            map_factors.inverse_south_north_x.shape(),
        ),
    ] {
        require_shape(field, horizontal_shape, actual)?;
    }
    require_coefficient(
        AcousticFluxCoefficient::ColumnMassMultiplier,
        shape.bottom_top_points(),
        column_mass_multiplier.len(),
    )?;
    require_coefficient(
        AcousticFluxCoefficient::ColumnMassOffset,
        shape.bottom_top_points(),
        column_mass_offset.len(),
    )
}

fn require_shape(
    field: AcousticFluxField,
    expected: GridShape,
    actual: GridShape,
) -> AcousticFluxAccumulationResult<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(AcousticFluxAccumulationError::FieldShapeMismatch {
            field,
            expected,
            actual,
        })
    }
}

fn require_coefficient(
    coefficient: AcousticFluxCoefficient,
    expected: usize,
    actual: usize,
) -> AcousticFluxAccumulationResult<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(AcousticFluxAccumulationError::CoefficientLengthMismatch {
            coefficient,
            expected,
            actual,
        })
    }
}
