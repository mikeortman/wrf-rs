use std::ops::Range;

/// Source-ordered boundary ranges active in one south–north row.
pub(crate) struct SpecifiedBoundaryRowRanges {
    pub(crate) south: Option<Range<usize>>,
    pub(crate) north: Option<Range<usize>>,
    pub(crate) west: Option<Range<usize>>,
    pub(crate) east: Option<Range<usize>>,
}

impl SpecifiedBoundaryRowRanges {
    pub(super) fn empty() -> Self {
        Self {
            south: None,
            north: None,
            west: None,
            east: None,
        }
    }
}
