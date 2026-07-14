/// Whether persistent boundary tendencies must be accumulated this substep.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DryTendencyAssemblyPhase {
    /// WRF `rk_step == 1`: add saved boundary tendencies before coupling.
    FirstSubstep,
    /// WRF `rk_step != 1`: reuse the already assembled persistent tendencies.
    LaterSubstep,
}

impl DryTendencyAssemblyPhase {
    pub(crate) const fn adds_saved_tendencies(self) -> bool {
        matches!(self, Self::FirstSubstep)
    }
}
