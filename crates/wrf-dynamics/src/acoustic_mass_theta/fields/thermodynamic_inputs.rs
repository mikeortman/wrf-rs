use wrf_compute::FieldStorage;

/// Borrowed saved potential temperature and its large-step tendency.
#[derive(Debug)]
pub struct AcousticMassThetaThermodynamicInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) saved_potential_temperature: &'a Field,
    pub(crate) potential_temperature_tendency: &'a Field,
}

impl<Field> Copy for AcousticMassThetaThermodynamicInputs<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticMassThetaThermodynamicInputs<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticMassThetaThermodynamicInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `t_1` and `ft` without copying field data.
    pub const fn new(
        saved_potential_temperature: &'a Field,
        potential_temperature_tendency: &'a Field,
    ) -> Self {
        Self {
            saved_potential_temperature,
            potential_temperature_tendency,
        }
    }
}
