use std::error::Error;
use std::fmt;
use std::ops::Range;

use wrf_compute::GridShape;

use crate::SpecifiedBoundaryUpdateAxis;

/// Failure while validating or applying a specified-zone update.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpecifiedBoundaryUpdateError {
    /// A physical domain or tile range is empty or outside its storage extent.
    InvalidRange {
        /// Axis associated with the invalid range.
        axis: SpecifiedBoundaryUpdateAxis,
        /// Rejected half-open range.
        range: Range<usize>,
        /// Available storage extent.
        extent: usize,
    },
    /// A tile is not contained by its location-specific physical domain.
    TileOutsideDomain {
        /// Axis associated with the tile.
        axis: SpecifiedBoundaryUpdateAxis,
        /// Rejected tile range.
        tile: Range<usize>,
        /// Effective staggered domain range.
        domain: Range<usize>,
    },
    /// A staggered location requires one more stored point.
    MissingUpperStagger {
        /// Axis missing its upper point.
        axis: SpecifiedBoundaryUpdateAxis,
        /// Required exclusive end.
        required_end: usize,
        /// Available storage extent.
        extent: usize,
    },
    /// Horizontal column mass must use exactly one stored vertical level.
    HorizontalMassRequiresSingleLevel {
        /// Rejected half-level domain.
        range: Range<usize>,
    },
    /// A field does not use the region's validated storage shape.
    ShapeMismatch {
        /// Scientific field role.
        field: &'static str,
        /// Region shape.
        expected: GridShape,
        /// Field shape.
        actual: GridShape,
    },
    /// A persistent CPU worker panicked.
    WorkerPanicked,
}

impl fmt::Display for SpecifiedBoundaryUpdateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRange {
                axis,
                range,
                extent,
            } => write!(
                formatter,
                "invalid specified-boundary {axis} range {range:?} for extent {extent}"
            ),
            Self::TileOutsideDomain { axis, tile, domain } => write!(
                formatter,
                "specified-boundary {axis} tile {tile:?} is outside domain {domain:?}"
            ),
            Self::MissingUpperStagger {
                axis,
                required_end,
                extent,
            } => write!(
                formatter,
                "specified-boundary {axis} stagger requires end {required_end}, but extent is {extent}"
            ),
            Self::HorizontalMassRequiresSingleLevel { range } => write!(
                formatter,
                "specified-boundary horizontal mass requires one vertical level, but received {range:?}"
            ),
            Self::ShapeMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "specified-boundary {field} shape {actual:?} does not match {expected:?}"
            ),
            Self::WorkerPanicked => formatter.write_str("specified-boundary worker panicked"),
        }
    }
}

impl Error for SpecifiedBoundaryUpdateError {}

/// Result type for specified-zone tendency updates.
pub type SpecifiedBoundaryUpdateResult<T> = Result<T, SpecifiedBoundaryUpdateError>;
