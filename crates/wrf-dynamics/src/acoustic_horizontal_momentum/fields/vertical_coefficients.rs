/// Borrowed hybrid-coordinate and interpolation coefficients for `advance_uv`.
#[derive(Clone, Copy, Debug)]
pub struct AcousticHorizontalVerticalCoefficients<'a> {
    pub(crate) half_level_mass_multiplier: &'a [f32],
    pub(crate) half_level_mass_offset: &'a [f32],
    pub(crate) lower_interpolation_weight: &'a [f32],
    pub(crate) upper_interpolation_weight: &'a [f32],
    pub(crate) inverse_half_level_spacing: &'a [f32],
}

impl<'a> AcousticHorizontalVerticalCoefficients<'a> {
    /// Groups WRF `c1h`, `c2h`, `fnm`, `fnp`, and `rdnw` without copies.
    pub const fn new(
        half_level_mass_multiplier: &'a [f32],
        half_level_mass_offset: &'a [f32],
        lower_interpolation_weight: &'a [f32],
        upper_interpolation_weight: &'a [f32],
        inverse_half_level_spacing: &'a [f32],
    ) -> Self {
        Self {
            half_level_mass_multiplier,
            half_level_mass_offset,
            lower_interpolation_weight,
            upper_interpolation_weight,
            inverse_half_level_spacing,
        }
    }
}
