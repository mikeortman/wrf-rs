use std::error::Error;
use std::fmt;

use wrf_compute::GridShape;

use super::{DryBoundaryRelaxationField, DryBoundaryRelaxationTarget};
use crate::{SpecifiedBoundaryRelaxationError, SpecifiedBoundaryUpdateError};

/// Failure while preparing or applying dry boundary relaxation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DryBoundaryRelaxationError {
    /// One location-specific boundary region is invalid.
    InvalidRegion {
        /// Field whose staggered region could not be represented.
        target: DryBoundaryRelaxationTarget,
        /// Typed region validation failure.
        source: SpecifiedBoundaryUpdateError,
    },
    /// Deriving the caller-owned tile-halo workspace shape overflowed.
    WorkspaceShapeOverflow,
    /// A state, tendency, or workspace field has the wrong storage shape.
    FieldShapeMismatch {
        /// Semantic field role.
        field: DryBoundaryRelaxationField,
        /// Required backend-native shape.
        expected: GridShape,
        /// Supplied backend-native shape.
        actual: GridShape,
    },
    /// One vertical mass-weighting coefficient does not span storage levels.
    CoefficientLengthMismatch {
        /// WRF coefficient name.
        coefficient: &'static str,
        /// Required storage-level count.
        expected: usize,
        /// Supplied coefficient count.
        actual: usize,
    },
    /// One scalar relaxation path rejected its boundary or stencil contract.
    SpecifiedRelaxation {
        /// Dry field being preflighted or updated.
        target: DryBoundaryRelaxationTarget,
        /// Typed scalar-kernel failure.
        source: SpecifiedBoundaryRelaxationError,
    },
    /// A persistent CPU worker panicked during mass weighting.
    WorkerPanicked,
    /// Backend block scheduling violated the validated workspace contract.
    SchedulerContractViolated,
}

impl fmt::Display for DryBoundaryRelaxationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRegion { target, source } => {
                write!(
                    formatter,
                    "invalid {target} boundary-relaxation region: {source}"
                )
            }
            Self::WorkspaceShapeOverflow => {
                formatter.write_str("dry boundary-relaxation workspace shape overflowed")
            }
            Self::FieldShapeMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "dry boundary-relaxation {field} shape {actual:?} does not match {expected:?}"
            ),
            Self::CoefficientLengthMismatch {
                coefficient,
                expected,
                actual,
            } => write!(
                formatter,
                "dry boundary-relaxation {coefficient} count {actual} does not match {expected}"
            ),
            Self::SpecifiedRelaxation { target, source } => {
                write!(formatter, "{target} boundary relaxation failed: {source}")
            }
            Self::WorkerPanicked => formatter.write_str("dry boundary-relaxation worker panicked"),
            Self::SchedulerContractViolated => {
                formatter.write_str("dry boundary-relaxation scheduler contract was violated")
            }
        }
    }
}

impl Error for DryBoundaryRelaxationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidRegion { source, .. } => Some(source),
            Self::SpecifiedRelaxation { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Result type for dry specified-boundary relaxation orchestration.
pub type DryBoundaryRelaxationResult<T> = Result<T, DryBoundaryRelaxationError>;
