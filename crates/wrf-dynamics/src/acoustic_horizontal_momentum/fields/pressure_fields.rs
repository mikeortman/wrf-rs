use wrf_compute::FieldStorage;

/// Borrowed pressure, geopotential, and inverse-density fields in the split gradient.
#[derive(Debug)]
pub struct AcousticHorizontalPressureFields<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) pressure_perturbation: &'a Field,
    pub(crate) base_pressure: &'a Field,
    pub(crate) geopotential_perturbation: &'a Field,
    pub(crate) pressure_point_geopotential: &'a Field,
    pub(crate) full_inverse_density: &'a Field,
    pub(crate) inverse_density_perturbation: &'a Field,
}

impl<Field> Copy for AcousticHorizontalPressureFields<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticHorizontalPressureFields<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticHorizontalPressureFields<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `p`, `pb`, `ph`, `php`, `alt`, and `al` without copies.
    pub const fn new(
        pressure_perturbation: &'a Field,
        base_pressure: &'a Field,
        geopotential_perturbation: &'a Field,
        pressure_point_geopotential: &'a Field,
        full_inverse_density: &'a Field,
        inverse_density_perturbation: &'a Field,
    ) -> Self {
        Self {
            pressure_perturbation,
            base_pressure,
            geopotential_perturbation,
            pressure_point_geopotential,
            full_inverse_density,
            inverse_density_perturbation,
        }
    }
}
