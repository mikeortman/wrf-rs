use wrf_compute::FieldStorage;

/// Borrowed base, staggered, and tendency column-mass fields.
#[derive(Debug)]
pub struct AcousticMassThetaMassInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) base_full: &'a Field,
    pub(crate) west_east: &'a Field,
    pub(crate) south_north: &'a Field,
    pub(crate) tendency: &'a Field,
}

impl<Field> Copy for AcousticMassThetaMassInputs<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticMassThetaMassInputs<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticMassThetaMassInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `mut`, `muu`, `muv`, and `mu_tend` without copying data.
    pub const fn new(
        base_full: &'a Field,
        west_east: &'a Field,
        south_north: &'a Field,
        tendency: &'a Field,
    ) -> Self {
        Self {
            base_full,
            west_east,
            south_north,
            tendency,
        }
    }
}
