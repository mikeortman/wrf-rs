/// Zero-based process-grid column and row for one patch.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PatchCoordinate {
    column: usize,
    row: usize,
}

impl PatchCoordinate {
    pub(crate) const fn new(column: usize, row: usize) -> Self {
        Self { column, row }
    }

    /// Returns the west-east process-grid coordinate.
    pub const fn column(self) -> usize {
        self.column
    }

    /// Returns the south-north process-grid coordinate.
    pub const fn row(self) -> usize {
        self.row
    }
}
