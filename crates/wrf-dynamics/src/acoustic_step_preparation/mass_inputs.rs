/// Immutable column masses used to form coupled perturbations.
#[derive(Clone, Copy)]
pub struct AcousticStepPreparationMassInputs<'a, Field> {
    pub(crate) base: &'a Field,
    pub(crate) west_east: &'a Field,
    pub(crate) south_north: &'a Field,
    pub(crate) full: &'a Field,
}

impl<'a, Field> AcousticStepPreparationMassInputs<'a, Field> {
    /// Groups `mub`, `muu`, `muv`, and `mut` without copying.
    pub const fn new(
        base: &'a Field,
        west_east: &'a Field,
        south_north: &'a Field,
        full: &'a Field,
    ) -> Self {
        Self {
            base,
            west_east,
            south_north,
            full,
        }
    }
}
