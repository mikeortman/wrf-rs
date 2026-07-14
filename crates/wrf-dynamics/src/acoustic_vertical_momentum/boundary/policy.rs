use crate::{AcousticVerticalLateralDomain, AcousticVerticalWestEastPeriodicity};

/// Lateral range policy for one implicit acoustic vertical solve.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcousticVerticalBoundaryPolicy {
    pub(crate) lateral_domain: AcousticVerticalLateralDomain,
    pub(crate) west_east_periodicity: AcousticVerticalWestEastPeriodicity,
}

impl AcousticVerticalBoundaryPolicy {
    /// Creates the exact WRF lateral-boundary combination.
    pub const fn new(
        lateral_domain: AcousticVerticalLateralDomain,
        west_east_periodicity: AcousticVerticalWestEastPeriodicity,
    ) -> Self {
        Self {
            lateral_domain,
            west_east_periodicity,
        }
    }
}
