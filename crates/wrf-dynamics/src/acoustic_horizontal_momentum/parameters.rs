use crate::{AcousticPressureMode, VerticalAcousticTopBoundary};

/// Scalar and governing-mode controls for WRF `advance_uv`.
#[derive(Clone, Copy, Debug)]
pub struct AcousticHorizontalMomentumParameters {
    pub(crate) inverse_west_east_grid_spacing: f32,
    pub(crate) inverse_south_north_grid_spacing: f32,
    pub(crate) acoustic_time_step: f32,
    pub(crate) boundary_pressure_weights: [f32; 3],
    pub(crate) divergence_damping: f32,
    pub(crate) mode: AcousticPressureMode,
    pub(crate) top_boundary: VerticalAcousticTopBoundary,
}

impl AcousticHorizontalMomentumParameters {
    /// Creates the controls corresponding to WRF `rdx`, `rdy`, `dts`,
    /// `cf1:cf3`, `emdiv`, `non_hydrostatic`, and `top_lid`.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        inverse_west_east_grid_spacing: f32,
        inverse_south_north_grid_spacing: f32,
        acoustic_time_step: f32,
        boundary_pressure_weights: [f32; 3],
        divergence_damping: f32,
        mode: AcousticPressureMode,
        top_boundary: VerticalAcousticTopBoundary,
    ) -> Self {
        Self {
            inverse_west_east_grid_spacing,
            inverse_south_north_grid_spacing,
            acoustic_time_step,
            boundary_pressure_weights,
            divergence_damping,
            mode,
            top_boundary,
        }
    }
}
