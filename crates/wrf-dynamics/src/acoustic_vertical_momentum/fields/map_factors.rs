use wrf_compute::FieldStorage;

/// Terrain height and mass-point map factors used at the lower boundary.
#[derive(Debug)]
pub struct AcousticVerticalMapFactors<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) terrain_height: &'a Field,
    pub(crate) west_east: &'a Field,
    pub(crate) south_north: &'a Field,
}

impl<Field> Copy for AcousticVerticalMapFactors<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticVerticalMapFactors<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticVerticalMapFactors<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `ht`, `msftx`, and `msfty`.
    pub const fn new(
        terrain_height: &'a Field,
        west_east: &'a Field,
        south_north: &'a Field,
    ) -> Self {
        Self {
            terrain_height,
            west_east,
            south_north,
        }
    }
}
