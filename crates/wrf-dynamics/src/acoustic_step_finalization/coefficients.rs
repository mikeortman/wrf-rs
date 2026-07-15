/// Borrowed hybrid-coordinate coefficients used while uncoupling fields.
#[derive(Clone, Copy, Debug)]
pub struct AcousticStepFinalizationCoefficients<'a> {
    pub(crate) half_level_mass_multiplier: &'a [f32],
    pub(crate) half_level_mass_offset: &'a [f32],
    pub(crate) full_level_mass_multiplier: &'a [f32],
    pub(crate) full_level_mass_offset: &'a [f32],
}

impl<'a> AcousticStepFinalizationCoefficients<'a> {
    /// Groups WRF `c1h`, `c2h`, `c1f`, and `c2f` without copying.
    pub const fn new(
        half_level_mass_multiplier: &'a [f32],
        half_level_mass_offset: &'a [f32],
        full_level_mass_multiplier: &'a [f32],
        full_level_mass_offset: &'a [f32],
    ) -> Self {
        Self {
            half_level_mass_multiplier,
            half_level_mass_offset,
            full_level_mass_multiplier,
            full_level_mass_offset,
        }
    }
}
