use wrf_compute::FieldStorage;

/// Mutable outputs for the five always-assigned dry tendencies.
pub struct DryBoundaryTendencies<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) west_east_momentum: &'a mut Field,
    pub(crate) south_north_momentum: &'a mut Field,
    pub(crate) perturbation_geopotential: &'a mut Field,
    pub(crate) potential_temperature: &'a mut Field,
    pub(crate) perturbation_column_mass: &'a mut Field,
}

impl<'a, Field> DryBoundaryTendencies<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups mutable tendencies in WRF `spec_bdy_dry` order through MU.
    pub const fn new(
        west_east_momentum: &'a mut Field,
        south_north_momentum: &'a mut Field,
        perturbation_geopotential: &'a mut Field,
        potential_temperature: &'a mut Field,
        perturbation_column_mass: &'a mut Field,
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
