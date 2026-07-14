use std::fmt;

/// Logical axis used by pressure-point geopotential range validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PressurePointGeopotentialAxis {
    /// West-east axis.
    WestEast,
    /// South-north axis.
    SouthNorth,
    /// Bottom-top axis.
    BottomTop,
}

impl fmt::Display for PressurePointGeopotentialAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WestEast => formatter.write_str("west-east"),
            Self::SouthNorth => formatter.write_str("south-north"),
            Self::BottomTop => formatter.write_str("bottom-top"),
        }
    }
}
