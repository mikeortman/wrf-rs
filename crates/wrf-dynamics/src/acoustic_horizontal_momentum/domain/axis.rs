use std::fmt;

/// Logical axis used by acoustic horizontal-momentum domain diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticHorizontalMomentumAxis {
    /// West-east axis.
    WestEast,
    /// South-north axis.
    SouthNorth,
    /// Bottom-top axis.
    BottomTop,
}

impl fmt::Display for AcousticHorizontalMomentumAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WestEast => "west-east",
            Self::SouthNorth => "south-north",
            Self::BottomTop => "bottom-top",
        })
    }
}
