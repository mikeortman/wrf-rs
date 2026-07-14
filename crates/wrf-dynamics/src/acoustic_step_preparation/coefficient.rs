use std::fmt;

/// Vertical coefficient array read by acoustic-step preparation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticStepPreparationCoefficient {
    /// Half-level column-mass multiplier (`c1h`).
    HalfLevelMassMultiplier,
    /// Half-level additive offset (`c2h`).
    HalfLevelOffset,
    /// Full-level column-mass multiplier (`c1f`).
    FullLevelMassMultiplier,
    /// Full-level additive offset (`c2f`).
    FullLevelOffset,
}

impl fmt::Display for AcousticStepPreparationCoefficient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::HalfLevelMassMultiplier => "half-level mass multiplier",
            Self::HalfLevelOffset => "half-level offset",
            Self::FullLevelMassMultiplier => "full-level mass multiplier",
            Self::FullLevelOffset => "full-level offset",
        })
    }
}
