use super::{
    RungeKuttaPreparationDiagnosticOutputs, RungeKuttaPreparationMassOutputs,
    RungeKuttaPreparationMomentumOutputs,
};

/// Complete mutable output set for one Runge-Kutta preparation pass.
pub struct RungeKuttaPreparationOutputs<'a, Field> {
    pub(crate) masses: RungeKuttaPreparationMassOutputs<'a, Field>,
    pub(crate) momentum: RungeKuttaPreparationMomentumOutputs<'a, Field>,
    pub(crate) diagnostics: RungeKuttaPreparationDiagnosticOutputs<'a, Field>,
}

impl<'a, Field> RungeKuttaPreparationOutputs<'a, Field> {
    /// Groups all outputs while Rust prevents mutable aliasing between them.
    pub fn new(
        masses: RungeKuttaPreparationMassOutputs<'a, Field>,
        momentum: RungeKuttaPreparationMomentumOutputs<'a, Field>,
        diagnostics: RungeKuttaPreparationDiagnosticOutputs<'a, Field>,
    ) -> Self {
        Self {
            masses,
            momentum,
            diagnostics,
        }
    }
}
