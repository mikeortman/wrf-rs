use std::fmt;

/// A logical range axis in the Held-Suarez damping region.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeldSuarezDampingAxis {
    /// Common west-east tendency range.
    WestEast,
    /// Common active vertical range.
    BottomTop,
    /// South-north range for west-east momentum.
    WestEastMomentumSouthNorth,
    /// South-north range for south-north momentum.
    SouthNorthMomentumSouthNorth,
}

impl fmt::Display for HeldSuarezDampingAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WestEast => formatter.write_str("west-east"),
            Self::BottomTop => formatter.write_str("bottom-top"),
            Self::WestEastMomentumSouthNorth => {
                formatter.write_str("west-east momentum south-north")
            }
            Self::SouthNorthMomentumSouthNorth => {
                formatter.write_str("south-north momentum south-north")
            }
        }
    }
}
