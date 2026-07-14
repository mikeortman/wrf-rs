use std::error::Error;
use std::fmt;

use wrf_compute::GridShape;

use crate::SpecifiedBoundaryUpdateAxis;

/// Failure while validating or applying zero-gradient specified boundaries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpecifiedBoundaryZeroGradientError {
    /// The mutable field does not match the validated region shape.
    ShapeMismatch {
        /// Required storage shape.
        expected: GridShape,
        /// Supplied storage shape.
        actual: GridShape,
    },
    /// The zone leaves no independent interior source point on an active axis.
    MissingInteriorSource {
        /// Axis without an interior source.
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

impl fmt::Display for SpecifiedBoundaryZeroGradientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ShapeMismatch { expected, actual } => write!(
                formatter,
                "zero-gradient specified-boundary field shape {actual:?} does not match {expected:?}"
            ),
            Self::MissingInteriorSource {
                axis,
                specified_zone_width,
                domain_points,
            } => write!(
                formatter,
                "zero-gradient {axis} zone width {specified_zone_width} leaves no independent interior source among {domain_points} points"
            ),
            Self::WorkerPanicked => {
                formatter.write_str("zero-gradient specified-boundary worker panicked")
            }
            Self::SchedulerContractViolated => formatter
                .write_str("zero-gradient specified-boundary scheduler contract was violated"),
        }
    }
}

impl Error for SpecifiedBoundaryZeroGradientError {}

/// Result type for zero-gradient specified-boundary updates.
pub type SpecifiedBoundaryZeroGradientResult<T> = Result<T, SpecifiedBoundaryZeroGradientError>;
