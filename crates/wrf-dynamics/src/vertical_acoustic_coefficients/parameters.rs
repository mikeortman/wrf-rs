use crate::VerticalAcousticTopBoundary;

/// Scalar controls for WRF's vertical acoustic coefficient construction.
#[derive(Clone, Copy, Debug)]
pub struct VerticalAcousticCoefficientParameters {
    pub(crate) acoustic_time_step: f32,
    pub(crate) gravitational_acceleration: f32,
    pub(crate) vertical_offcentering: f32,
    pub(crate) top_boundary: VerticalAcousticTopBoundary,
}

impl VerticalAcousticCoefficientParameters {
    /// Creates the scalar controls corresponding to WRF `dts`, `g`, `epssm`,
    /// and `top_lid`.
    pub const fn new(
        acoustic_time_step: f32,
        gravitational_acceleration: f32,
        vertical_offcentering: f32,
        top_boundary: VerticalAcousticTopBoundary,
    ) -> Self {
        Self {
            acoustic_time_step,
            gravitational_acceleration,
            vertical_offcentering,
            top_boundary,
        }
    }

    pub(crate) fn squared_time_gravity_factor(self) -> f32 {
        let factor = 0.5_f32
            * self.acoustic_time_step
            * self.gravitational_acceleration
            * (1.0_f32 + self.vertical_offcentering);
        factor * factor
    }
}
