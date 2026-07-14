use wrf_compute::FieldStorage;

/// Three-dimensional coefficients and factors for vertical momentum advancement.
#[derive(Debug)]
pub struct AcousticVerticalSolveInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) pressure_coefficient: &'a Field,
    pub(crate) moisture_coefficient: &'a Field,
    pub(crate) inverse_density: &'a Field,
    pub(crate) lower_diagonal: &'a Field,
    pub(crate) inverse_eliminated_diagonal: &'a Field,
    pub(crate) upper_elimination_factor: &'a Field,
}

impl<Field> Copy for AcousticVerticalSolveInputs<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticVerticalSolveInputs<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticVerticalSolveInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `c2a`, `cqw`, `alt`, `a`, `alpha`, and `gamma`.
    pub const fn new(
        pressure_coefficient: &'a Field,
        moisture_coefficient: &'a Field,
        inverse_density: &'a Field,
        lower_diagonal: &'a Field,
        inverse_eliminated_diagonal: &'a Field,
        upper_elimination_factor: &'a Field,
    ) -> Self {
        Self {
            pressure_coefficient,
            moisture_coefficient,
            inverse_density,
            lower_diagonal,
            inverse_eliminated_diagonal,
            upper_elimination_factor,
        }
    }
}
