use std::fmt;

/// Grid axis used in dry-tendency region diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DryTendencyAssemblyAxis {
    /// West-to-east memory axis.
    WestEast,
    /// South-to-north memory axis.
    SouthNorth,
    /// Bottom-to-top memory axis.
    BottomTop,
}

impl fmt::Display for DryTendencyAssemblyAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WestEast => "west-east",
            Self::SouthNorth => "south-north",
            Self::BottomTop => "bottom-top",
        })
    }
}
