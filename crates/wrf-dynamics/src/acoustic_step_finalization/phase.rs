/// Runge-Kutta phase controlling the diabatic-heating correction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticStepFinalizationPhase {
    /// A predictor/corrector stage before the final Runge-Kutta stage.
    Intermediate,
    /// The final Runge-Kutta stage removes time-split microphysics heating.
    Final,
}
