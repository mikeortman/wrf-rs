use std::ops::Range;

/// One horizontal decomposition tile requested for a microphysics update.
///
/// Ranges are zero-based, half-open positions inside the allocated field
/// shape, before the driver applies domain and boundary-zone clipping. A tile
/// that clips to nothing is skipped exactly like an empty Fortran tile loop.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MicrophysicsTile {
    west_east_range: Range<usize>,
    south_north_range: Range<usize>,
}

impl MicrophysicsTile {
    /// Creates a tile from unclipped west-east and south-north ranges.
    pub const fn new(west_east_range: Range<usize>, south_north_range: Range<usize>) -> Self {
        Self {
            west_east_range,
            south_north_range,
        }
    }

    pub(crate) fn west_east_range(&self) -> Range<usize> {
        self.west_east_range.clone()
    }

    pub(crate) fn south_north_range(&self) -> Range<usize> {
        self.south_north_range.clone()
    }
}
