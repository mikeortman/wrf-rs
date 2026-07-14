use wrf_compute::FieldStorage;

use crate::{
    DryBoundaryRelaxationMassCoefficients, DryBoundaryRelaxationState,
    DryBoundaryRelaxationWorkspace, SpecifiedBoundaryRelaxationCoefficients,
    SpecifiedBoundaryRelaxationParameters,
};

use super::DryLargeStepRelaxationBoundaryValues;

/// First-substep inputs consumed only by `relax_bdy_dry`.
///
/// The prognostic state, boundary-file values, caller-owned mass-weighting
/// workspace, and relaxation coefficients are read exclusively while the
/// relaxation stage is active, so later substeps never supply them.
pub struct DryLargeStepRelaxationInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) state: DryBoundaryRelaxationState<'a, Field>,
    pub(crate) boundary_values: DryLargeStepRelaxationBoundaryValues<'a, Field>,
    pub(crate) workspace: DryBoundaryRelaxationWorkspace<'a, Field>,
    pub(crate) mass_coefficients: DryBoundaryRelaxationMassCoefficients<'a>,
    pub(crate) relaxation_coefficients: SpecifiedBoundaryRelaxationCoefficients<'a>,
    pub(crate) parameters: SpecifiedBoundaryRelaxationParameters,
}

impl<'a, Field> DryLargeStepRelaxationInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups the relaxation-only state, boundaries, workspace, and controls.
    pub const fn new(
        state: DryBoundaryRelaxationState<'a, Field>,
        boundary_values: DryLargeStepRelaxationBoundaryValues<'a, Field>,
        workspace: DryBoundaryRelaxationWorkspace<'a, Field>,
        mass_coefficients: DryBoundaryRelaxationMassCoefficients<'a>,
        relaxation_coefficients: SpecifiedBoundaryRelaxationCoefficients<'a>,
        parameters: SpecifiedBoundaryRelaxationParameters,
    ) -> Self {
        Self {
            state,
            boundary_values,
            workspace,
            mass_coefficients,
            relaxation_coefficients,
            parameters,
        }
    }
}
