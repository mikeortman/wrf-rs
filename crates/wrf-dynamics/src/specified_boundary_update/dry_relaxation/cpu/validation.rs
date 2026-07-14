use wrf_compute::{CpuField, FieldStorage, GridShape};

use super::super::{
    DryBoundaryRelaxationBoundaryFields, DryBoundaryRelaxationError, DryBoundaryRelaxationField,
    DryBoundaryRelaxationMassCoefficients, DryBoundaryRelaxationRegion,
    DryBoundaryRelaxationResult, DryBoundaryRelaxationState, DryBoundaryRelaxationTarget,
    DryBoundaryRelaxationTendencies, DryBoundaryVerticalRelaxation,
};
use super::inputs::{full_field_inputs, workspace_inputs};
use crate::specified_boundary_update::relaxation::validate_cpu_relaxation;
use crate::{
    SpecifiedBoundaryRelaxationCoefficients, SpecifiedBoundaryRelaxationParameters,
    SpecifiedBoundaryWestEastPeriodicity,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn validate_operation(
    tendencies: &DryBoundaryRelaxationTendencies<'_, CpuField<f32>>,
    state: &DryBoundaryRelaxationState<'_, CpuField<f32>>,
    boundaries: DryBoundaryRelaxationBoundaryFields<'_, CpuField<f32>>,
    vertical: &DryBoundaryVerticalRelaxation<'_, CpuField<f32>>,
    workspace: &CpuField<f32>,
    mass_coefficients: DryBoundaryRelaxationMassCoefficients<'_>,
    relaxation_coefficients: SpecifiedBoundaryRelaxationCoefficients<'_>,
    parameters: SpecifiedBoundaryRelaxationParameters,
    west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
    region: &DryBoundaryRelaxationRegion,
) -> DryBoundaryRelaxationResult<()> {
    validate_shapes(tendencies, state, vertical, workspace, region)?;
    validate_mass_coefficients(mass_coefficients, region.volume_shape())?;

    validate_relaxation(
        DryBoundaryRelaxationTarget::WestEastMomentum,
        tendencies.west_east_momentum,
        full_field_inputs(
            state.west_east_momentum,
            boundaries.west_east_momentum,
            relaxation_coefficients,
        ),
        parameters,
        west_east_periodicity,
        region.west_east_momentum(),
    )?;
    validate_relaxation(
        DryBoundaryRelaxationTarget::SouthNorthMomentum,
        tendencies.south_north_momentum,
        full_field_inputs(
            state.south_north_momentum,
            boundaries.south_north_momentum,
            relaxation_coefficients,
        ),
        parameters,
        west_east_periodicity,
        region.south_north_momentum(),
    )?;
    validate_relaxation(
        DryBoundaryRelaxationTarget::PerturbationGeopotential,
        tendencies.perturbation_geopotential,
        workspace_inputs(
            workspace,
            boundaries.perturbation_geopotential,
            relaxation_coefficients,
            region,
        ),
        parameters,
        west_east_periodicity,
        region.perturbation_geopotential(),
    )?;
    validate_relaxation(
        DryBoundaryRelaxationTarget::PotentialTemperature,
        tendencies.potential_temperature,
        workspace_inputs(
            workspace,
            boundaries.potential_temperature,
            relaxation_coefficients,
            region,
        ),
        parameters,
        west_east_periodicity,
        region.potential_temperature(),
    )?;
    validate_relaxation(
        DryBoundaryRelaxationTarget::PerturbationColumnMass,
        tendencies.perturbation_column_mass,
        full_field_inputs(
            state.perturbation_column_mass,
            boundaries.perturbation_column_mass,
            relaxation_coefficients,
        ),
        parameters,
        west_east_periodicity,
        region.perturbation_column_mass(),
    )?;
    if let DryBoundaryVerticalRelaxation::Nested {
        velocity: _,
        tendency,
        boundary,
    } = vertical
    {
        validate_relaxation(
            DryBoundaryRelaxationTarget::VerticalMomentum,
            tendency,
            workspace_inputs(workspace, *boundary, relaxation_coefficients, region),
            parameters,
            west_east_periodicity,
            region.vertical_momentum(),
        )?;
    }
    Ok(())
}

fn validate_shapes(
    tendencies: &DryBoundaryRelaxationTendencies<'_, CpuField<f32>>,
    state: &DryBoundaryRelaxationState<'_, CpuField<f32>>,
    vertical: &DryBoundaryVerticalRelaxation<'_, CpuField<f32>>,
    workspace: &CpuField<f32>,
    region: &DryBoundaryRelaxationRegion,
) -> DryBoundaryRelaxationResult<()> {
    let volume_shape = region.volume_shape();
    let horizontal_shape = region.horizontal_shape();
    for (field, role) in [
        (
            state.west_east_momentum,
            DryBoundaryRelaxationField::WestEastMomentum,
        ),
        (
            state.south_north_momentum,
            DryBoundaryRelaxationField::SouthNorthMomentum,
        ),
        (
            state.perturbation_geopotential,
            DryBoundaryRelaxationField::PerturbationGeopotential,
        ),
        (
            state.potential_temperature,
            DryBoundaryRelaxationField::PotentialTemperature,
        ),
        (
            &*tendencies.west_east_momentum,
            DryBoundaryRelaxationField::WestEastMomentumTendency,
        ),
        (
            &*tendencies.south_north_momentum,
            DryBoundaryRelaxationField::SouthNorthMomentumTendency,
        ),
        (
            &*tendencies.perturbation_geopotential,
            DryBoundaryRelaxationField::PerturbationGeopotentialTendency,
        ),
        (
            &*tendencies.potential_temperature,
            DryBoundaryRelaxationField::PotentialTemperatureTendency,
        ),
    ] {
        validate_shape(field, role, volume_shape)?;
    }
    for (field, role) in [
        (
            state.perturbation_column_mass,
            DryBoundaryRelaxationField::PerturbationColumnMass,
        ),
        (
            state.full_column_mass,
            DryBoundaryRelaxationField::FullColumnMass,
        ),
        (
            &*tendencies.perturbation_column_mass,
            DryBoundaryRelaxationField::PerturbationColumnMassTendency,
        ),
    ] {
        validate_shape(field, role, horizontal_shape)?;
    }
    validate_shape(
        workspace,
        DryBoundaryRelaxationField::MassWeightingWorkspace,
        region.workspace_shape(),
    )?;
    if let DryBoundaryVerticalRelaxation::Nested {
        velocity,
        tendency,
        boundary: _,
    } = vertical
    {
        validate_shape(
            velocity,
            DryBoundaryRelaxationField::VerticalVelocity,
            volume_shape,
        )?;
        validate_shape(
            tendency,
            DryBoundaryRelaxationField::VerticalMomentumTendency,
            volume_shape,
        )?;
    }
    Ok(())
}

fn validate_shape(
    field: &CpuField<f32>,
    role: DryBoundaryRelaxationField,
    expected: GridShape,
) -> DryBoundaryRelaxationResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(DryBoundaryRelaxationError::FieldShapeMismatch {
            field: role,
            expected,
            actual,
        });
    }
    Ok(())
}

