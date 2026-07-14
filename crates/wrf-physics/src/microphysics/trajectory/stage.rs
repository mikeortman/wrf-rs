/// Stable observation points in one ARW time-split microphysics step.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArwMicrophysicsStage {
    /// Diagnostic fields and pre-microphysics snapshots have been prepared.
    Prepared,
    /// The selected microphysics scheme has updated thermodynamic state.
    MicrophysicsApplied,
    /// Perturbation state and diabatic tendencies have been finalized.
    Finished,
}
