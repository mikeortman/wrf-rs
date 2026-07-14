use wrf_compute::FieldStorage;

/// Borrowed large-step tendencies consumed by acoustic advancement.
///
/// WRF applies physical boundary assignments to the U/V tendencies before the
/// acoustic loop, so those two fields are mutably borrowed even though the
/// numerical advancement kernels only read them.
#[derive(Debug)]
pub struct AcousticTrajectoryTendencies<'a, Field: FieldStorage<f32>> {
    pub(crate) west_east_momentum: &'a mut Field,
    pub(crate) south_north_momentum: &'a mut Field,
    pub(crate) vertical_momentum: &'a Field,
    pub(crate) potential_temperature: &'a Field,
    pub(crate) perturbation_geopotential: &'a Field,
}

impl<'a, Field: FieldStorage<f32>> AcousticTrajectoryTendencies<'a, Field> {
    /// Groups the five volume tendencies read by the local sequence.
    pub fn new(
        west_east_momentum: &'a mut Field,
        south_north_momentum: &'a mut Field,
        vertical_momentum: &'a Field,
        potential_temperature: &'a Field,
        perturbation_geopotential: &'a Field,
    ) -> Self {
        Self {
            west_east_momentum,
            south_north_momentum,
            vertical_momentum,
            potential_temperature,
            perturbation_geopotential,
        }
    }
}
