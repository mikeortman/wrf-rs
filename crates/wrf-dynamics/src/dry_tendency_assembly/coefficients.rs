/// Borrowed vertical coefficients for diabatic heating.
#[derive(Clone, Copy)]
pub struct DryTendencyAssemblyCoefficients<'a> {
    pub(crate) full_mass_multiplier: &'a [f32],
    pub(crate) vertical_offset: &'a [f32],
}

impl<'a> DryTendencyAssemblyCoefficients<'a> {
    /// Groups WRF's `c1` and `c2` arrays without copying.
    pub const fn new(full_mass_multiplier: &'a [f32], vertical_offset: &'a [f32]) -> Self {
        Self {
            full_mass_multiplier,
            vertical_offset,
        }
    }
}
