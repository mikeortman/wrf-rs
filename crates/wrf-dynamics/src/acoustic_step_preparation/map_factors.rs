/// Map factors actually read by WRF `small_step_prep`.
#[derive(Clone, Copy)]
pub struct AcousticStepPreparationMapFactors<'a, Field> {
    pub(crate) west_east_momentum_south_north: &'a Field,
    pub(crate) inverse_south_north_momentum_west_east: &'a Field,
    pub(crate) mass_point_south_north: &'a Field,
}

impl<'a, Field> AcousticStepPreparationMapFactors<'a, Field> {
    /// Groups `msfuy`, `msfvx_inv`, and `msfty`.
    pub const fn new(
        west_east_momentum_south_north: &'a Field,
        inverse_south_north_momentum_west_east: &'a Field,
        mass_point_south_north: &'a Field,
    ) -> Self {
        Self {
            west_east_momentum_south_north,
            inverse_south_north_momentum_west_east,
            mass_point_south_north,
        }
    }
}
