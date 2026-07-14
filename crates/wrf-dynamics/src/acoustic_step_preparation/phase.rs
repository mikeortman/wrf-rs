/// Runge–Kutta phase controlling time-level switching.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticStepPreparationPhase {
    /// WRF `rk_step == 1`: replace previous levels with current levels.
    FirstSubstep,
    /// WRF `rk_step != 1`: form perturbations relative to previous levels.
    LaterSubstep,
}

impl AcousticStepPreparationPhase {
    pub(crate) const fn switches_time_levels(self) -> bool {
        matches!(self, Self::FirstSubstep)
    }
}
