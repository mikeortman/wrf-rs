/// Mutable previous/current perturbation column-mass fields.
pub struct AcousticStepPreparationColumnMassTimeLevels<'a, Field> {
    pub(crate) previous: &'a mut Field,
    pub(crate) current: &'a mut Field,
}

impl<'a, Field> AcousticStepPreparationColumnMassTimeLevels<'a, Field> {
    /// Groups WRF's `mu_1` and `mu_2` fields without allocation.
    pub fn new(previous: &'a mut Field, current: &'a mut Field) -> Self {
        Self { previous, current }
    }
}
