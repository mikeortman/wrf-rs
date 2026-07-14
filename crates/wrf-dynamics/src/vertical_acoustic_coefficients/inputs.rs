use wrf_compute::FieldStorage;

/// Borrowed three-dimensional and column inputs for `calc_coef_w`.
#[derive(Debug)]
pub struct VerticalAcousticCoefficientInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) full_column_mass: &'a Field,
    pub(crate) vertical_moisture_coefficient: &'a Field,
    pub(crate) pressure_coefficient: &'a Field,
}

impl<Field> Copy for VerticalAcousticCoefficientInputs<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for VerticalAcousticCoefficientInputs<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> VerticalAcousticCoefficientInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `mut`, `cqw`, and `c2a` without copying field data.
    pub const fn new(
        full_column_mass: &'a Field,
        vertical_moisture_coefficient: &'a Field,
        pressure_coefficient: &'a Field,
    ) -> Self {
        Self {
            full_column_mass,
            vertical_moisture_coefficient,
            pressure_coefficient,
        }
    }
}
