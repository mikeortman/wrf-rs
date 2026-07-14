use std::num::NonZeroUsize;

use crate::{TopologyError, TopologyResult};

/// Requested thread-level tile grid within one process patch.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TileGrid {
    columns: NonZeroUsize,
    rows: NonZeroUsize,
}

impl TileGrid {
    /// Creates a non-empty tile grid.
    pub fn try_new(columns: usize, rows: usize) -> TopologyResult<Self> {
        let columns = NonZeroUsize::new(columns).ok_or(TopologyError::ZeroTileGridDimension)?;
        let rows = NonZeroUsize::new(rows).ok_or(TopologyError::ZeroTileGridDimension)?;
        Ok(Self { columns, rows })
    }

    /// Returns the requested tile-column count.
    pub const fn columns(self) -> usize {
        self.columns.get()
    }

    /// Returns the requested tile-row count.
    pub const fn rows(self) -> usize {
        self.rows.get()
    }
}
