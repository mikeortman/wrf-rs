/// Hybrid-coordinate column-mass coefficients used by the implicit solve.
#[derive(Clone, Copy, Debug)]
pub struct VerticalAcousticMassCoefficients<'a> {
    pub(crate) half_level_multiplier: &'a [f32],
    pub(crate) half_level_offset: &'a [f32],
    pub(crate) full_level_multiplier: &'a [f32],
    pub(crate) full_level_offset: &'a [f32],
}

impl<'a> VerticalAcousticMassCoefficients<'a> {
    /// Groups WRF `c1h`, `c2h`, `c1f`, and `c2f` without copying them.
    pub const fn new(
        half_level_multiplier: &'a [f32],
        half_level_offset: &'a [f32],
        full_level_multiplier: &'a [f32],
        full_level_offset: &'a [f32],
    ) -> Self {
        Self {
            half_level_multiplier,
            half_level_offset,
            full_level_multiplier,
            full_level_offset,
        }
    }
}
