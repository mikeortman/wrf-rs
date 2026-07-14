use wrf_compute::CpuField;

/// Borrowed ARW model state consumed by one time-split microphysics step.
pub struct ArwMicrophysicsState<'a> {
    pub(crate) perturbation_potential_temperature: &'a mut CpuField<f32>,
    pub(crate) moisture_species_fields: &'a mut [CpuField<f32>],
    pub(crate) perturbation_inverse_density: &'a CpuField<f32>,
    pub(crate) base_inverse_density: &'a CpuField<f32>,
    pub(crate) perturbation_pressure: &'a CpuField<f32>,
    pub(crate) base_pressure: &'a CpuField<f32>,
    pub(crate) perturbation_geopotential: &'a CpuField<f32>,
    pub(crate) base_geopotential: &'a CpuField<f32>,
    pub(crate) accumulated_precipitation: &'a mut CpuField<f32>,
    pub(crate) step_precipitation: &'a mut CpuField<f32>,
}

impl<'a> ArwMicrophysicsState<'a> {
    /// Groups the complete Kessler-relevant ARW state without copying.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        perturbation_potential_temperature: &'a mut CpuField<f32>,
        moisture_species_fields: &'a mut [CpuField<f32>],
        perturbation_inverse_density: &'a CpuField<f32>,
        base_inverse_density: &'a CpuField<f32>,
        perturbation_pressure: &'a CpuField<f32>,
        base_pressure: &'a CpuField<f32>,
        perturbation_geopotential: &'a CpuField<f32>,
        base_geopotential: &'a CpuField<f32>,
        accumulated_precipitation: &'a mut CpuField<f32>,
        step_precipitation: &'a mut CpuField<f32>,
    ) -> Self {
        Self {
            perturbation_potential_temperature,
            moisture_species_fields,
            perturbation_inverse_density,
            base_inverse_density,
            perturbation_pressure,
            base_pressure,
            perturbation_geopotential,
            base_geopotential,
            accumulated_precipitation,
            step_precipitation,
        }
    }
}
