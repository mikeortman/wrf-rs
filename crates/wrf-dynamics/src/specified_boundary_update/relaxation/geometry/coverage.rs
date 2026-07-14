use std::ops::Range;

/// Minimum model-coordinate coverage required by the five-point stencil.
pub(in crate::specified_boundary_update::relaxation) struct SpecifiedBoundaryRelaxationCoverage {
    pub(in crate::specified_boundary_update::relaxation) west_east: Range<usize>,
    pub(in crate::specified_boundary_update::relaxation) south_north: Range<usize>,
    pub(in crate::specified_boundary_update::relaxation) bottom_top: Range<usize>,
}
