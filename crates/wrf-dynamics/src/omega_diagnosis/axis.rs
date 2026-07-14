use std::fmt;

/// Logical axis used by omega-diagnosis range validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OmegaDiagnosisAxis {
    /// West-east axis.
    WestEast,
    /// South-north axis.
    SouthNorth,
    /// Bottom-top axis.
    BottomTop,
}

impl fmt::Display for OmegaDiagnosisAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WestEast => formatter.write_str("west-east"),
            Self::SouthNorth => formatter.write_str("south-north"),
            Self::BottomTop => formatter.write_str("bottom-top"),
        }
    }
}
