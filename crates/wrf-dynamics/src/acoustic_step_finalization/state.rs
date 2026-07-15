use wrf_compute::FieldStorage;

/// Mutable prognostic fields reconstructed after acoustic integration.
#[derive(Debug)]
pub struct AcousticStepFinalizationState<'a, Field: FieldStorage<f32>> {
    pub(crate) west_east_velocity: &'a mut Field,
    pub(crate) south_north_velocity: &'a mut Field,
    pub(crate) vertical_velocity: &'a mut Field,
    pub(crate) potential_temperature: &'a mut Field,
    pub(crate) perturbation_geopotential: &'a mut Field,
    pub(crate) vertical_mass_flux: &'a mut Field,
    pub(crate) perturbation_column_mass: &'a mut Field,
}

impl<'a, Field: FieldStorage<f32>> AcousticStepFinalizationState<'a, Field> {
    /// Groups all mutable outputs without copying or aliasing storage.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        west_east_velocity: &'a mut Field,
        south_north_velocity: &'a mut Field,
        vertical_velocity: &'a mut Field,
        potential_temperature: &'a mut Field,
        perturbation_geopotential: &'a mut Field,
        vertical_mass_flux: &'a mut Field,
        perturbation_column_mass: &'a mut Field,
    ) -> Self {
        Self {
            west_east_velocity,
            south_north_velocity,
            vertical_velocity,
            potential_temperature,
            perturbation_geopotential,
            vertical_mass_flux,
            perturbation_column_mass,
        }
    }
}