fn validate_mass_coefficients(
    coefficients: DryBoundaryRelaxationMassCoefficients<'_>,
    shape: GridShape,
) -> DryBoundaryRelaxationResult<()> {
    for (coefficient, actual) in [
        (
            "half-level multiplier",
            coefficients.half_level_multiplier.len(),
        ),
        ("half-level offset", coefficients.half_level_offset.len()),
        (
            "full-level multiplier",
            coefficients.full_level_multiplier.len(),
        ),
        ("full-level offset", coefficients.full_level_offset.len()),
    ] {
        if actual != shape.bottom_top_points() {
            return Err(DryBoundaryRelaxationError::CoefficientLengthMismatch {
                coefficient,
                expected: shape.bottom_top_points(),
                actual,
            });
        }
    }
    Ok(())
}

fn validate_relaxation(
    target: DryBoundaryRelaxationTarget,
    tendency: &CpuField<f32>,
    inputs: crate::SpecifiedBoundaryRelaxationInputs<'_, CpuField<f32>>,
    parameters: SpecifiedBoundaryRelaxationParameters,
    west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
    region: &crate::SpecifiedBoundaryUpdateRegion,
) -> DryBoundaryRelaxationResult<()> {
    validate_cpu_relaxation(tendency, &inputs, parameters, west_east_periodicity, region)
        .map_err(|source| DryBoundaryRelaxationError::SpecifiedRelaxation { target, source })
}
