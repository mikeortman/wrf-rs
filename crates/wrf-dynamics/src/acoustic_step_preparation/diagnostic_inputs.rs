/// Immutable pressure, inverse-density, and omega diagnostics.
#[derive(Clone, Copy)]
pub struct AcousticStepPreparationDiagnosticInputs<'a, Field> {
    pub(crate) base_pressure: &'a Field,
    pub(crate) perturbation_pressure: &'a Field,
    pub(crate) inverse_density: &'a Field,
    pub(crate) omega: &'a Field,
}

impl<'a, Field> AcousticStepPreparationDiagnosticInputs<'a, Field> {
    /// Groups `pb`, `p`, `alt`, and `ww` without copying.
    pub const fn new(
        base_pressure: &'a Field,
        perturbation_pressure: &'a Field,
        inverse_density: &'a Field,
        omega: &'a Field,
    ) -> Self {
        Self {
            base_pressure,
            perturbation_pressure,
            inverse_density,
            omega,
        }
    }
}
