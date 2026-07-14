use wrf_compute::FieldStorage;

/// Mutable pressure-system state updated by one `calc_p_rho` equivalent call.
#[derive(Debug)]
pub struct AcousticPressureState<'a, Field: FieldStorage<f32>> {
    pub(crate) inverse_density_perturbation: &'a mut Field,
    pub(crate) pressure_perturbation: &'a mut Field,
    pub(crate) geopotential_perturbation: &'a mut Field,
    pub(crate) previous_pressure_perturbation: &'a mut Field,
}

impl<'a, Field: FieldStorage<f32>> AcousticPressureState<'a, Field> {
    /// Groups WRF `al`, `p`, `ph`, and `pm1` as non-aliasing mutable state.
    pub fn new(
        inverse_density_perturbation: &'a mut Field,
        pressure_perturbation: &'a mut Field,
        geopotential_perturbation: &'a mut Field,
        previous_pressure_perturbation: &'a mut Field,
    ) -> Self {
        Self {
            inverse_density_perturbation,
            pressure_perturbation,
            geopotential_perturbation,
            previous_pressure_perturbation,
        }
    }
}
