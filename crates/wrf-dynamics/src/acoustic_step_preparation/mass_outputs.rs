/// Mutable saved/staggered masses and divergence-damping state.
pub struct AcousticStepPreparationMassOutputs<'a, Field> {
    pub(crate) saved_west_east: &'a mut Field,
    pub(crate) saved_south_north: &'a mut Field,
    pub(crate) saved_mass_point: &'a mut Field,
    pub(crate) divergence_damping: &'a mut Field,
}

impl<'a, Field> AcousticStepPreparationMassOutputs<'a, Field> {
    /// Groups WRF's `muus`, `muvs`, `muts`, and `mudf` outputs.
    pub fn new(
        saved_west_east: &'a mut Field,
        saved_south_north: &'a mut Field,
        saved_mass_point: &'a mut Field,
        divergence_damping: &'a mut Field,
    ) -> Self {
        Self {
            saved_west_east,
            saved_south_north,
            saved_mass_point,
            divergence_damping,
        }
    }
}
