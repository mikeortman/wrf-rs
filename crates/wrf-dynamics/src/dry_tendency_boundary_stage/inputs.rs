use wrf_compute::FieldStorage;

use crate::{
    DryBoundaryTendencyBoundaryFields, DryTendencyAssemblyCoefficients,
    DryTendencyAssemblyForwardTendencies, DryTendencyAssemblyMapFactors,
    DryTendencyAssemblySavedTendencies, DryTendencyAssemblyThermodynamics,
};

/// Borrowed inputs consumed by the coupled dry-tendency boundary stage.
pub struct DryTendencyBoundaryStageInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) forward: DryTendencyAssemblyForwardTendencies<'a, Field>,
    pub(crate) saved: DryTendencyAssemblySavedTendencies<'a, Field>,
    pub(crate) thermodynamics: DryTendencyAssemblyThermodynamics<'a, Field>,
    pub(crate) map_factors: DryTendencyAssemblyMapFactors<'a, Field>,
    pub(crate) coefficients: DryTendencyAssemblyCoefficients<'a>,
    pub(crate) boundaries: DryBoundaryTendencyBoundaryFields<'a, Field>,
}

impl<'a, Field> DryTendencyBoundaryStageInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups immutable inputs and the forward tendencies updated on stage one.
    pub const fn new(
        forward: DryTendencyAssemblyForwardTendencies<'a, Field>,
        saved: DryTendencyAssemblySavedTendencies<'a, Field>,
        thermodynamics: DryTendencyAssemblyThermodynamics<'a, Field>,
        map_factors: DryTendencyAssemblyMapFactors<'a, Field>,
        coefficients: DryTendencyAssemblyCoefficients<'a>,
        boundaries: DryBoundaryTendencyBoundaryFields<'a, Field>,
    ) -> Self {
        Self {
            forward,
            saved,
            thermodynamics,
            map_factors,
            coefficients,
            boundaries,
        }
    }
}
