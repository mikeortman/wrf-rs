use wrf_compute::FieldStorage;

/// Immutable base-state and pressure-gradient fields.
#[derive(Clone, Copy, Debug)]
pub struct AcousticTrajectoryPressureInputs<'a, Field: FieldStorage<f32>> {
    pub(crate) base_pressure: &'a Field,
    pub(crate) full_inverse_density: &'a Field,
    pub(crate) pressure_point_geopotential: &'a Field,
    pub(crate) base_geopotential: &'a Field,
}

impl<'a, Field: FieldStorage<f32>> AcousticTrajectoryPressureInputs<'a, Field> {
    /// Groups WRF `pb`, `alt`, `php`, and `phb` without copies.
    pub const fn new(
        base_pressure: &'a Field,
        full_inverse_density: &'a Field,
        pressure_point_geopotential: &'a Field,
        base_geopotential: &'a Field,
    ) -> Self {
        Self {
            base_pressure,
            full_inverse_density,
            pressure_point_geopotential,
            base_geopotential,
        }
    }
}
