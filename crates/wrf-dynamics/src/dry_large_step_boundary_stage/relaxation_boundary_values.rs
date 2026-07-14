use wrf_compute::FieldStorage;

use crate::SpecifiedBoundaryRelaxationBoundaryValues;

/// Boundary-file values for the five always-relaxed dry fields.
///
/// Relaxation pairs these values with the same boundary-file tendencies that
/// `spec_bdy_dry` later assigns, so the stage accepts the tendencies once and
/// zips them with these values internally.
pub struct DryLargeStepRelaxationBoundaryValues<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) west_east_momentum: SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>,
    pub(crate) south_north_momentum: SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>,
    pub(crate) perturbation_geopotential: SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>,
    pub(crate) potential_temperature: SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>,
    pub(crate) perturbation_column_mass: SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>,
}

impl<Field> Clone for DryLargeStepRelaxationBoundaryValues<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Field> Copy for DryLargeStepRelaxationBoundaryValues<'_, Field> where Field: FieldStorage<f32> {}

impl<'a, Field> DryLargeStepRelaxationBoundaryValues<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups boundary values in WRF `relax_bdy_dry` order through column mass.
    pub const fn new(
        west_east_momentum: SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>,
        south_north_momentum: SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>,
        perturbation_geopotential: SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>,
        potential_temperature: SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>,
        perturbation_column_mass: SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>,
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
