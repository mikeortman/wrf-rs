use crate::{AcousticMassThetaLateralDomain, AcousticMassThetaWestEastPeriodicity};

/// Boundary policy consumed by WRF `advance_mu_t` range derivation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcousticMassThetaBoundaryPolicy {
    pub(crate) lateral_domain: AcousticMassThetaLateralDomain,
    pub(crate) west_east_periodicity: AcousticMassThetaWestEastPeriodicity,
}

impl AcousticMassThetaBoundaryPolicy {
    /// Creates a typed replacement for WRF's nested, specified, and periodic-X flags.
    pub const fn new(
        lateral_domain: AcousticMassThetaLateralDomain,
        west_east_periodicity: AcousticMassThetaWestEastPeriodicity,
    ) -> Self {
        Self {
            lateral_domain,
            west_east_periodicity,
        }
    }
}
