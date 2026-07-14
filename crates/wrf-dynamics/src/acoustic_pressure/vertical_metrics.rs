/// Vertical-coordinate metrics used by the two dynamical modes.
#[derive(Clone, Copy, Debug)]
pub struct AcousticPressureVerticalMetrics<'a> {
    pub(crate) inverse_eta_thickness: &'a [f32],
    pub(crate) eta_thickness: &'a [f32],
}

impl<'a> AcousticPressureVerticalMetrics<'a> {
    /// Groups WRF `rdnw` and `dnw` without copying them.
    pub const fn new(inverse_eta_thickness: &'a [f32], eta_thickness: &'a [f32]) -> Self {
        Self {
            inverse_eta_thickness,
            eta_thickness,
        }
    }
}
