use wrf_compute::FieldStorage;

/// Mutable C-grid horizontal momentum advanced by one acoustic step.
#[derive(Debug)]
pub struct AcousticHorizontalMomentumState<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) west_east: &'a mut Field,
    pub(crate) south_north: &'a mut Field,
}

impl<'a, Field> AcousticHorizontalMomentumState<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `u` and `v` as non-aliasing mutable fields.
    pub const fn new(west_east: &'a mut Field, south_north: &'a mut Field) -> Self {
        Self {
            west_east,
            south_north,
        }
    }
}
