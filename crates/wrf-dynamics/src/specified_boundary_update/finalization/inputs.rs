use wrf_compute::FieldStorage;

use super::SpecifiedBoundaryFinalizationBoundaryFields;

/// Borrowed WRF boundary arrays and normalization fields used by finalization.
#[derive(Clone, Copy)]
pub struct SpecifiedBoundaryFinalizationInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) column_mass: &'a Field,
    pub(crate) map_factor: &'a Field,
    pub(crate) column_mass_multiplier: &'a [f32],
    pub(crate) column_mass_offset: &'a [f32],
    pub(crate) boundary_values: SpecifiedBoundaryFinalizationBoundaryFields<'a, Field>,
    pub(crate) boundary_tendencies: SpecifiedBoundaryFinalizationBoundaryFields<'a, Field>,
}

impl<'a, Field> SpecifiedBoundaryFinalizationInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups the immutable inputs passed to WRF `spec_bdy_final`.
    pub const fn new(
        column_mass: &'a Field,
        map_factor: &'a Field,
        column_mass_multiplier: &'a [f32],
        column_mass_offset: &'a [f32],
        boundary_values: SpecifiedBoundaryFinalizationBoundaryFields<'a, Field>,
        boundary_tendencies: SpecifiedBoundaryFinalizationBoundaryFields<'a, Field>,
    ) -> Self {
        Self {
            column_mass,
            map_factor,
            column_mass_multiplier,
            column_mass_offset,
            boundary_values,
            boundary_tendencies,
        }
    }
}
