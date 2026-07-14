/// Scalar controls for WRF `advance_w`.
#[derive(Clone, Copy, Debug)]
pub struct AcousticVerticalParameters {
    pub(crate) inverse_west_east_grid_spacing: f32,
    pub(crate) inverse_south_north_grid_spacing: f32,
    pub(crate) acoustic_time_step: f32,
    pub(crate) base_potential_temperature: f32,
    pub(crate) time_centering: f32,
    pub(crate) surface_velocity_weights: [f32; 3],
}

impl AcousticVerticalParameters {
    /// Creates controls corresponding to WRF `rdx`, `rdy`, `dts`, `t0`,
    /// `epssm`, and `cf1..cf3`.
    pub const fn new(
        inverse_west_east_grid_spacing: f32,
        inverse_south_north_grid_spacing: f32,
        acoustic_time_step: f32,
        base_potential_temperature: f32,
        time_centering: f32,
        surface_velocity_weights: [f32; 3],
    ) -> Self {
        Self {
            inverse_west_east_grid_spacing,
            inverse_south_north_grid_spacing,
            acoustic_time_step,
            base_potential_temperature,
            time_centering,
            surface_velocity_weights,
        }
    }
}
