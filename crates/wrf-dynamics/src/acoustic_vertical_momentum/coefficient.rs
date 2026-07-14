use std::fmt;

/// One-dimensional coefficient role consumed by `advance_w`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticVerticalCoefficient {
    /// WRF `c1h`.
    HalfLevelMassMultiplier,
    /// WRF `c2h`.
    HalfLevelMassOffset,
    /// WRF `c1f`.
    FullLevelMassMultiplier,
    /// WRF `c2f`.
    FullLevelMassOffset,
    /// WRF `fnm`.
    UpperInterpolationWeight,
    /// WRF `fnp`.
    LowerInterpolationWeight,
    /// WRF `rdnw`.
    InverseHalfLevelSpacing,
    /// WRF `rdn`.
    InverseFullLevelSpacing,
}

impl fmt::Display for AcousticVerticalCoefficient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::HalfLevelMassMultiplier => "half-level mass multiplier",
            Self::HalfLevelMassOffset => "half-level mass offset",
            Self::FullLevelMassMultiplier => "full-level mass multiplier",
            Self::FullLevelMassOffset => "full-level mass offset",
            Self::UpperInterpolationWeight => "upper interpolation weight",
            Self::LowerInterpolationWeight => "lower interpolation weight",
            Self::InverseHalfLevelSpacing => "inverse half-level spacing",
            Self::InverseFullLevelSpacing => "inverse full-level spacing",
        })
    }
}
