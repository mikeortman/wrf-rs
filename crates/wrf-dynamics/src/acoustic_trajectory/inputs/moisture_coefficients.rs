use wrf_compute::FieldStorage;

/// Immutable moisture corrections at U, V, and W points.
#[derive(Clone, Copy, Debug)]
pub struct AcousticTrajectoryMoistureCoefficients<'a, Field: FieldStorage<f32>> {
    pub(crate) west_east: &'a Field,
    pub(crate) south_north: &'a Field,
    pub(crate) vertical: &'a Field,
}

impl<'a, Field: FieldStorage<f32>> AcousticTrajectoryMoistureCoefficients<'a, Field> {
    /// Groups WRF `cqu`, `cqv`, and `cqw` without copies.
    pub const fn new(west_east: &'a Field, south_north: &'a Field, vertical: &'a Field) -> Self {
        Self {
            west_east,
            south_north,
            vertical,
        }
    }
}
