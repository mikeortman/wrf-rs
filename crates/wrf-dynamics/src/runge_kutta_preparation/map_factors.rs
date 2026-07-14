use crate::{MomentumCouplingMapFactors, OmegaDiagnosisMapFactors};

/// Map factors that actually participate in WRF Runge-Kutta preparation.
///
/// WRF passes three additional map-factor arrays to `rk_step_prep`, but the
/// seven called routines do not read them. The Rust boundary omits those dead
/// arguments while preserving every participating value.
#[derive(Clone, Copy, Debug)]
pub struct RungeKuttaPreparationMapFactors<'a, Field> {
    pub(crate) mass_point_west_east: &'a Field,
    pub(crate) mass_point_south_north: &'a Field,
    pub(crate) west_east_momentum_south_north: &'a Field,
    pub(crate) inverse_south_north_momentum_west_east: &'a Field,
}

impl<'a, Field> RungeKuttaPreparationMapFactors<'a, Field> {
    /// Groups the four map-factor fields read by the integrated diagnostics.
    pub const fn new(
        mass_point_west_east: &'a Field,
        mass_point_south_north: &'a Field,
        west_east_momentum_south_north: &'a Field,
        inverse_south_north_momentum_west_east: &'a Field,
    ) -> Self {
        Self {
            mass_point_west_east,
            mass_point_south_north,
            west_east_momentum_south_north,
            inverse_south_north_momentum_west_east,
        }
    }

    pub(crate) const fn momentum(&self) -> MomentumCouplingMapFactors<'a, Field> {
        MomentumCouplingMapFactors::new(
            self.west_east_momentum_south_north,
            self.inverse_south_north_momentum_west_east,
            self.mass_point_south_north,
        )
    }

    pub(crate) const fn omega(&self) -> OmegaDiagnosisMapFactors<'a, Field> {
        OmegaDiagnosisMapFactors::new(
            self.mass_point_west_east,
            self.west_east_momentum_south_north,
            self.inverse_south_north_momentum_west_east,
        )
    }
}
