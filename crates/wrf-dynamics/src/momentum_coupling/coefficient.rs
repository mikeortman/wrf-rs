use std::fmt;

/// Vertical coefficient role in WRF momentum coupling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MomentumCouplingCoefficient {
    /// Half-level column-mass multiplier (`c1h`).
    HalfLevelMassMultiplier,
    /// Half-level additive term (`c2h`).
    HalfLevelOffset,
    /// Full-level column-mass multiplier (`c1f`).
    FullLevelMassMultiplier,
    /// Full-level additive term (`c2f`).
    FullLevelOffset,
}

impl fmt::Display for MomentumCouplingCoefficient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HalfLevelMassMultiplier => formatter.write_str("half-level mass multiplier"),
            Self::HalfLevelOffset => formatter.write_str("half-level offset"),
            Self::FullLevelMassMultiplier => formatter.write_str("full-level mass multiplier"),
            Self::FullLevelOffset => formatter.write_str("full-level offset"),
        }
    }
}
