/// Widths controlling WRF specified-zone tendency assignment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpecifiedBoundaryTendencyParameters {
    pub(crate) boundary_width: usize,
    pub(crate) specified_zone_width: usize,
}

impl SpecifiedBoundaryTendencyParameters {
    /// Captures the stored boundary width and the outer zone being assigned.
    pub const fn new(boundary_width: usize, specified_zone_width: usize) -> Self {
        Self {
            boundary_width,
            specified_zone_width,
        }
    }

    /// Returns the number of points stored normal to each boundary.
    pub const fn boundary_width(self) -> usize {
        self.boundary_width
    }

    /// Returns the outer zone assigned from boundary-file tendencies.
    pub const fn specified_zone_width(self) -> usize {
        self.specified_zone_width
    }
}
