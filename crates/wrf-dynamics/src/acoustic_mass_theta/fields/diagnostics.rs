use wrf_compute::FieldStorage;

/// Mutable saved and diagnostic outputs produced with the prognostic update.
#[derive(Debug)]
pub struct AcousticMassThetaDiagnostics<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) time_centered_column_mass: &'a mut Field,
    pub(crate) coupled_column_mass: &'a mut Field,
    pub(crate) divergence_damping_mass_tendency: &'a mut Field,
    pub(crate) previous_potential_temperature: &'a mut Field,
}

impl<'a, Field> AcousticMassThetaDiagnostics<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `muave`, `muts`, `mudf`, and `t_ave` outputs.
    pub const fn new(
        time_centered_column_mass: &'a mut Field,
        coupled_column_mass: &'a mut Field,
        divergence_damping_mass_tendency: &'a mut Field,
        previous_potential_temperature: &'a mut Field,
    ) -> Self {
        Self {
            time_centered_column_mass,
            coupled_column_mass,
            divergence_damping_mass_tendency,
            previous_potential_temperature,
        }
    }
}
