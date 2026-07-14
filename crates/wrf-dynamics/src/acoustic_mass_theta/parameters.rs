/// Scalar controls for WRF `advance_mu_t`.
#[derive(Clone, Copy, Debug)]
pub struct AcousticMassThetaParameters {
    pub(crate) inverse_west_east_grid_spacing: f32,
    pub(crate) inverse_south_north_grid_spacing: f32,
    pub(crate) acoustic_time_step: f32,
    pub(crate) time_centering: f32,
}

impl AcousticMassThetaParameters {
    /// Creates controls corresponding to WRF `rdx`, `rdy`, `dts`, and `epssm`.
    pub const fn new(
        inverse_west_east_grid_spacing: f32,
        inverse_south_north_grid_spacing: f32,
        acoustic_time_step: f32,
        time_centering: f32,
    ) -> Self {
        Self {
            inverse_west_east_grid_spacing,
            inverse_south_north_grid_spacing,
            acoustic_time_step,
            time_centering,
        }
    }
}
