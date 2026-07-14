use std::fmt;

/// Grid axis named by an acoustic-pressure range error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticPressureAxis {
    /// West-east mass points.
    WestEast,
    /// South-north mass points.
    SouthNorth,
    /// Bottom-top half levels.
    BottomTop,
}

impl fmt::Display for AcousticPressureAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WestEast => "west-east",
            Self::SouthNorth => "south-north",
            Self::BottomTop => "bottom-top",
        })
    }
}
