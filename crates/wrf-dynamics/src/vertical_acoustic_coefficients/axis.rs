use std::fmt;

/// Logical axis used by vertical-acoustic coefficient range diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerticalAcousticCoefficientAxis {
    /// West-east mass-point axis.
    WestEast,
    /// South-north mass-point axis.
    SouthNorth,
    /// Bottom-top half-level axis.
    BottomTop,
}

impl fmt::Display for VerticalAcousticCoefficientAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WestEast => "west-east",
            Self::SouthNorth => "south-north",
            Self::BottomTop => "bottom-top",
        })
    }
}
