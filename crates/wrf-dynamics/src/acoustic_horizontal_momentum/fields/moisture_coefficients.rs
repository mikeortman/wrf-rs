use wrf_compute::FieldStorage;

/// Borrowed moisture corrections at horizontal momentum points.
#[derive(Debug)]
pub struct AcousticHorizontalMoistureCoefficients<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) west_east: &'a Field,
    pub(crate) south_north: &'a Field,
}

impl<Field> Copy for AcousticHorizontalMoistureCoefficients<'_, Field> where Field: FieldStorage<f32>
{}

impl<Field> Clone for AcousticHorizontalMoistureCoefficients<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticHorizontalMoistureCoefficients<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `cqu` and `cqv` without copying field data.
    pub const fn new(west_east: &'a Field, south_north: &'a Field) -> Self {
        Self {
            west_east,
            south_north,
        }
    }
}
