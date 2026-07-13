use std::fmt;

/// A logical axis in a three-dimensional positive-definite slab.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PositiveDefiniteSlabAxis {
    /// First-index-contiguous west-east axis.
    WestEast,
    /// Vertical bottom-top axis.
    BottomTop,
    /// South-north axis.
    SouthNorth,
}

impl fmt::Display for PositiveDefiniteSlabAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WestEast => formatter.write_str("west-east"),
            Self::BottomTop => formatter.write_str("bottom-top"),
            Self::SouthNorth => formatter.write_str("south-north"),
        }
    }
}
