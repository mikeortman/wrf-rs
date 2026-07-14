use wrf_compute::FieldStorage;

/// Borrowed boundary-file tendencies for the four geographic sides.
pub struct SpecifiedBoundaryTendencies<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) west: &'a Field,
    pub(crate) east: &'a Field,
    pub(crate) south: &'a Field,
    pub(crate) north: &'a Field,
}

impl<Field> Clone for SpecifiedBoundaryTendencies<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Field> Copy for SpecifiedBoundaryTendencies<'_, Field> where Field: FieldStorage<f32> {}

impl<'a, Field> SpecifiedBoundaryTendencies<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups tendencies in west, east, south, north order.
    pub const fn new(west: &'a Field, east: &'a Field, south: &'a Field, north: &'a Field) -> Self {
        Self {
            west,
            east,
            south,
            north,
        }
    }
}
