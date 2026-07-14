/// Borrowed vertical coefficients for one momentum-coupling operation.
#[derive(Clone, Copy)]
pub struct MomentumCouplingCoefficients<'a> {
    pub(crate) half_level_mass_multiplier: &'a [f32],
    pub(crate) half_level_offset: &'a [f32],
    pub(crate) full_level_mass_multiplier: &'a [f32],
    pub(crate) full_level_offset: &'a [f32],
}

impl<'a> MomentumCouplingCoefficients<'a> {
    /// Groups WRF's `c1h`, `c2h`, `c1f`, and `c2f` arrays without copying.
    pub const fn new(
        half_level_mass_multiplier: &'a [f32],
        half_level_offset: &'a [f32],
        full_level_mass_multiplier: &'a [f32],
        full_level_offset: &'a [f32],
    ) -> Self {
        Self {
            half_level_mass_multiplier,
            half_level_offset,
            full_level_mass_multiplier,
            full_level_offset,
        }
    }
}
