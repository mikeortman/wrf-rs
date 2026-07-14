use wrf_compute::FieldStorage;

/// Immutable C-grid map factors and terrain height.
#[derive(Clone, Copy, Debug)]
pub struct AcousticTrajectoryMapFactors<'a, Field: FieldStorage<f32>> {
    pub(crate) west_east_x: &'a Field,
    pub(crate) west_east_y: &'a Field,
    pub(crate) south_north_x: &'a Field,
    pub(crate) inverse_south_north_x: &'a Field,
    pub(crate) south_north_y: &'a Field,
    pub(crate) mass_point_x: &'a Field,
    pub(crate) mass_point_y: &'a Field,
    pub(crate) terrain_height: &'a Field,
}

impl<'a, Field: FieldStorage<f32>> AcousticTrajectoryMapFactors<'a, Field> {
    /// Groups the seven live map factors and terrain height.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        west_east_x: &'a Field,
        west_east_y: &'a Field,
        south_north_x: &'a Field,
        inverse_south_north_x: &'a Field,
        south_north_y: &'a Field,
        mass_point_x: &'a Field,
        mass_point_y: &'a Field,
        terrain_height: &'a Field,
    ) -> Self {
        Self {
            west_east_x,
            west_east_y,
            south_north_x,
            inverse_south_north_x,
            south_north_y,
            mass_point_x,
            mass_point_y,
            terrain_height,
        }
    }
}
