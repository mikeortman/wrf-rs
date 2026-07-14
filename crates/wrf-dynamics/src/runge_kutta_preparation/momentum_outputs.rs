/// Mutable mass-coupled momentum outputs.
pub struct RungeKuttaPreparationMomentumOutputs<'a, Field> {
    pub(crate) west_east: &'a mut Field,
    pub(crate) south_north: &'a mut Field,
    pub(crate) vertical: &'a mut Field,
}

impl<'a, Field> RungeKuttaPreparationMomentumOutputs<'a, Field> {
    /// Groups WRF `ru`, `rv`, and `rw` without allocating.
    pub fn new(
        west_east: &'a mut Field,
        south_north: &'a mut Field,
        vertical: &'a mut Field,
    ) -> Self {
        Self {
            west_east,
            south_north,
            vertical,
        }
    }
}
