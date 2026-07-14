use crate::{HorizontalBounds, IndexRange};

/// Physical WRF domain bounds in logical west-east, bottom-top, south-north order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomainBounds {
    horizontal: HorizontalBounds,
    bottom_top: IndexRange,
}

impl DomainBounds {
    /// Creates physical domain bounds from validated axis ranges.
    pub const fn new(
        west_east: IndexRange,
        bottom_top: IndexRange,
        south_north: IndexRange,
    ) -> Self {
        Self {
            horizontal: HorizontalBounds::new(west_east, south_north),
            bottom_top,
        }
    }

    /// Returns both horizontal ranges.
    pub const fn horizontal(self) -> HorizontalBounds {
        self.horizontal
    }

    /// Returns the west-east range.
    pub const fn west_east(self) -> IndexRange {
        self.horizontal.west_east()
    }

    /// Returns the bottom-top range.
    pub const fn bottom_top(self) -> IndexRange {
        self.bottom_top
    }

    /// Returns the south-north range.
    pub const fn south_north(self) -> IndexRange {
        self.horizontal.south_north()
    }
}
