/// Borrowed horizontal velocity fields for omega diagnosis.
#[derive(Clone, Copy, Debug)]
pub struct OmegaDiagnosisVelocities<'a, Field> {
    pub(crate) west_east: &'a Field,
    pub(crate) south_north: &'a Field,
}

impl<'a, Field> OmegaDiagnosisVelocities<'a, Field> {
    /// Groups the west-east and south-north velocity fields.
    pub const fn new(west_east: &'a Field, south_north: &'a Field) -> Self {
        Self {
            west_east,
            south_north,
        }
    }
}
