use wrf_compute::FieldStorage;

use crate::{
    AcousticVerticalBoundaryPolicy, AcousticVerticalCoefficient, AcousticVerticalError,
    AcousticVerticalField, AcousticVerticalInputs, AcousticVerticalRegion, AcousticVerticalResult,
    AcousticVerticalState, AcousticVerticalWorkspace,
};

use super::super::region::AcousticVerticalActiveRanges;

pub(super) struct OperationValidation;

impl OperationValidation {
    pub(super) fn validate<Field>(
        state: &AcousticVerticalState<'_, Field>,
        workspace: &AcousticVerticalWorkspace<'_, Field>,
        inputs: AcousticVerticalInputs<'_, Field>,
        boundary_policy: AcousticVerticalBoundaryPolicy,
        region: &AcousticVerticalRegion,
    ) -> AcousticVerticalResult<AcousticVerticalActiveRanges>
    where
        Field: FieldStorage<f32>,
    {
        let volume_shape = region.shape();
        let horizontal_shape = volume_shape.horizontal_shape();
        for (field, actual) in [
            (
                AcousticVerticalField::VerticalMomentum,
                state.vertical_momentum.shape(),
            ),
            (
                AcousticVerticalField::PerturbationGeopotential,
                state.perturbation_geopotential.shape(),
            ),
            (
                AcousticVerticalField::TimeAveragedThermodynamics,
                state.time_averaged_thermodynamics.shape(),
            ),
            (
                AcousticVerticalField::GeopotentialRightHandSide,
                workspace.geopotential_right_hand_side.shape(),
            ),
            (
                AcousticVerticalField::VerticalMomentumTendency,
                inputs.momentum.tendency.shape(),
            ),
            (
                AcousticVerticalField::VerticalMassFlux,
                inputs.momentum.vertical_mass_flux.shape(),
            ),
            (
                AcousticVerticalField::SavedVerticalMomentum,
                inputs.momentum.saved_vertical_momentum.shape(),
            ),
            (
                AcousticVerticalField::WestEastMomentum,
                inputs.momentum.west_east_momentum.shape(),
            ),
            (
                AcousticVerticalField::SouthNorthMomentum,
                inputs.momentum.south_north_momentum.shape(),
            ),
            (
                AcousticVerticalField::PotentialTemperature,
                inputs.thermodynamics.potential_temperature.shape(),
            ),
            (
                AcousticVerticalField::SavedPotentialTemperature,
                inputs.thermodynamics.saved_potential_temperature.shape(),
            ),
            (
                AcousticVerticalField::SavedPerturbationGeopotential,
                inputs.geopotential.saved_perturbation.shape(),
            ),
            (
                AcousticVerticalField::BaseGeopotential,
                inputs.geopotential.base.shape(),
            ),
            (
                AcousticVerticalField::PerturbationGeopotentialTendency,
                inputs.geopotential.tendency.shape(),
            ),
            (
                AcousticVerticalField::PressureCoefficient,
                inputs.solve.pressure_coefficient.shape(),
            ),
            (
                AcousticVerticalField::MoistureCoefficient,
                inputs.solve.moisture_coefficient.shape(),
            ),
            (
                AcousticVerticalField::InverseDensity,
                inputs.solve.inverse_density.shape(),
            ),
            (
                AcousticVerticalField::LowerDiagonal,
                inputs.solve.lower_diagonal.shape(),
            ),
            (
                AcousticVerticalField::InverseEliminatedDiagonal,
                inputs.solve.inverse_eliminated_diagonal.shape(),
            ),
            (
                AcousticVerticalField::UpperEliminationFactor,
                inputs.solve.upper_elimination_factor.shape(),
            ),
        ] {
            validate_shape(field, volume_shape, actual)?;
        }
        for (field, actual) in [
            (
                AcousticVerticalField::FullColumnMass,
                inputs.mass.full_column_mass.shape(),
            ),
            (
                AcousticVerticalField::TimeCenteredColumnMass,
                inputs.mass.time_centered_column_mass.shape(),
            ),
            (
                AcousticVerticalField::CoupledColumnMass,
                inputs.mass.coupled_column_mass.shape(),
            ),
            (
                AcousticVerticalField::TerrainHeight,
                inputs.maps.terrain_height.shape(),
            ),
            (
                AcousticVerticalField::MassPointWestEastMapFactor,
                inputs.maps.west_east.shape(),
            ),
            (
                AcousticVerticalField::MassPointSouthNorthMapFactor,
                inputs.maps.south_north.shape(),
            ),
        ] {
            validate_shape(field, horizontal_shape, actual)?;
        }
        let expected = volume_shape.bottom_top_points();
        for (coefficient, actual) in [
            (
                AcousticVerticalCoefficient::HalfLevelMassMultiplier,
                inputs.vertical.half_level_mass_multiplier.len(),
            ),
            (
                AcousticVerticalCoefficient::HalfLevelMassOffset,
                inputs.vertical.half_level_mass_offset.len(),
            ),
            (
                AcousticVerticalCoefficient::FullLevelMassMultiplier,
                inputs.vertical.full_level_mass_multiplier.len(),
            ),
            (
                AcousticVerticalCoefficient::FullLevelMassOffset,
                inputs.vertical.full_level_mass_offset.len(),
            ),
            (
                AcousticVerticalCoefficient::UpperInterpolationWeight,
                inputs.vertical.upper_interpolation_weight.len(),
            ),
            (
                AcousticVerticalCoefficient::LowerInterpolationWeight,
                inputs.vertical.lower_interpolation_weight.len(),
            ),
            (
                AcousticVerticalCoefficient::InverseHalfLevelSpacing,
                inputs.vertical.inverse_half_level_spacing.len(),
            ),
            (
                AcousticVerticalCoefficient::InverseFullLevelSpacing,
                inputs.vertical.inverse_full_level_spacing.len(),
            ),
        ] {
            if actual != expected {
                return Err(AcousticVerticalError::CoefficientLengthMismatch {
                    coefficient,
                    expected,
                    actual,
                });
            }
        }
        region.active_ranges(boundary_policy)
    }
}

fn validate_shape(
    field: AcousticVerticalField,
    expected: wrf_compute::GridShape,
    actual: wrf_compute::GridShape,
) -> AcousticVerticalResult<()> {
    if actual != expected {
        return Err(AcousticVerticalError::FieldShapeMismatch {
            field,
            expected,
            actual,
        });
    }
    Ok(())
}
