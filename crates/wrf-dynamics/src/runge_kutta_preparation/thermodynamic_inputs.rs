/// Borrowed fields used by inverse-density and pressure-point diagnostics.
#[derive(Clone, Copy, Debug)]
pub struct RungeKuttaPreparationThermodynamicInputs<'a, Field> {
    pub(crate) perturbation_inverse_density: &'a Field,
    pub(crate) base_state_inverse_density: &'a Field,
    pub(crate) perturbation_full_level_geopotential: &'a Field,
    pub(crate) base_state_full_level_geopotential: &'a Field,
}

impl<'a, Field> RungeKuttaPreparationThermodynamicInputs<'a, Field> {
    /// Groups WRF `al`, `alb`, `ph`, and `phb` without copying.
    pub const fn new(
        perturbation_inverse_density: &'a Field,
        base_state_inverse_density: &'a Field,
        perturbation_full_level_geopotential: &'a Field,
        base_state_full_level_geopotential: &'a Field,
    ) -> Self {
        Self {
            perturbation_inverse_density,
            base_state_inverse_density,
            perturbation_full_level_geopotential,
            base_state_full_level_geopotential,
        }
    }
}
