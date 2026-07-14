use std::fmt;

/// Scientific role of a vertical coefficient used by `advance_uv`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticHorizontalMomentumCoefficient {
    /// Half-level mass multiplier (`c1h`).
    HalfLevelMassMultiplier,
    /// Half-level mass offset (`c2h`).
    HalfLevelMassOffset,
    /// Lower interpolation weight (`fnm`).
    LowerInterpolationWeight,
    /// Upper interpolation weight (`fnp`).
    UpperInterpolationWeight,
    /// Inverse half-level eta spacing (`rdnw`).
    InverseHalfLevelEtaSpacing,
}

impl fmt::Display for AcousticHorizontalMomentumCoefficient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::HalfLevelMassMultiplier => "half-level mass multiplier",
            Self::HalfLevelMassOffset => "half-level mass offset",
            Self::LowerInterpolationWeight => "lower interpolation weight",
            Self::UpperInterpolationWeight => "upper interpolation weight",
            Self::InverseHalfLevelEtaSpacing => "inverse half-level eta spacing",
        })
    }
}
