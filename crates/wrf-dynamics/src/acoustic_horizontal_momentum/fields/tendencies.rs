use wrf_compute::FieldStorage;

/// Borrowed large-step C-grid horizontal momentum tendencies.
#[derive(Debug)]
pub struct AcousticHorizontalMomentumTendencies<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) west_east: &'a Field,
    pub(crate) south_north: &'a Field,
}

impl<Field> Copy for AcousticHorizontalMomentumTendencies<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticHorizontalMomentumTendencies<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticHorizontalMomentumTendencies<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `ru_tend` and `rv_tend` without copying field data.
    pub const fn new(west_east: &'a Field, south_north: &'a Field) -> Self {
        Self {
            west_east,
            south_north,
        }
    }
}
