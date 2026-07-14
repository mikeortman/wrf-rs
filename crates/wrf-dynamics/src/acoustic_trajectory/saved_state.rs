use wrf_compute::FieldStorage;

/// Saved large-step state and preparation diagnostics reused by small steps.
#[derive(Debug)]
pub struct AcousticTrajectorySavedState<'a, Field: FieldStorage<f32>> {
    pub(crate) west_east_momentum: &'a mut Field,
    pub(crate) south_north_momentum: &'a mut Field,
    pub(crate) vertical_momentum: &'a mut Field,
    pub(crate) potential_temperature: &'a mut Field,
    pub(crate) perturbation_geopotential: &'a mut Field,
    pub(crate) perturbation_column_mass: &'a mut Field,
    pub(crate) vertical_mass_flux: &'a mut Field,
    pub(crate) pressure_coefficient: &'a mut Field,
}

impl<'a, Field: FieldStorage<f32>> AcousticTrajectorySavedState<'a, Field> {
    /// Groups WRF's saved fields, `ww1`, and `c2a` without allocation.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        west_east_momentum: &'a mut Field,
        south_north_momentum: &'a mut Field,
        vertical_momentum: &'a mut Field,
        potential_temperature: &'a mut Field,
        perturbation_geopotential: &'a mut Field,
        perturbation_column_mass: &'a mut Field,
        vertical_mass_flux: &'a mut Field,
        pressure_coefficient: &'a mut Field,
    ) -> Self {
        Self {
            west_east_momentum,
            south_north_momentum,
            vertical_momentum,
            potential_temperature,
            perturbation_geopotential,
            perturbation_column_mass,
            vertical_mass_flux,
            pressure_coefficient,
        }
    }
}
