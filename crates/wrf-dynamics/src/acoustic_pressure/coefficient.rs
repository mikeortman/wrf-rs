use std::fmt;

/// One vertical coefficient or metric used by acoustic pressure diagnosis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticPressureCoefficient {
    /// Half-level column-mass multiplier (`c1h`).
    HalfLevelMassMultiplier,
    /// Half-level additive mass offset (`c2h`).
    HalfLevelMassOffset,
    /// Hydrostatic pressure multiplier (`c3h`).
    HydrostaticPressureMultiplier,
    /// Inverse vertical-coordinate thickness (`rdnw`).
    InverseEtaThickness,
    /// Vertical-coordinate thickness (`dnw`).
    EtaThickness,
}

impl fmt::Display for AcousticPressureCoefficient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::HalfLevelMassMultiplier => "half-level mass multiplier",
            Self::HalfLevelMassOffset => "half-level mass offset",
            Self::HydrostaticPressureMultiplier => "hydrostatic pressure multiplier",
            Self::InverseEtaThickness => "inverse eta thickness",
            Self::EtaThickness => "eta thickness",
        })
    }
}
