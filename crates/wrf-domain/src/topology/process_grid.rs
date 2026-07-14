use std::num::NonZeroUsize;

use crate::{TopologyError, TopologyResult};

/// Two-dimensional process grid used by WRF's RSL_LITE decomposition.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProcessGrid {
    columns: NonZeroUsize,
    rows: NonZeroUsize,
}

impl ProcessGrid {
    /// Creates a non-empty process grid.
    pub fn try_new(columns: usize, rows: usize) -> TopologyResult<Self> {
        let columns = NonZeroUsize::new(columns).ok_or(TopologyError::ZeroProcessGridDimension)?;
        let rows = NonZeroUsize::new(rows).ok_or(TopologyError::ZeroProcessGridDimension)?;
        columns
            .get()
            .checked_mul(rows.get())
            .ok_or(TopologyError::ProcessCountOverflow)?;
        Ok(Self { columns, rows })
    }

    /// Returns the west-east process count.
    pub const fn columns(self) -> usize {
        self.columns.get()
    }

    /// Returns the south-north process count.
    pub const fn rows(self) -> usize {
        self.rows.get()
    }

    /// Returns the total process count.
    pub const fn process_count(self) -> usize {
        self.columns.get() * self.rows.get()
    }
}
