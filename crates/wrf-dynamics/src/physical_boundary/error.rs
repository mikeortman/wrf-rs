use std::error::Error;
use std::fmt;
use std::ops::Range;

use wrf_compute::GridShape;

use crate::PhysicalBoundaryAxis;

/// Result alias for physical boundary-zone kernels.
pub type PhysicalBoundaryResult<T> = Result<T, PhysicalBoundaryError>;

/// Failure while validating or applying a physical boundary-zone assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PhysicalBoundaryError {
    /// A physical domain or tile range is empty or outside its storage extent.
    InvalidRange {
        /// Axis associated with the invalid range.
        axis: PhysicalBoundaryAxis,
        /// Rejected half-open range.
        range: Range<usize>,
        /// Available storage extent.
        extent: usize,
    },
    /// The stored halo cannot hold WRF's four-point boundary zone.
    MissingBoundaryZone {
        /// Axis whose halo is too shallow.
        axis: PhysicalBoundaryAxis,
        /// Physical domain range whose halo was checked.
        domain: Range<usize>,
        /// Available storage extent.
        extent: usize,
    },
    /// A tile is not contained by its staggered physical domain.
    TileOutsideDomain {
        /// Axis associated with the tile.
        axis: PhysicalBoundaryAxis,
        /// Rejected tile range.
        tile: Range<usize>,
        /// Permitted staggered domain range.
        domain: Range<usize>,
    },
    /// A field does not use the region's validated storage shape.
    ShapeMismatch {
        /// Expected storage shape for this call.
        expected: GridShape,
        /// Supplied field shape.
        actual: GridShape,
    },
}

impl fmt::Display for PhysicalBoundaryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRange {
                axis,
                range,
                extent,
            } => write!(
                formatter,
                "invalid physical-boundary {axis} range {range:?} for extent {extent}"
            ),
            Self::MissingBoundaryZone {
                axis,
                domain,
                extent,
            } => write!(
                formatter,
                "physical-boundary {axis} domain {domain:?} needs a four-point halo \
                 on each side, but extent is {extent}"
            ),
            Self::TileOutsideDomain { axis, tile, domain } => write!(
                formatter,
                "physical-boundary {axis} tile {tile:?} is outside domain {domain:?}"
            ),
            Self::ShapeMismatch { expected, actual } => write!(
                formatter,
                "physical-boundary field shape {actual:?} does not match {expected:?}"
            ),
        }
    }
}

impl Error for PhysicalBoundaryError {}
