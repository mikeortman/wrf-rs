use wrf_compute::FieldStorage;

/// Mutable prognostic state advanced by one acoustic step.
#[derive(Debug)]
pub struct AcousticMassThetaState<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) vertical_mass_flux: &'a mut Field,
    pub(crate) column_mass: &'a mut Field,
    pub(crate) potential_temperature: &'a mut Field,
}

impl<'a, Field> AcousticMassThetaState<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `ww`, `mu`, and `t` as non-aliasing mutable fields.
    pub const fn new(
        vertical_mass_flux: &'a mut Field,
        column_mass: &'a mut Field,
        potential_temperature: &'a mut Field,
    ) -> Self {
        Self {
            vertical_mass_flux,
            column_mass,
            potential_temperature,
        }
    }
}
