/// Extra physical-boundary storage requested in each horizontal dimension.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct BoundaryWidths {
    west_east: usize,
    south_north: usize,
}

impl BoundaryWidths {
    /// Creates horizontal physical-boundary widths.
    pub const fn new(west_east: usize, south_north: usize) -> Self {
        Self {
            west_east,
            south_north,
        }
    }

    /// Returns the west-east boundary width.
    pub const fn west_east(self) -> usize {
        self.west_east
    }

    /// Returns the south-north boundary width.
    pub const fn south_north(self) -> usize {
        self.south_north
    }
}
