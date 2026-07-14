use wrf_compute::FieldStorage;

use crate::{
    DryBoundaryTendencyBoundaryFields, DryTendencyAssemblyCoefficients,
    DryTendencyAssemblyForwardTendencies, DryTendencyAssemblyMapFactors,
    DryTendencyAssemblyThermodynamics,
};

/// Borrowed inputs shared by every mode of the dry large-step boundary stage.
///
/// The boundary-file tendencies are read by first-substep relaxation and by
/// the closing boundary assignment, so they are supplied once here.
pub struct DryLargeStepBoundaryStageInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) forward: DryTendencyAssemblyForwardTendencies<'a, Field>,
    pub(crate) thermodynamics: DryTendencyAssemblyThermodynamics<'a, Field>,
    pub(crate) map_factors: DryTendencyAssemblyMapFactors<'a, Field>,
    pub(crate) coefficients: DryTendencyAssemblyCoefficients<'a>,
    pub(crate) boundaries: DryBoundaryTendencyBoundaryFields<'a, Field>,
}

impl<'a, Field> DryLargeStepBoundaryStageInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups the assembly inputs and the shared boundary-file tendencies.
    pub const fn new(
        forward: DryTendencyAssemblyForwardTendencies<'a, Field>,
        thermodynamics: DryTendencyAssemblyThermodynamics<'a, Field>,
        map_factors: DryTendencyAssemblyMapFactors<'a, Field>,
        coefficients: DryTendencyAssemblyCoefficients<'a>,
        boundaries: DryBoundaryTendencyBoundaryFields<'a, Field>,
    ) -> Self {
        Self {
            forward,
            thermodynamics,
            map_factors,
            coefficients,
            boundaries,
        }
    }
}
