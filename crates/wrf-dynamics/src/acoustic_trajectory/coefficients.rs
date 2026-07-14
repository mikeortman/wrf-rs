/// One-dimensional hybrid-coordinate, interpolation, and metric coefficients.
#[derive(Clone, Copy, Debug)]
pub struct AcousticTrajectoryCoefficients<'a> {
    pub(crate) half_level_mass_multiplier: &'a [f32],
    pub(crate) half_level_mass_offset: &'a [f32],
    pub(crate) full_level_mass_multiplier: &'a [f32],
    pub(crate) full_level_mass_offset: &'a [f32],
    pub(crate) hydrostatic_pressure_multiplier: &'a [f32],
    pub(crate) half_level_eta_thickness: &'a [f32],
    pub(crate) inverse_half_level_spacing: &'a [f32],
    pub(crate) inverse_full_level_spacing: &'a [f32],
    pub(crate) lower_interpolation_weight: &'a [f32],
    pub(crate) upper_interpolation_weight: &'a [f32],
}

impl<'a> AcousticTrajectoryCoefficients<'a> {
    /// Groups the ten live one-dimensional coefficient roles.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        half_level_mass_multiplier: &'a [f32],
        half_level_mass_offset: &'a [f32],
        full_level_mass_multiplier: &'a [f32],
        full_level_mass_offset: &'a [f32],
        hydrostatic_pressure_multiplier: &'a [f32],
        half_level_eta_thickness: &'a [f32],
        inverse_half_level_spacing: &'a [f32],
        inverse_full_level_spacing: &'a [f32],
        lower_interpolation_weight: &'a [f32],
        upper_interpolation_weight: &'a [f32],
    ) -> Self {
        Self {
            half_level_mass_multiplier,
            half_level_mass_offset,
            full_level_mass_multiplier,
            full_level_mass_offset,
            hydrostatic_pressure_multiplier,
            half_level_eta_thickness,
            inverse_half_level_spacing,
            inverse_full_level_spacing,
            lower_interpolation_weight,
            upper_interpolation_weight,
        }
    }
}
