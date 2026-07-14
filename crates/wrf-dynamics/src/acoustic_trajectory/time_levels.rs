use wrf_compute::FieldStorage;

/// Mutable WRF acoustic prognostic time levels.
#[derive(Debug)]
pub struct AcousticTrajectoryTimeLevels<'a, Field: FieldStorage<f32>> {
    pub(crate) previous_west_east_momentum: &'a mut Field,
    pub(crate) current_west_east_momentum: &'a mut Field,
    pub(crate) previous_south_north_momentum: &'a mut Field,
    pub(crate) current_south_north_momentum: &'a mut Field,
    pub(crate) previous_vertical_momentum: &'a mut Field,
    pub(crate) current_vertical_momentum: &'a mut Field,
    pub(crate) previous_potential_temperature: &'a mut Field,
    pub(crate) current_potential_temperature: &'a mut Field,
    pub(crate) previous_perturbation_geopotential: &'a mut Field,
    pub(crate) current_perturbation_geopotential: &'a mut Field,
    pub(crate) previous_perturbation_column_mass: &'a mut Field,
    pub(crate) current_perturbation_column_mass: &'a mut Field,
}

impl<'a, Field: FieldStorage<f32>> AcousticTrajectoryTimeLevels<'a, Field> {
    /// Groups the mutable `*_1` and `*_2` fields without copying storage.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        previous_west_east_momentum: &'a mut Field,
        current_west_east_momentum: &'a mut Field,
        previous_south_north_momentum: &'a mut Field,
        current_south_north_momentum: &'a mut Field,
        previous_vertical_momentum: &'a mut Field,
        current_vertical_momentum: &'a mut Field,
        previous_potential_temperature: &'a mut Field,
        current_potential_temperature: &'a mut Field,
        previous_perturbation_geopotential: &'a mut Field,
        current_perturbation_geopotential: &'a mut Field,
        previous_perturbation_column_mass: &'a mut Field,
        current_perturbation_column_mass: &'a mut Field,
    ) -> Self {
        Self {
            previous_west_east_momentum,
            current_west_east_momentum,
            previous_south_north_momentum,
            current_south_north_momentum,
            previous_vertical_momentum,
            current_vertical_momentum,
            previous_potential_temperature,
            current_potential_temperature,
            previous_perturbation_geopotential,
            current_perturbation_geopotential,
            previous_perturbation_column_mass,
            current_perturbation_column_mass,
        }
    }
}
