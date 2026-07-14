use wrf_compute::FieldStorage;

/// Column-mass time levels consumed by vertical advancement.
#[derive(Debug)]
pub struct AcousticVerticalMassInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) full_column_mass: &'a Field,
    pub(crate) time_centered_column_mass: &'a Field,
    pub(crate) coupled_column_mass: &'a Field,
}

impl<Field> Copy for AcousticVerticalMassInputs<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticVerticalMassInputs<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticVerticalMassInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups live WRF `mut`, `muave`, and `muts` inputs.
    pub const fn new(
        full_column_mass: &'a Field,
        time_centered_column_mass: &'a Field,
        coupled_column_mass: &'a Field,
    ) -> Self {
        Self {
            full_column_mass,
            time_centered_column_mass,
            coupled_column_mass,
        }
    }
}
