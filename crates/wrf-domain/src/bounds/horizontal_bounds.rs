use crate::IndexRange;

/// West-east and south-north bounds for a horizontal grid region.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HorizontalBounds {
    west_east: IndexRange,
    south_north: IndexRange,
}

impl HorizontalBounds {
    /// Creates horizontal bounds from validated axis ranges.
    pub const fn new(west_east: IndexRange, south_north: IndexRange) -> Self {
        Self {
            west_east,
            south_north,
        }
    }

    /// Returns the west-east range.
    pub const fn west_east(self) -> IndexRange {
        self.west_east
    }

    /// Returns the south-north range.
    pub const fn south_north(self) -> IndexRange {
        self.south_north
    }

    /// Returns whether both axes contain the supplied region.
    pub const fn contains(self, other: Self) -> bool {
        self.west_east.contains_range(other.west_east)
            && self.south_north.contains_range(other.south_north)
    }
}
