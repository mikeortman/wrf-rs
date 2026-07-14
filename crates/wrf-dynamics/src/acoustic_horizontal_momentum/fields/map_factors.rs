use wrf_compute::FieldStorage;

/// Borrowed C-grid map factors used by the split pressure gradients.
#[derive(Debug)]
pub struct AcousticHorizontalMapFactors<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) west_east_x: &'a Field,
    pub(crate) west_east_y: &'a Field,
    pub(crate) south_north_x: &'a Field,
    pub(crate) inverse_south_north_x: &'a Field,
    pub(crate) south_north_y: &'a Field,
}

impl<Field> Copy for AcousticHorizontalMapFactors<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticHorizontalMapFactors<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticHorizontalMapFactors<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `msfux`, `msfuy`, `msfvx`, `msfvx_inv`, and `msfvy`.
    pub const fn new(
        west_east_x: &'a Field,
        west_east_y: &'a Field,
        south_north_x: &'a Field,
        inverse_south_north_x: &'a Field,
        south_north_y: &'a Field,
    ) -> Self {
        Self {
            west_east_x,
            west_east_y,
            south_north_x,
            inverse_south_north_x,
            south_north_y,
        }
    }
}
