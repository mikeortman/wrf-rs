use std::error::Error;
use std::fmt;

use wrf_compute::GridShape;

/// Failure while validating or applying a geopotential boundary update.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpecifiedBoundaryGeopotentialError {
    /// A field does not use the shape required by the validated region.
    ShapeMismatch {
        /// Scientific field role.
        field: &'static str,
        /// Required storage shape.
        expected: GridShape,
        /// Supplied storage shape.
        actual: GridShape,
    },
    /// A vertical coefficient does not span the complete storage column.
    CoefficientLengthMismatch {
        /// Scientific coefficient role.
        coefficient: &'static str,
        /// Required coefficient count.
        expected: usize,
        /// Supplied coefficient count.
        actual: usize,
    },
    /// A persistent CPU worker panicked.
    WorkerPanicked,
    /// Validated storage did not satisfy the CPU scheduler contract.
    SchedulerContractViolated,
}

impl fmt::Display for SpecifiedBoundaryGeopotentialError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ShapeMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "specified-boundary geopotential {field} shape {actual:?} does not match {expected:?}"
            ),
            Self::CoefficientLengthMismatch {
                coefficient,
                expected,
                actual,
            } => write!(
                formatter,
                "specified-boundary geopotential {coefficient} length {actual} does not match {expected}"
            ),
            Self::WorkerPanicked => {
                formatter.write_str("specified-boundary geopotential worker panicked")
            }
            Self::SchedulerContractViolated => formatter
                .write_str("specified-boundary geopotential scheduler contract was violated"),
        }
    }
}

impl Error for SpecifiedBoundaryGeopotentialError {}

/// Result type for specified-boundary geopotential updates.
pub type SpecifiedBoundaryGeopotentialResult<T> = Result<T, SpecifiedBoundaryGeopotentialError>;
