use std::fmt;

/// Field role in WRF omega diagnosis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OmegaDiagnosisField {
    /// Diagnosed dry-air eta-velocity output (`ww`).
    OmegaOutput,
    /// West-east velocity (`u`).
    WestEastVelocity,
    /// South-north velocity (`v`).
    SouthNorthVelocity,
    /// Perturbation dry-air column mass (`mup`).
    PerturbationColumnMass,
    /// Base-state dry-air column mass (`mub`).
    BaseColumnMass,
    /// Mass-point west-east map factor (`msftx`).
    MassPointWestEastMapFactor,
    /// West-east momentum south-north map factor (`msfuy`).
    WestEastMomentumSouthNorthMapFactor,
    /// Inverse south-north momentum west-east map factor (`msfvx_inv`).
    InverseSouthNorthMomentumWestEastMapFactor,
}

impl fmt::Display for OmegaDiagnosisField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OmegaOutput => formatter.write_str("omega output"),
            Self::WestEastVelocity => formatter.write_str("west-east velocity"),
            Self::SouthNorthVelocity => formatter.write_str("south-north velocity"),
            Self::PerturbationColumnMass => formatter.write_str("perturbation column mass"),
            Self::BaseColumnMass => formatter.write_str("base-state column mass"),
            Self::MassPointWestEastMapFactor => {
                formatter.write_str("mass-point west-east map factor")
            }
            Self::WestEastMomentumSouthNorthMapFactor => {
                formatter.write_str("west-east momentum south-north map factor")
            }
            Self::InverseSouthNorthMomentumWestEastMapFactor => {
                formatter.write_str("inverse south-north momentum west-east map factor")
            }
        }
    }
}
