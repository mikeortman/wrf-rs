use crate::{MomentumCouplingVelocities, OmegaDiagnosisVelocities};

/// Borrowed C-grid velocity fields read during Runge-Kutta preparation.
#[derive(Clone, Copy, Debug)]
pub struct RungeKuttaPreparationVelocities<'a, Field> {
    pub(crate) west_east: &'a Field,
    pub(crate) south_north: &'a Field,
    pub(crate) vertical: &'a Field,
}

impl<'a, Field> RungeKuttaPreparationVelocities<'a, Field> {
    /// Groups WRF `u`, `v`, and `w` without copying field storage.
    pub const fn new(west_east: &'a Field, south_north: &'a Field, vertical: &'a Field) -> Self {
        Self {
            west_east,
            south_north,
            vertical,
        }
    }

    pub(crate) const fn momentum(&self) -> MomentumCouplingVelocities<'a, Field> {
        MomentumCouplingVelocities::new(self.west_east, self.south_north, self.vertical)
    }

    pub(crate) const fn omega(&self) -> OmegaDiagnosisVelocities<'a, Field> {
        OmegaDiagnosisVelocities::new(self.west_east, self.south_north)
    }
}
