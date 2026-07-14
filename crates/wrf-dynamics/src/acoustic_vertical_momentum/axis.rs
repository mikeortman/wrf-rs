use std::fmt;

/// Logical axis used in acoustic vertical range errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticVerticalAxis {
    /// West-east mass-point axis.
    WestEast,
    /// South-north mass-point axis.
    SouthNorth,
    /// Bottom-top full-level axis.
    BottomTop,
}

impl fmt::Display for AcousticVerticalAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WestEast => "west-east",
            Self::SouthNorth => "south-north",
            Self::BottomTop => "bottom-top",
        })
    }
}
