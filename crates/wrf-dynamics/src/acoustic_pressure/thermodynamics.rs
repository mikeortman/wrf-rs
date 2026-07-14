use wrf_compute::FieldStorage;

/// Immutable thermodynamic fields used by the linearized equation of state.
#[derive(Clone, Copy, Debug)]
pub struct AcousticPressureThermodynamics<'a, Field: FieldStorage<f32>> {
    pub(crate) full_inverse_density: &'a Field,
    pub(crate) current_temperature: &'a Field,
    pub(crate) reference_temperature_perturbation: &'a Field,
    pub(crate) pressure_coefficient: &'a Field,
}

impl<'a, Field: FieldStorage<f32>> AcousticPressureThermodynamics<'a, Field> {
    /// Groups WRF `alt`, `t_2`, `t_1`, and `c2a` without copying them.
    pub const fn new(
        full_inverse_density: &'a Field,
        current_temperature: &'a Field,
        reference_temperature_perturbation: &'a Field,
        pressure_coefficient: &'a Field,
    ) -> Self {
        Self {
            full_inverse_density,
            current_temperature,
            reference_temperature_perturbation,
            pressure_coefficient,
        }
    }
}
