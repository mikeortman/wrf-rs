use wrf_compute::FieldStorage;

/// Borrowed boundary-file values for the four geographic sides.
pub struct SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) west: &'a Field,
    pub(crate) east: &'a Field,
    pub(crate) south: &'a Field,
    pub(crate) north: &'a Field,
}

impl<Field> Clone for SpecifiedBoundaryRelaxationBoundaryValues<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Field> Copy for SpecifiedBoundaryRelaxationBoundaryValues<'_, Field> where
    Field: FieldStorage<f32>
{
}

impl<'a, Field> SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups values in west, east, south, north order.
    pub const fn new(west: &'a Field, east: &'a Field, south: &'a Field, north: &'a Field) -> Self {
        Self {
            west,
            east,
            south,
            north,
        }
    }
}
