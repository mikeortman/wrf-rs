use std::fmt;

/// Scientific role of a field in acoustic mass-flux accumulation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticFluxField {
    /// Current west-east mass flux (`ru`).
    CurrentWestEast,
    /// Current south-north mass flux (`rv`).
    CurrentSouthNorth,
    /// Current vertical mass flux (`ww`).
    CurrentVertical,
    /// Saved linear west-east mass flux (`u_lin`).
    LinearWestEast,
    /// Saved linear south-north mass flux (`v_lin`).
    LinearSouthNorth,
    /// Saved linear vertical mass flux (`ww_lin`).
    LinearVertical,
    /// Time-averaged west-east mass flux (`ru_m`).
    AverageWestEast,
    /// Time-averaged south-north mass flux (`rv_m`).
    AverageSouthNorth,
    /// Time-averaged vertical mass flux (`ww_m`).
    AverageVertical,
    /// West-east staggered full column mass (`muu`).
    WestEastColumnMass,
    /// South-north staggered full column mass (`muv`).
    SouthNorthColumnMass,
    /// West-east Y map factor (`msfuy`).
    WestEastYMapFactor,
    /// Inverse south-north X map factor (`msfvx_inv`).
    InverseSouthNorthXMapFactor,
}

impl fmt::Display for AcousticFluxField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::CurrentWestEast => "current west-east flux",
            Self::CurrentSouthNorth => "current south-north flux",
            Self::CurrentVertical => "current vertical flux",
            Self::LinearWestEast => "linear west-east flux",
            Self::LinearSouthNorth => "linear south-north flux",
            Self::LinearVertical => "linear vertical flux",
            Self::AverageWestEast => "average west-east flux",
            Self::AverageSouthNorth => "average south-north flux",
            Self::AverageVertical => "average vertical flux",
            Self::WestEastColumnMass => "west-east column mass",
            Self::SouthNorthColumnMass => "south-north column mass",
            Self::WestEastYMapFactor => "west-east Y map factor",
            Self::InverseSouthNorthXMapFactor => "inverse south-north X map factor",
        })
    }
}
