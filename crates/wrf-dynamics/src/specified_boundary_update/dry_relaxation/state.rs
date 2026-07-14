use wrf_compute::FieldStorage;

/// Immutable dry state used by WRF boundary relaxation.
pub struct DryBoundaryRelaxationState<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) west_east_momentum: &'a Field,
    pub(crate) south_north_momentum: &'a Field,
    pub(crate) perturbation_geopotential: &'a Field,
    pub(crate) potential_temperature: &'a Field,
    pub(crate) perturbation_column_mass: &'a Field,
    pub(crate) full_column_mass: &'a Field,
}

impl<Field> Clone for DryBoundaryRelaxationState<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Field> Copy for DryBoundaryRelaxationState<'_, Field> where Field: FieldStorage<f32> {}

impl<'a, Field> DryBoundaryRelaxationState<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups `ru`, `rv`, `ph`, `t`, `mu`, and `mut` without copying storage.
    pub const fn new(
        west_east_momentum: &'a Field,
        south_north_momentum: &'a Field,
        perturbation_geopotential: &'a Field,
        potential_temperature: &'a Field,
        perturbation_column_mass: &'a Field,
        full_column_mass: &'a Field,
    ) -> Self {
        Self {
            west_east_momentum,
            south_north_momentum,
            perturbation_geopotential,
            potential_temperature,
            perturbation_column_mass,
            full_column_mass,
        }
    }
}
