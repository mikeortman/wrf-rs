use std::error::Error;
use std::fmt;
use std::ops::Range;

use wrf_compute::GridShape;

use crate::SpecifiedBoundaryUpdateAxis;

/// Failure while validating or applying specified-boundary relaxation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpecifiedBoundaryRelaxationError {
    /// Boundary arrays cannot have an empty normal dimension.
    InvalidBoundaryWidth,
    /// The fixed outer zone must retain one neighbor outside each relaxed point.
    InvalidSpecifiedZoneWidth,
    /// A configured zone extends beyond stored boundary values.
    ZoneExceedsBoundaryWidth {
        /// Stored boundary-array width.
        boundary_width: usize,
        /// Requested zone width.
        zone_width: usize,
    },
    /// The innermost relaxed point lacks its required boundary-array neighbor.
    RelaxationNeighborOutsideBoundary {
        /// Stored boundary-array width.
        boundary_width: usize,
        /// Requested relaxation-zone width.
        relaxation_zone_width: usize,
    },
    /// The mutable tendency does not use the region's storage shape.
    ShapeMismatch {
        /// Expected shape from the validated region.
        expected: GridShape,
        /// Actual mutable tendency shape.
        actual: GridShape,
    },
    /// Field-view ranges do not describe its native allocation.
    FieldViewShapeMismatch {
        /// Required west–east points from the view range.
        expected_west_east_points: usize,
        /// Required south–north points from the view range.
        expected_south_north_points: usize,
        /// Required bottom–top points from the view range.
        expected_bottom_top_points: usize,
        /// Actual native allocation shape.
        actual: GridShape,
    },
    /// A halo-extended field view does not cover every stencil neighbor.
    FieldCoverageMissing {
        /// Axis whose stencil coverage is incomplete.
        axis: SpecifiedBoundaryUpdateAxis,
        /// Minimum required model-coordinate range.
        required: Range<usize>,
        /// Range represented by the field view.
        actual: Range<usize>,
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
    /// A coefficient array does not span the stored boundary width.
    CoefficientLengthMismatch {
        /// Coefficient role.
        coefficient: &'static str,
        /// Required coefficient count.
        expected: usize,
        /// Actual coefficient count.
        actual: usize,
    },
    /// Computing the boundary-file vertical extent overflowed.
    BoundaryVerticalExtentOverflow,
    /// A persistent CPU worker panicked.
    WorkerPanicked,
    /// Backend block scheduling violated the validated plane contract.
    SchedulerContractViolated,
}

impl fmt::Display for SpecifiedBoundaryRelaxationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBoundaryWidth => {
                formatter.write_str("specified-boundary relaxation width must be positive")
            }
            Self::InvalidSpecifiedZoneWidth => formatter
                .write_str("specified-boundary relaxation requires a positive fixed outer zone"),
            Self::ZoneExceedsBoundaryWidth {
                boundary_width,
                zone_width,
            } => write!(
                formatter,
                "specified-boundary zone {zone_width} exceeds boundary width {boundary_width}"
            ),
            Self::RelaxationNeighborOutsideBoundary {
                boundary_width,
                relaxation_zone_width,
            } => write!(
                formatter,
                "relaxation zone {relaxation_zone_width} lacks its inward neighbor in boundary width {boundary_width}"
            ),
            Self::ShapeMismatch { expected, actual } => write!(
                formatter,
                "specified-boundary relaxation tendency shape {actual:?} does not match {expected:?}"
            ),
            Self::FieldViewShapeMismatch {
                expected_west_east_points,
                expected_south_north_points,
                expected_bottom_top_points,
                actual,
            } => write!(
                formatter,
                "specified-boundary field view requires shape west-east={expected_west_east_points}, south-north={expected_south_north_points}, bottom-top={expected_bottom_top_points}, got {actual:?}"
            ),
            Self::FieldCoverageMissing {
                axis,
                required,
                actual,
            } => write!(
                formatter,
                "specified-boundary field view {axis} range {actual:?} does not cover stencil range {required:?}"
            ),
            Self::BoundaryShapeMismatch {
                field,
                expected_line_points,
                expected_vertical_points,
                expected_boundary_width,
                actual,
            } => write!(
                formatter,
                "specified-boundary relaxation {field} shape {actual:?} does not match line={expected_line_points}, vertical={expected_vertical_points}, width={expected_boundary_width}"
            ),
            Self::CoefficientLengthMismatch {
                coefficient,
                expected,
                actual,
            } => write!(
                formatter,
                "specified-boundary {coefficient} coefficient count {actual} does not match {expected}"
            ),
            Self::BoundaryVerticalExtentOverflow => {
                formatter.write_str("specified-boundary relaxation vertical extent overflowed")
            }
            Self::WorkerPanicked => {
                formatter.write_str("specified-boundary relaxation worker panicked")
            }
            Self::SchedulerContractViolated => {
                formatter.write_str("specified-boundary relaxation scheduler contract was violated")
            }
        }
    }
}

impl Error for SpecifiedBoundaryRelaxationError {}

/// Result type for specified-boundary relaxation forcing.
pub type SpecifiedBoundaryRelaxationResult<T> = Result<T, SpecifiedBoundaryRelaxationError>;
