/// Borrowed perturbation and base-state dry-air column mass.
#[derive(Clone, Copy, Debug)]
pub struct RungeKuttaPreparationMassInputs<'a, Field> {
    pub(crate) perturbation: &'a Field,
    pub(crate) base_state: &'a Field,
}

impl<'a, Field> RungeKuttaPreparationMassInputs<'a, Field> {
    /// Groups WRF `mu` and `mub` without copying either field.
    pub const fn new(perturbation: &'a Field, base_state: &'a Field) -> Self {
        Self {
            perturbation,
            base_state,
        }
    }
}
