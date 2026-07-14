use wrf_compute::FieldStorage;

use crate::SpecifiedBoundaryTendencies;

/// Boundary-file tendencies for the five always-assigned dry fields.
pub struct DryBoundaryTendencyBoundaryFields<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) west_east_momentum: SpecifiedBoundaryTendencies<'a, Field>,
    pub(crate) south_north_momentum: SpecifiedBoundaryTendencies<'a, Field>,
    pub(crate) perturbation_geopotential: SpecifiedBoundaryTendencies<'a, Field>,
    pub(crate) potential_temperature: SpecifiedBoundaryTendencies<'a, Field>,
    pub(crate) perturbation_column_mass: SpecifiedBoundaryTendencies<'a, Field>,
}

impl<Field> Clone for DryBoundaryTendencyBoundaryFields<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Field> Copy for DryBoundaryTendencyBoundaryFields<'_, Field> where Field: FieldStorage<f32> {}

impl<'a, Field> DryBoundaryTendencyBoundaryFields<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups boundary tendencies in WRF `spec_bdy_dry` order through MU.
    pub const fn new(
        west_east_momentum: SpecifiedBoundaryTendencies<'a, Field>,
        south_north_momentum: SpecifiedBoundaryTendencies<'a, Field>,
        perturbation_geopotential: SpecifiedBoundaryTendencies<'a, Field>,
        potential_temperature: SpecifiedBoundaryTendencies<'a, Field>,
        perturbation_column_mass: SpecifiedBoundaryTendencies<'a, Field>,
    ) -> Self {
        Self {
            west_east_momentum,
            south_north_momentum,
            perturbation_geopotential,
            potential_temperature,
            perturbation_column_mass,
        }
    }
}
