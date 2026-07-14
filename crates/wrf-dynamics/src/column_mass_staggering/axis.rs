use std::fmt;

/// Logical field axis used in column-mass staggering diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColumnMassStaggeringAxis {
    /// West-east memory axis.
    WestEast,
    /// South-north memory axis.
    SouthNorth,
}

impl fmt::Display for ColumnMassStaggeringAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WestEast => formatter.write_str("west-east"),
            Self::SouthNorth => formatter.write_str("south-north"),
        }
    }
}
