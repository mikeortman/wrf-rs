use wrf_compute::FieldStorage;

/// Saved pre-acoustic fields added back during finalization.
#[derive(Clone, Copy, Debug)]
pub struct AcousticStepFinalizationSavedState<'a, Field: FieldStorage<f32>> {
    pub(crate) west_east_velocity: &'a Field,
    pub(crate) south_north_velocity: &'a Field,
    pub(crate) vertical_velocity: &'a Field,
    pub(crate) potential_temperature: &'a Field,
    pub(crate) perturbation_geopotential: &'a Field,
    pub(crate) perturbation_column_mass: &'a Field,
    pub(crate) vertical_mass_flux: &'a Field,
    pub(crate) diabatic_heating: &'a Field,
}

impl<'a, Field: FieldStorage<f32>> AcousticStepFinalizationSavedState<'a, Field> {
    /// Groups WRF's saved fields plus the final-stage heating input.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        west_east_velocity: &'a Field,
        south_north_velocity: &'a Field,
        vertical_velocity: &'a Field,
        potential_temperature: &'a Field,
        perturbation_geopotential: &'a Field,
        perturbation_column_mass: &'a Field,
        vertical_mass_flux: &'a Field,
        diabatic_heating: &'a Field,
    ) -> Self {
        Self {
            west_east_velocity,
            south_north_velocity,
            vertical_velocity,
            potential_temperature,
            perturbation_geopotential,
            perturbation_column_mass,
            vertical_mass_flux,
            diabatic_heating,
        }
    }
}
