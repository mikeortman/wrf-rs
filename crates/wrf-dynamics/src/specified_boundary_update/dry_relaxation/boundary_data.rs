use wrf_compute::FieldStorage;

use crate::{SpecifiedBoundaryRelaxationBoundaryValues, SpecifiedBoundaryTendencies};

/// Boundary-file values and time tendencies for one dry prognostic field.
pub struct DryBoundaryRelaxationBoundaryData<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) values: SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>,
    pub(crate) tendencies: SpecifiedBoundaryTendencies<'a, Field>,
}

impl<Field> Clone for DryBoundaryRelaxationBoundaryData<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Field> Copy for DryBoundaryRelaxationBoundaryData<'_, Field> where Field: FieldStorage<f32> {}

impl<'a, Field> DryBoundaryRelaxationBoundaryData<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups four side values and their matching time tendencies.
    pub const fn new(
        values: SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>,
        tendencies: SpecifiedBoundaryTendencies<'a, Field>,
    ) -> Self {
        Self { values, tendencies }
    }
}
