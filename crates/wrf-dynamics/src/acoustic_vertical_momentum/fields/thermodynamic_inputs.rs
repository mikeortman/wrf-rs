use wrf_compute::FieldStorage;

/// Potential-temperature time levels used to normalize `t_2ave`.
#[derive(Debug)]
pub struct AcousticVerticalThermodynamicInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) potential_temperature: &'a Field,
    pub(crate) saved_potential_temperature: &'a Field,
}

impl<Field> Copy for AcousticVerticalThermodynamicInputs<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticVerticalThermodynamicInputs<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticVerticalThermodynamicInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `t_2` and the `t_save` value passed as `t_1`.
    pub const fn new(
        potential_temperature: &'a Field,
        saved_potential_temperature: &'a Field,
    ) -> Self {
        Self {
            potential_temperature,
            saved_potential_temperature,
        }
    }
}
