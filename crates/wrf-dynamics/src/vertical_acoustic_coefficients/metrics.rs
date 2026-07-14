/// Inverse eta-coordinate spacings used by `calc_coef_w`.
#[derive(Clone, Copy, Debug)]
pub struct VerticalAcousticMetrics<'a> {
    pub(crate) inverse_full_level_spacing: &'a [f32],
    pub(crate) inverse_half_level_spacing: &'a [f32],
}

impl<'a> VerticalAcousticMetrics<'a> {
    /// Groups WRF `rdn` and `rdnw` without copying them.
    pub const fn new(
        inverse_full_level_spacing: &'a [f32],
        inverse_half_level_spacing: &'a [f32],
    ) -> Self {
        Self {
            inverse_full_level_spacing,
            inverse_half_level_spacing,
        }
    }
}
