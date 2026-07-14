use wrf_compute::CpuField;

use super::super::{
    DryBoundaryTendencies, DryBoundaryTendencyBoundaryFields, DryBoundaryTendencyError,
    DryBoundaryTendencyRegion, DryBoundaryTendencyResult, DryBoundaryTendencyTarget,
    DryBoundaryVerticalTendency,
};
use crate::SpecifiedBoundaryTendencyParameters;
use crate::specified_boundary_update::tendency_assignment::validate_cpu_tendency_assignment;

pub(super) fn validate_operation(
    tendencies: &DryBoundaryTendencies<'_, CpuField<f32>>,
    boundaries: DryBoundaryTendencyBoundaryFields<'_, CpuField<f32>>,
    vertical: &DryBoundaryVerticalTendency<'_, CpuField<f32>>,
    parameters: SpecifiedBoundaryTendencyParameters,
    region: &DryBoundaryTendencyRegion,
) -> DryBoundaryTendencyResult<()> {
    validate_assignment(
        DryBoundaryTendencyTarget::WestEastMomentum,
        tendencies.west_east_momentum,
        boundaries.west_east_momentum,
        parameters,
        region.west_east_momentum(),
    )?;
    validate_assignment(
        DryBoundaryTendencyTarget::SouthNorthMomentum,
        tendencies.south_north_momentum,
        boundaries.south_north_momentum,
        parameters,
        region.south_north_momentum(),
    )?;
    validate_assignment(
        DryBoundaryTendencyTarget::PerturbationGeopotential,
        tendencies.perturbation_geopotential,
        boundaries.perturbation_geopotential,
        parameters,
        region.perturbation_geopotential(),
    )?;
    validate_assignment(
        DryBoundaryTendencyTarget::PotentialTemperature,
        tendencies.potential_temperature,
        boundaries.potential_temperature,
        parameters,
        region.potential_temperature(),
    )?;
    validate_assignment(
        DryBoundaryTendencyTarget::PerturbationColumnMass,
        tendencies.perturbation_column_mass,
        boundaries.perturbation_column_mass,
        parameters,
        region.perturbation_column_mass(),
    )?;
    if let DryBoundaryVerticalTendency::Nested {
        tendency,
        boundaries,
    } = vertical
    {
        validate_assignment(
            DryBoundaryTendencyTarget::VerticalMomentum,
            tendency,
            *boundaries,
            parameters,
            region.vertical_momentum(),
        )?;
    }
    Ok(())
}

fn validate_assignment(
    target: DryBoundaryTendencyTarget,
    tendency: &CpuField<f32>,
    boundaries: crate::SpecifiedBoundaryTendencies<'_, CpuField<f32>>,
    parameters: SpecifiedBoundaryTendencyParameters,
    region: &crate::SpecifiedBoundaryUpdateRegion,
) -> DryBoundaryTendencyResult<()> {
    validate_cpu_tendency_assignment(tendency, &boundaries, parameters, region)
        .map_err(|source| DryBoundaryTendencyError::SpecifiedTendency { target, source })
}
