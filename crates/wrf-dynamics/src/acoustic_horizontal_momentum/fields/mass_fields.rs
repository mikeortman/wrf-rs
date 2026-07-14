use wrf_compute::FieldStorage;

/// Borrowed column-mass roles used by horizontal acoustic momentum.
#[derive(Debug)]
pub struct AcousticHorizontalMassFields<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) perturbation: &'a Field,
    pub(crate) west_east: &'a Field,
    pub(crate) south_north: &'a Field,
    pub(crate) divergence_damping: &'a Field,
}

impl<Field> Copy for AcousticHorizontalMassFields<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticHorizontalMassFields<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticHorizontalMassFields<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `mu`, `muu`, `muv`, and `mudf` without copying fields.
    pub const fn new(
        perturbation: &'a Field,
        west_east: &'a Field,
        south_north: &'a Field,
        divergence_damping: &'a Field,
    ) -> Self {
        Self {
            perturbation,
            west_east,
            south_north,
            divergence_damping,
        }
    }
}
