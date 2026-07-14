use wrf_compute::FieldStorage;

use super::{
    SpecifiedBoundaryRelaxationBoundaryValues, SpecifiedBoundaryRelaxationCoefficients,
    SpecifiedBoundaryRelaxationField,
};
use crate::SpecifiedBoundaryTendencies;

/// Borrowed state required by WRF's specified-boundary relaxation stencil.
pub struct SpecifiedBoundaryRelaxationInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) field: SpecifiedBoundaryRelaxationField<'a, Field>,
    pub(crate) boundary_values: SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>,
    pub(crate) boundary_tendencies: SpecifiedBoundaryTendencies<'a, Field>,
    pub(crate) coefficients: SpecifiedBoundaryRelaxationCoefficients<'a>,
}

impl<'a, Field> SpecifiedBoundaryRelaxationInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups the model field, boundary states, boundary tendencies, and weights.
    pub const fn new(
        field: SpecifiedBoundaryRelaxationField<'a, Field>,
        boundary_values: SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>,
        boundary_tendencies: SpecifiedBoundaryTendencies<'a, Field>,
        coefficients: SpecifiedBoundaryRelaxationCoefficients<'a>,
    ) -> Self {
        Self {
            field,
            boundary_values,
            boundary_tendencies,
            coefficients,
        }
    }
}
