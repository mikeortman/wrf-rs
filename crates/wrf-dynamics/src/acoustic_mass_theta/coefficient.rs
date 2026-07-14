use std::fmt;

/// Scientific role of a vertical coefficient used by `advance_mu_t`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticMassThetaCoefficient {
    /// Half-level mass multiplier (`c1h`).
    HalfLevelMassMultiplier,
    /// Half-level mass offset (`c2h`).
    HalfLevelMassOffset,
    /// Half-level eta thickness (`dnw`).
    HalfLevelEtaThickness,
    /// Lower full-level interpolation weight (`fnm`).
    LowerInterpolationWeight,
    /// Upper full-level interpolation weight (`fnp`).
    UpperInterpolationWeight,
    /// Inverse half-level eta spacing (`rdnw`).
    InverseHalfLevelEtaSpacing,
}

impl fmt::Display for AcousticMassThetaCoefficient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::HalfLevelMassMultiplier => "half-level mass multiplier",
            Self::HalfLevelMassOffset => "half-level mass offset",
            Self::HalfLevelEtaThickness => "half-level eta thickness",
            Self::LowerInterpolationWeight => "lower interpolation weight",
            Self::UpperInterpolationWeight => "upper interpolation weight",
            Self::InverseHalfLevelEtaSpacing => "inverse half-level eta spacing",
        })
    }
}
