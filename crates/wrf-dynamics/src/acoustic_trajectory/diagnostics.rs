use wrf_compute::FieldStorage;

/// Mutable diagnostics and solver state produced across an acoustic trajectory.
#[derive(Debug)]
pub struct AcousticTrajectoryDiagnostics<'a, Field: FieldStorage<f32>> {
    pub(crate) vertical_mass_flux: &'a mut Field,
    pub(crate) saved_west_east_column_mass: &'a mut Field,
    pub(crate) saved_south_north_column_mass: &'a mut Field,
    pub(crate) coupled_column_mass: &'a mut Field,
    pub(crate) divergence_damping_column_mass: &'a mut Field,
    pub(crate) inverse_density_perturbation: &'a mut Field,
    pub(crate) pressure_perturbation: &'a mut Field,
    pub(crate) previous_pressure_perturbation: &'a mut Field,
    pub(crate) lower_diagonal: &'a mut Field,
    pub(crate) inverse_eliminated_diagonal: &'a mut Field,
    pub(crate) upper_elimination_factor: &'a mut Field,
    pub(crate) time_centered_column_mass: &'a mut Field,
    pub(crate) time_averaged_thermodynamics: &'a mut Field,
    pub(crate) average_west_east_mass_flux: &'a mut Field,
    pub(crate) average_south_north_mass_flux: &'a mut Field,
    pub(crate) average_vertical_mass_flux: &'a mut Field,
}

impl<'a, Field: FieldStorage<f32>> AcousticTrajectoryDiagnostics<'a, Field> {
    /// Groups mutable diagnostic fields without allocating or aliasing storage.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        vertical_mass_flux: &'a mut Field,
        saved_west_east_column_mass: &'a mut Field,
        saved_south_north_column_mass: &'a mut Field,
        coupled_column_mass: &'a mut Field,
        divergence_damping_column_mass: &'a mut Field,
        inverse_density_perturbation: &'a mut Field,
        pressure_perturbation: &'a mut Field,
        previous_pressure_perturbation: &'a mut Field,
        lower_diagonal: &'a mut Field,
        inverse_eliminated_diagonal: &'a mut Field,
        upper_elimination_factor: &'a mut Field,
        time_centered_column_mass: &'a mut Field,
        time_averaged_thermodynamics: &'a mut Field,
        average_west_east_mass_flux: &'a mut Field,
        average_south_north_mass_flux: &'a mut Field,
        average_vertical_mass_flux: &'a mut Field,
    ) -> Self {
        Self {
            vertical_mass_flux,
            saved_west_east_column_mass,
            saved_south_north_column_mass,
            coupled_column_mass,
            divergence_damping_column_mass,
            inverse_density_perturbation,
            pressure_perturbation,
            previous_pressure_perturbation,
            lower_diagonal,
            inverse_eliminated_diagonal,
            upper_elimination_factor,
            time_centered_column_mass,
            time_averaged_thermodynamics,
            average_west_east_mass_flux,
            average_south_north_mass_flux,
            average_vertical_mass_flux,
        }
    }
}
