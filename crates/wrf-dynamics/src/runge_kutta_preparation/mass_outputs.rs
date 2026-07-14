/// Mutable full and staggered dry-air column-mass outputs.
pub struct RungeKuttaPreparationMassOutputs<'a, Field> {
    pub(crate) full: &'a mut Field,
    pub(crate) west_east_momentum: &'a mut Field,
    pub(crate) south_north_momentum: &'a mut Field,
}

impl<'a, Field> RungeKuttaPreparationMassOutputs<'a, Field> {
    /// Groups WRF `mut`, `muu`, and `muv` without allocating.
    pub fn new(
        full: &'a mut Field,
        west_east_momentum: &'a mut Field,
        south_north_momentum: &'a mut Field,
    ) -> Self {
        Self {
            full,
            west_east_momentum,
            south_north_momentum,
        }
    }
}
