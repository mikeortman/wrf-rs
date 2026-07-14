use std::fmt;

/// Logical axis used when reporting an invalid Kessler execution range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KesslerMicrophysicsAxis {
    /// West-east horizontal axis.
    WestEast,
    /// South-north horizontal axis.
    SouthNorth,
    /// Bottom-top vertical axis.
    BottomTop,
}

impl fmt::Display for KesslerMicrophysicsAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WestEast => formatter.write_str("west-east"),
            Self::SouthNorth => formatter.write_str("south-north"),
            Self::BottomTop => formatter.write_str("bottom-top"),
        }
    }
}
