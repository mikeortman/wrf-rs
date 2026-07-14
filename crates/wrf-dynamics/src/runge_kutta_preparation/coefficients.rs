use crate::{MomentumCouplingCoefficients, OmegaDiagnosisCoefficients};

/// Borrowed vertical coefficients shared by momentum and omega diagnostics.
#[derive(Clone, Copy, Debug)]
pub struct RungeKuttaPreparationCoefficients<'a> {
    pub(crate) half_level_mass_multiplier: &'a [f32],
    pub(crate) half_level_mass_offset: &'a [f32],
    pub(crate) full_level_mass_multiplier: &'a [f32],
    pub(crate) full_level_mass_offset: &'a [f32],
    pub(crate) eta_layer_thickness: &'a [f32],
}

impl<'a> RungeKuttaPreparationCoefficients<'a> {
    /// Groups WRF `c1h`, `c2h`, `c1f`, `c2f`, and `dnw` without copying.
    pub const fn new(
        half_level_mass_multiplier: &'a [f32],
        half_level_mass_offset: &'a [f32],
        full_level_mass_multiplier: &'a [f32],
        full_level_mass_offset: &'a [f32],
        eta_layer_thickness: &'a [f32],
    ) -> Self {
        Self {
            half_level_mass_multiplier,
            half_level_mass_offset,
            full_level_mass_multiplier,
            full_level_mass_offset,
            eta_layer_thickness,
        }
    }

    pub(crate) const fn momentum(self) -> MomentumCouplingCoefficients<'a> {
        MomentumCouplingCoefficients::new(
            self.half_level_mass_multiplier,
            self.half_level_mass_offset,
            self.full_level_mass_multiplier,
            self.full_level_mass_offset,
        )
    }

    pub(crate) const fn omega(self) -> OmegaDiagnosisCoefficients<'a> {
        OmegaDiagnosisCoefficients::new(
            self.half_level_mass_multiplier,
            self.half_level_mass_offset,
            self.eta_layer_thickness,
        )
    }
}
