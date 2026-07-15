use std::fmt;

/// Vertical coefficient role used by acoustic-finalization validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticStepFinalizationCoefficient {
    /// Half-level column-mass multiplier (`c1h`).
    HalfLevelMassMultiplier,
    /// Half-level column-mass offset (`c2h`).
    HalfLevelMassOffset,
    /// Full-level column-mass multiplier (`c1f`).
    FullLevelMassMultiplier,
    /// Full-level column-mass offset (`c2f`).
    FullLevelMassOffset,
}

impl fmt::Display for AcousticStepFinalizationCoefficient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HalfLevelMassMultiplier => formatter.write_str("half-level mass multiplier"),
            Self::HalfLevelMassOffset => formatter.write_str("half-level mass offset"),
            Self::FullLevelMassMultiplier => formatter.write_str("full-level mass multiplier"),
            Self::FullLevelMassOffset => formatter.write_str("full-level mass offset"),
        }
    }
}
