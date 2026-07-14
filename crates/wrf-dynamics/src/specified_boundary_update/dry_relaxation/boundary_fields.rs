use wrf_compute::FieldStorage;

use super::DryBoundaryRelaxationBoundaryData;

/// Boundary data for the five always-relaxed dry fields.
pub struct DryBoundaryRelaxationBoundaryFields<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) west_east_momentum: DryBoundaryRelaxationBoundaryData<'a, Field>,
    pub(crate) south_north_momentum: DryBoundaryRelaxationBoundaryData<'a, Field>,
    pub(crate) perturbation_geopotential: DryBoundaryRelaxationBoundaryData<'a, Field>,
    pub(crate) potential_temperature: DryBoundaryRelaxationBoundaryData<'a, Field>,
    pub(crate) perturbation_column_mass: DryBoundaryRelaxationBoundaryData<'a, Field>,
}

impl<Field> Clone for DryBoundaryRelaxationBoundaryFields<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Field> Copy for DryBoundaryRelaxationBoundaryFields<'_, Field> where Field: FieldStorage<f32> {}

impl<'a, Field> DryBoundaryRelaxationBoundaryFields<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups fields in WRF `relax_bdy_dry` call order through column mass.
    pub const fn new(
        west_east_momentum: DryBoundaryRelaxationBoundaryData<'a, Field>,
        south_north_momentum: DryBoundaryRelaxationBoundaryData<'a, Field>,
        perturbation_geopotential: DryBoundaryRelaxationBoundaryData<'a, Field>,
        potential_temperature: DryBoundaryRelaxationBoundaryData<'a, Field>,
        perturbation_column_mass: DryBoundaryRelaxationBoundaryData<'a, Field>,
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
