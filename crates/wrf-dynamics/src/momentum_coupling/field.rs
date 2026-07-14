use std::fmt;

/// Field role in WRF momentum coupling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MomentumCouplingField {
    /// Mass-coupled west-east momentum output (`ru`).
    WestEastMomentumOutput,
    /// Mass-coupled south-north momentum output (`rv`).
    SouthNorthMomentumOutput,
    /// Mass-coupled vertical momentum output (`rw`).
    VerticalMomentumOutput,
    /// West-east velocity input (`u`).
    WestEastVelocity,
    /// South-north velocity input (`v`).
    SouthNorthVelocity,
    /// Vertical velocity input (`w`).
    VerticalVelocity,
    /// Full column mass at west-east momentum points (`muu`).
    WestEastMomentumMass,
    /// Full column mass at south-north momentum points (`muv`).
    SouthNorthMomentumMass,
    /// Full column mass at mass points (`mut`).
    MassPointColumnMass,
    /// West-east momentum map factor (`msfu`).
    WestEastMapFactor,
    /// Inverse south-north momentum map factor (`msfv_inv`).
    InverseSouthNorthMapFactor,
    /// Mass-point map factor (`msft`).
    MassPointMapFactor,
}

impl fmt::Display for MomentumCouplingField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WestEastMomentumOutput => formatter.write_str("west-east momentum output"),
            Self::SouthNorthMomentumOutput => formatter.write_str("south-north momentum output"),
            Self::VerticalMomentumOutput => formatter.write_str("vertical momentum output"),
            Self::WestEastVelocity => formatter.write_str("west-east velocity"),
            Self::SouthNorthVelocity => formatter.write_str("south-north velocity"),
            Self::VerticalVelocity => formatter.write_str("vertical velocity"),
            Self::WestEastMomentumMass => formatter.write_str("west-east momentum mass"),
            Self::SouthNorthMomentumMass => formatter.write_str("south-north momentum mass"),
            Self::MassPointColumnMass => formatter.write_str("mass-point column mass"),
            Self::WestEastMapFactor => formatter.write_str("west-east map factor"),
            Self::InverseSouthNorthMapFactor => {
                formatter.write_str("inverse south-north map factor")
            }
            Self::MassPointMapFactor => formatter.write_str("mass-point map factor"),
        }
    }
}
