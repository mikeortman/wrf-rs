use std::fmt;

/// Logical axis used by acoustic mass-and-theta validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticMassThetaAxis {
    /// West-east axis.
    WestEast,
    /// South-north axis.
    SouthNorth,
    /// Bottom-top axis.
    BottomTop,
}

impl fmt::Display for AcousticMassThetaAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WestEast => "west-east",
            Self::SouthNorth => "south-north",
            Self::BottomTop => "bottom-top",
        })
    }
}
