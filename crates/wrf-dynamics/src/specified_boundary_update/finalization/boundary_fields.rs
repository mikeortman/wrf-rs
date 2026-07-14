use wrf_compute::FieldStorage;

/// Borrowed values for the four WRF specified-boundary sides.
pub struct SpecifiedBoundaryFinalizationBoundaryFields<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) west: &'a Field,
    pub(crate) east: &'a Field,
    pub(crate) south: &'a Field,
    pub(crate) north: &'a Field,
}

impl<Field> Clone for SpecifiedBoundaryFinalizationBoundaryFields<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Field> Copy for SpecifiedBoundaryFinalizationBoundaryFields<'_, Field> where
    Field: FieldStorage<f32>
{
}

impl<'a, Field> SpecifiedBoundaryFinalizationBoundaryFields<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups boundary arrays in geographic order.
    pub const fn new(west: &'a Field, east: &'a Field, south: &'a Field, north: &'a Field) -> Self {
        Self {
            west,
            east,
            south,
            north,
        }
    }
}
