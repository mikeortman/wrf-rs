use wrf_compute::FieldStorage;

/// Immutable base, staggered, full, and tendency column masses.
#[derive(Clone, Copy, Debug)]
pub struct AcousticTrajectoryMassInputs<'a, Field: FieldStorage<f32>> {
    pub(crate) base: &'a Field,
    pub(crate) west_east: &'a Field,
    pub(crate) south_north: &'a Field,
    pub(crate) full: &'a Field,
    pub(crate) tendency: &'a Field,
}

impl<'a, Field: FieldStorage<f32>> AcousticTrajectoryMassInputs<'a, Field> {
    /// Groups WRF `mub`, `muu`, `muv`, `mut`, and `mu_tend`.
    pub const fn new(
        base: &'a Field,
        west_east: &'a Field,
        south_north: &'a Field,
        full: &'a Field,
        tendency: &'a Field,
    ) -> Self {
        Self {
            base,
            west_east,
            south_north,
            full,
            tendency,
        }
    }
}
