/// Borrowed hybrid-coordinate and vertical-metric coefficients.
#[derive(Clone, Copy, Debug)]
pub struct AcousticMassThetaVerticalCoefficients<'a> {
    pub(crate) half_level_mass_multiplier: &'a [f32],
    pub(crate) half_level_mass_offset: &'a [f32],
    pub(crate) half_level_eta_thickness: &'a [f32],
    pub(crate) lower_interpolation_weight: &'a [f32],
    pub(crate) upper_interpolation_weight: &'a [f32],
    pub(crate) inverse_half_level_spacing: &'a [f32],
}

impl<'a> AcousticMassThetaVerticalCoefficients<'a> {
    /// Groups WRF `c1h`, `c2h`, `dnw`, `fnm`, `fnp`, and `rdnw`.
    pub const fn new(
        half_level_mass_multiplier: &'a [f32],
        half_level_mass_offset: &'a [f32],
        half_level_eta_thickness: &'a [f32],
        lower_interpolation_weight: &'a [f32],
        upper_interpolation_weight: &'a [f32],
        inverse_half_level_spacing: &'a [f32],
    ) -> Self {
        Self {
            half_level_mass_multiplier,
            half_level_mass_offset,
            half_level_eta_thickness,
            lower_interpolation_weight,
            upper_interpolation_weight,
            inverse_half_level_spacing,
        }
    }
}
