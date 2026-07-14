use std::error::Error;
use std::fmt;

use wrf_compute::GridShape;

/// Failure while validating or assigning specified-boundary tendencies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpecifiedBoundaryTendencyError {
    /// Boundary arrays cannot have an empty normal dimension.
    InvalidBoundaryWidth,
    /// The requested specified zone is wider than the stored boundary arrays.
    SpecifiedZoneExceedsBoundaryWidth {
        /// Stored boundary-array width.
        boundary_width: usize,
        /// Requested specified-zone width.
        specified_zone_width: usize,
    },
    /// The mutable tendency does not use the region's storage shape.
    ShapeMismatch {
        /// Expected shape from the validated region.
        expected: GridShape,
        /// Actual mutable tendency shape.
        actual: GridShape,
    },
    /// A side array does not match WRF's line, vertical, and normal dimensions.
    BoundaryShapeMismatch {
        /// Geographic side and role.
        field: &'static str,
        /// Required points along the boundary line.
        expected_line_points: usize,
        /// Required boundary-file vertical points.
        expected_vertical_points: usize,
        /// Required points normal to the boundary.
        expected_boundary_width: usize,
        /// Actual side-array shape.
        actual: GridShape,
    },
    /// Computing the boundary-file vertical extent overflowed.
    BoundaryVerticalExtentOverflow,
    /// A persistent CPU worker panicked.
    WorkerPanicked,
    /// Backend block scheduling violated the validated plane contract.
    SchedulerContractViolated,
}

impl fmt::Display for SpecifiedBoundaryTendencyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBoundaryWidth => {
                formatter.write_str("specified-boundary tendency width must be positive")
            }
            Self::SpecifiedZoneExceedsBoundaryWidth {
                boundary_width,
                specified_zone_width,
            } => write!(
                formatter,
                "specified-boundary tendency zone {specified_zone_width} exceeds boundary width {boundary_width}"
            ),
            Self::ShapeMismatch { expected, actual } => write!(
                formatter,
                "specified-boundary mutable tendency shape {actual:?} does not match {expected:?}"
            ),
            Self::BoundaryShapeMismatch {
                field,
                expected_line_points,
                expected_vertical_points,
                expected_boundary_width,
                actual,
            } => write!(
                formatter,
                "specified-boundary {field} shape {actual:?} does not match line={expected_line_points}, vertical={expected_vertical_points}, width={expected_boundary_width}"
            ),
            Self::BoundaryVerticalExtentOverflow => {
                formatter.write_str("specified-boundary tendency vertical extent overflowed")
            }
            Self::WorkerPanicked => {
                formatter.write_str("specified-boundary tendency worker panicked")
            }
            Self::SchedulerContractViolated => {
                formatter.write_str("specified-boundary tendency scheduler contract was violated")
            }
        }
    }
}

impl Error for SpecifiedBoundaryTendencyError {}

/// Result type for specified-boundary tendency assignment.
pub type SpecifiedBoundaryTendencyResult<T> = Result<T, SpecifiedBoundaryTendencyError>;
