use std::ops::Range;

/// Source-ordered relaxation ranges active in one south–north row.
pub(in crate::specified_boundary_update::relaxation) struct SpecifiedBoundaryRelaxationRowRanges {
    pub(in crate::specified_boundary_update::relaxation) south: Option<Range<usize>>,
    pub(in crate::specified_boundary_update::relaxation) north: Option<Range<usize>>,
    pub(in crate::specified_boundary_update::relaxation) west: Option<Range<usize>>,
    pub(in crate::specified_boundary_update::relaxation) east: Option<Range<usize>>,
}

impl SpecifiedBoundaryRelaxationRowRanges {
    pub(in crate::specified_boundary_update::relaxation) const fn empty() -> Self {
        Self {
            south: None,
            north: None,
            west: None,
            east: None,
        }
    }

    pub(in crate::specified_boundary_update::relaxation) fn iter(
        &self,
    ) -> impl Iterator<Item = &Range<usize>> {
        [
            self.south.as_ref(),
            self.north.as_ref(),
            self.west.as_ref(),
            self.east.as_ref(),
        ]
        .into_iter()
        .flatten()
    }
}
