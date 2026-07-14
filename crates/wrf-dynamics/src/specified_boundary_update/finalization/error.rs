use std::error::Error;
use std::fmt;

use wrf_compute::GridShape;

use crate::SpecifiedBoundaryUpdateError;

/// Failure while validating or forcing specified-boundary state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpecifiedBoundaryFinalizationError {
    /// Domain, stagger, or tile validation failed.
    InvalidRegion(SpecifiedBoundaryUpdateError),
    /// Boundary arrays cannot have zero points normal to the boundary.
    InvalidBoundaryWidth,
    /// The active specified zone exceeds the stored boundary width.
    SpecifiedZoneExceedsBoundaryWidth {
        /// Stored boundary-file width.
        boundary_width: usize,
        /// Requested outer specified zone.
        specified_zone_width: usize,
    },
    /// A model field does not use the finalization region's storage shape.
    ShapeMismatch {
        /// Scientific field role.
        field: &'static str,
        /// Required storage shape.
        expected: GridShape,
        /// Supplied storage shape.
        actual: GridShape,
    },
    /// A boundary array does not use the required oriented shape.
    BoundaryShapeMismatch {
        /// Boundary-array role.
        field: &'static str,
        /// Required number of points along the boundary.
        expected_line_points: usize,
        /// Required number of vertical points.
        expected_vertical_points: usize,
        /// Required number of points normal to the boundary.
        expected_boundary_width: usize,
        /// Supplied storage shape.
        actual: GridShape,
    },
    /// A vertical coefficient does not span the model field's storage column.
    CoefficientLengthMismatch {
        /// Scientific coefficient role.
        coefficient: &'static str,
        /// Required coefficient count.
        expected: usize,
        /// Supplied coefficient count.
        actual: usize,
    },
    /// The physical half-level domain cannot represent the boundary vertical extent.
    BoundaryVerticalExtentOverflow,
    /// A persistent CPU worker panicked.
    WorkerPanicked,
    /// Validated storage did not satisfy the CPU scheduler contract.
    SchedulerContractViolated,
}

impl fmt::Display for SpecifiedBoundaryFinalizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRegion(error) => {
                write!(formatter, "invalid boundary-finalization region: {error}")
            }
            Self::InvalidBoundaryWidth => {
                formatter.write_str("boundary-finalization boundary width must be positive")
            }
            Self::SpecifiedZoneExceedsBoundaryWidth {
                boundary_width,
                specified_zone_width,
            } => write!(
                formatter,
                "specified zone width {specified_zone_width} exceeds boundary width {boundary_width}"
            ),
            Self::ShapeMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "boundary-finalization {field} shape {actual:?} does not match {expected:?}"
            ),
            Self::BoundaryShapeMismatch {
                field,
                expected_line_points,
                expected_vertical_points,
                expected_boundary_width,
                actual,
            } => write!(
                formatter,
                "boundary-finalization {field} shape {actual:?} does not match line={expected_line_points}, vertical={expected_vertical_points}, width={expected_boundary_width}"
            ),
            Self::CoefficientLengthMismatch {
                coefficient,
                expected,
                actual,
            } => write!(
                formatter,
                "boundary-finalization {coefficient} length {actual} does not match {expected}"
            ),
            Self::BoundaryVerticalExtentOverflow => {
                formatter.write_str("boundary-finalization vertical extent overflowed")
            }
            Self::WorkerPanicked => formatter.write_str("boundary-finalization worker panicked"),
            Self::SchedulerContractViolated => {
                formatter.write_str("boundary-finalization scheduler contract was violated")
            }
        }
    }
}

impl Error for SpecifiedBoundaryFinalizationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidRegion(error) => Some(error),
            _ => None,
        }
    }
}

impl From<SpecifiedBoundaryUpdateError> for SpecifiedBoundaryFinalizationError {
    fn from(error: SpecifiedBoundaryUpdateError) -> Self {
        Self::InvalidRegion(error)
    }
}

/// Result type for specified-boundary finalization.
pub type SpecifiedBoundaryFinalizationResult<T> = Result<T, SpecifiedBoundaryFinalizationError>;
