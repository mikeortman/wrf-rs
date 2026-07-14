/// Divergence-damping treatment for the pressure history field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticPressureDampingPhase {
    /// Initialize pressure history before the first acoustic substep.
    Initialize,
    /// Apply forward pressure weighting and advance pressure history.
    Advance,
}
