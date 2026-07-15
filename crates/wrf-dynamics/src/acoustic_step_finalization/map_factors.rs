use wrf_compute::FieldStorage;

/// Map factors read by WRF `small_step_finish`.
#[derive(Clone, Copy, Debug)]
pub struct AcousticStepFinalizationMapFactors<'a, Field: FieldStorage<f32>> {
    pub(crate) west_east_velocity_y: &'a Field,
    pub(crate) south_north_velocity_x: &'a Field,
    pub(crate) mass_point_y: &'a Field,
}

impl<'a, Field: FieldStorage<f32>> AcousticStepFinalizationMapFactors<'a, Field> {
    /// Groups live WRF `msfuy`, `msfvx`, and `msfty` roles.
    pub const fn new(
        west_east_velocity_y: &'a Field,
        south_north_velocity_x: &'a Field,
        mass_point_y: &'a Field,
    ) -> Self {
        Self {
            west_east_velocity_y,
            south_north_velocity_x,
            mass_point_y,
        }
    }
}
