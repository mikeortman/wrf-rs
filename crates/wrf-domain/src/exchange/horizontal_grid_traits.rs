/// Periodic-boundary selection for a horizontal field.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct HorizontalPeriodicity {
    west_east: bool,
    south_north: bool,
}

impl HorizontalPeriodicity {
    /// Creates horizontal periodicity flags.
    pub const fn new(west_east: bool, south_north: bool) -> Self {
        Self {
            west_east,
            south_north,
        }
    }

    /// Returns whether the west-east boundary wraps.
    pub const fn west_east(self) -> bool {
        self.west_east
    }

    /// Returns whether the south-north boundary wraps.
    pub const fn south_north(self) -> bool {
        self.south_north
    }
}

/// Arakawa-grid staggering carried by a field in each horizontal axis.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct HorizontalStaggering {
    west_east: bool,
    south_north: bool,
}

impl HorizontalStaggering {
    /// Creates horizontal staggering flags.
    pub const fn new(west_east: bool, south_north: bool) -> Self {
        Self {
            west_east,
            south_north,
        }
    }

    /// Returns whether the field is west-east staggered.
    pub const fn west_east(self) -> bool {
        self.west_east
    }

    /// Returns whether the field is south-north staggered.
    pub const fn south_north(self) -> bool {
        self.south_north
    }
}
