/// Half-level coefficients used by the linearized equation of state.
#[derive(Clone, Copy, Debug)]
pub struct AcousticPressureCoefficients<'a> {
    pub(crate) half_level_mass_multiplier: &'a [f32],
    pub(crate) half_level_mass_offset: &'a [f32],
    pub(crate) hydrostatic_pressure_multiplier: &'a [f32],
}

impl<'a> AcousticPressureCoefficients<'a> {
    /// Groups WRF `c1h`, `c2h`, and `c3h` without copying them.
    pub const fn new(
        half_level_mass_multiplier: &'a [f32],
        half_level_mass_offset: &'a [f32],
        hydrostatic_pressure_multiplier: &'a [f32],
    ) -> Self {
        Self {
            half_level_mass_multiplier,
            half_level_mass_offset,
            hydrostatic_pressure_multiplier,
        }
    }
}
