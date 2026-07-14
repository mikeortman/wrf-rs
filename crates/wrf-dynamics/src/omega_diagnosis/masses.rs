/// Borrowed dry-air column-mass fields for omega diagnosis.
#[derive(Clone, Copy, Debug)]
pub struct OmegaDiagnosisMasses<'a, Field> {
    pub(crate) perturbation: &'a Field,
    pub(crate) base_state: &'a Field,
}

impl<'a, Field> OmegaDiagnosisMasses<'a, Field> {
    /// Groups perturbation and base-state column mass.
    pub const fn new(perturbation: &'a Field, base_state: &'a Field) -> Self {
        Self {
            perturbation,
            base_state,
        }
    }
}
