use std::fmt;

/// Vertical coefficient role in WRF omega diagnosis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OmegaDiagnosisCoefficient {
    /// Half-level column-mass multiplier (`c1h`).
    HalfLevelMassMultiplier,
    /// Half-level column-mass offset (`c2h`).
    HalfLevelMassOffset,
    /// Eta-layer thickness (`dnw`).
    EtaLayerThickness,
}

impl fmt::Display for OmegaDiagnosisCoefficient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HalfLevelMassMultiplier => formatter.write_str("half-level mass multiplier"),
            Self::HalfLevelMassOffset => formatter.write_str("half-level mass offset"),
            Self::EtaLayerThickness => formatter.write_str("eta-layer thickness"),
        }
    }
}
