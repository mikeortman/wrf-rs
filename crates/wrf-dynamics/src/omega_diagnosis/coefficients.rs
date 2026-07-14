/// Borrowed vertical coefficients for WRF omega diagnosis.
#[derive(Clone, Copy, Debug)]
pub struct OmegaDiagnosisCoefficients<'a> {
    pub(crate) half_level_mass_multiplier: &'a [f32],
    pub(crate) half_level_mass_offset: &'a [f32],
    pub(crate) eta_layer_thickness: &'a [f32],
}

impl<'a> OmegaDiagnosisCoefficients<'a> {
    /// Groups the three vertical arrays used by `calc_ww_cp`.
    pub const fn new(
        half_level_mass_multiplier: &'a [f32],
        half_level_mass_offset: &'a [f32],
        eta_layer_thickness: &'a [f32],
    ) -> Self {
        Self {
            half_level_mass_multiplier,
            half_level_mass_offset,
            eta_layer_thickness,
        }
    }
}
