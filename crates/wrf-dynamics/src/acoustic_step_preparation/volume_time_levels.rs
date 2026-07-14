/// Mutable previous/current volume fields switched and coupled by preparation.
pub struct AcousticStepPreparationVolumeTimeLevels<'a, Field> {
    pub(crate) previous_west_east_velocity: &'a mut Field,
    pub(crate) current_west_east_velocity: &'a mut Field,
    pub(crate) previous_south_north_velocity: &'a mut Field,
    pub(crate) current_south_north_velocity: &'a mut Field,
    pub(crate) previous_vertical_velocity: &'a mut Field,
    pub(crate) current_vertical_velocity: &'a mut Field,
    pub(crate) previous_potential_temperature: &'a mut Field,
    pub(crate) current_potential_temperature: &'a mut Field,
    pub(crate) previous_geopotential: &'a mut Field,
    pub(crate) current_geopotential: &'a mut Field,
}

impl<'a, Field> AcousticStepPreparationVolumeTimeLevels<'a, Field> {
    /// Groups WRF's `*_1` and `*_2` volume fields without allocation.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        previous_west_east_velocity: &'a mut Field,
        current_west_east_velocity: &'a mut Field,
        previous_south_north_velocity: &'a mut Field,
        current_south_north_velocity: &'a mut Field,
        previous_vertical_velocity: &'a mut Field,
        current_vertical_velocity: &'a mut Field,
        previous_potential_temperature: &'a mut Field,
        current_potential_temperature: &'a mut Field,
        previous_geopotential: &'a mut Field,
        current_geopotential: &'a mut Field,
    ) -> Self {
        Self {
            previous_west_east_velocity,
            current_west_east_velocity,
            previous_south_north_velocity,
            current_south_north_velocity,
            previous_vertical_velocity,
            current_vertical_velocity,
            previous_potential_temperature,
            current_potential_temperature,
            previous_geopotential,
            current_geopotential,
        }
    }
}
