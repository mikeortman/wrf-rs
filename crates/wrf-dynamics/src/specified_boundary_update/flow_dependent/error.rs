use std::error::Error;
use std::fmt;

use wrf_compute::GridShape;

use crate::{
    SpecifiedBoundaryFlowField, SpecifiedBoundaryUpdateAxis, SpecifiedBoundaryUpdateError,
};

/// Failure while validating or applying flow-dependent scalar boundaries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpecifiedBoundaryFlowError {
    /// The shared specified-boundary region is invalid.
    Region(SpecifiedBoundaryUpdateError),
    /// One field does not match the validated region shape.
    ShapeMismatch {
        /// Scientific role of the mismatched field.
        field: SpecifiedBoundaryFlowField,
        /// Required storage shape.
        expected: GridShape,
        /// Supplied storage shape.
        actual: GridShape,
    },
    /// A contacted upper boundary lacks its staggered velocity neighbor.
    MissingVelocityNeighbor {
        /// Horizontal axis whose upper neighbor is missing.
        axis: SpecifiedBoundaryUpdateAxis,
        /// Zero-based neighbor index required by the source routine.
        required_index: usize,
        /// Available storage extent on that axis.
        extent: usize,
    },
    /// The zone leaves no independent interior scalar source point.
    MissingInteriorSource {
        /// Axis without an independent source.
        axis: SpecifiedBoundaryUpdateAxis,
        /// Requested specified-zone width.
        specified_zone_width: usize,
        /// Effective physical point count.
        domain_points: usize,
    },
    /// A persistent CPU worker panicked.
    WorkerPanicked,
    /// Validated storage did not satisfy the CPU scheduler contract.
    SchedulerContractViolated,
}

impl fmt::Display for SpecifiedBoundaryFlowError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Region(error) => {
                write!(formatter, "invalid flow-dependent boundary region: {error}")
            }
            Self::ShapeMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "flow-dependent {field} shape {actual:?} does not match {expected:?}"
            ),
            Self::MissingVelocityNeighbor {
                axis,
                required_index,
                extent,
            } => write!(
                formatter,
                "flow-dependent {axis} boundary requires velocity index {required_index}, but storage extent is {extent}"
            ),
            Self::MissingInteriorSource {
                axis,
                specified_zone_width,
                domain_points,
            } => write!(
                formatter,
                "flow-dependent {axis} zone width {specified_zone_width} leaves no independent interior source among {domain_points} points"
            ),
            Self::WorkerPanicked => {
                formatter.write_str("flow-dependent specified-boundary worker panicked")
            }
            Self::SchedulerContractViolated => formatter
                .write_str("flow-dependent specified-boundary scheduler contract was violated"),
        }
    }
}

impl Error for SpecifiedBoundaryFlowError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Region(error) => Some(error),
            _ => None,
        }
    }
}

impl From<SpecifiedBoundaryUpdateError> for SpecifiedBoundaryFlowError {
    fn from(error: SpecifiedBoundaryUpdateError) -> Self {
        Self::Region(error)
    }
}

/// Result type for flow-dependent specified scalar boundaries.
pub type SpecifiedBoundaryFlowResult<T> = Result<T, SpecifiedBoundaryFlowError>;
