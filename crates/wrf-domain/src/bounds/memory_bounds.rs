use crate::{HorizontalBounds, IndexRange};

/// Allocated bounds for one patch, including halo and WRF guard points.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MemoryBounds {
    horizontal: HorizontalBounds,
    bottom_top: IndexRange,
}

impl MemoryBounds {
    pub(crate) const fn new(horizontal: HorizontalBounds, bottom_top: IndexRange) -> Self {
        Self {
            horizontal,
            bottom_top,
        }
    }

    /// Returns both horizontal memory ranges.
    pub const fn horizontal(self) -> HorizontalBounds {
        self.horizontal
    }

    /// Returns the west-east memory range.
    pub const fn west_east(self) -> IndexRange {
        self.horizontal.west_east()
    }

    /// Returns the bottom-top memory range.
    pub const fn bottom_top(self) -> IndexRange {
        self.bottom_top
    }

    /// Returns the south-north memory range.
    pub const fn south_north(self) -> IndexRange {
        self.horizontal.south_north()
    }

    /// Returns the checked number of stored values.
    pub fn point_count(self) -> Option<usize> {
        self.west_east()
            .len()
            .checked_mul(self.bottom_top.len())?
            .checked_mul(self.south_north().len())
    }
}
