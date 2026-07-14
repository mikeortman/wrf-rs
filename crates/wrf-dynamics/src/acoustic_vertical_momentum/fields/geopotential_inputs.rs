use wrf_compute::FieldStorage;

/// Immutable geopotential time level, base state, and tendency.
#[derive(Debug)]
pub struct AcousticVerticalGeopotentialInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) saved_perturbation: &'a Field,
    pub(crate) base: &'a Field,
    pub(crate) tendency: &'a Field,
}

impl<Field> Copy for AcousticVerticalGeopotentialInputs<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticVerticalGeopotentialInputs<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticVerticalGeopotentialInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `ph_1`, `phb`, and `ph_tend`.
    pub const fn new(saved_perturbation: &'a Field, base: &'a Field, tendency: &'a Field) -> Self {
        Self {
            saved_perturbation,
            base,
            tendency,
        }
    }
}
