use std::fmt;

/// Scientific role of a one-dimensional vertical coefficient.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerticalAcousticCoefficient {
    /// Half-level column-mass multiplier (`c1h`).
    HalfLevelMassMultiplier,
    /// Half-level column-mass offset (`c2h`).
    HalfLevelMassOffset,
    /// Full-level column-mass multiplier (`c1f`).
    FullLevelMassMultiplier,
    /// Full-level column-mass offset (`c2f`).
    FullLevelMassOffset,
    /// Inverse full-level eta spacing (`rdn`).
    InverseFullLevelEtaSpacing,
    /// Inverse half-level eta spacing (`rdnw`).
    InverseHalfLevelEtaSpacing,
}

impl fmt::Display for VerticalAcousticCoefficient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::HalfLevelMassMultiplier => "half-level mass multiplier",
            Self::HalfLevelMassOffset => "half-level mass offset",
            Self::FullLevelMassMultiplier => "full-level mass multiplier",
            Self::FullLevelMassOffset => "full-level mass offset",
            Self::InverseFullLevelEtaSpacing => "inverse full-level eta spacing",
            Self::InverseHalfLevelEtaSpacing => "inverse half-level eta spacing",
        })
    }
}
