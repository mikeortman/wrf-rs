use std::error::Error;
use std::fmt;

use crate::{DryBoundaryRelaxationError, DryBoundaryTendencyError, DryTendencyAssemblyError};

/// Result returned by the dry large-step boundary stage.
pub type DryLargeStepBoundaryStageResult<Value> = Result<Value, DryLargeStepBoundaryStageError>;

/// Validation or execution failure from one of the three source-order stages.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DryLargeStepBoundaryStageError {
    /// First-substep dry boundary relaxation rejected its inputs or execution.
    Relaxation(DryBoundaryRelaxationError),
    /// Dry-tendency assembly rejected its region, inputs, or execution.
    Assembly(DryTendencyAssemblyError),
    /// Specified-boundary assignment rejected its region, inputs, or execution.
    BoundaryAssignment(DryBoundaryTendencyError),
}

impl fmt::Display for DryLargeStepBoundaryStageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Relaxation(source) => {
                write!(formatter, "dry boundary relaxation failed: {source}")
            }
            Self::Assembly(source) => write!(formatter, "dry-tendency assembly failed: {source}"),
            Self::BoundaryAssignment(source) => {
                write!(
                    formatter,
                    "dry boundary-tendency assignment failed: {source}"
                )
            }
        }
    }
}

impl Error for DryLargeStepBoundaryStageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Relaxation(source) => Some(source),
            Self::Assembly(source) => Some(source),
            Self::BoundaryAssignment(source) => Some(source),
        }
    }
}

impl From<DryBoundaryRelaxationError> for DryLargeStepBoundaryStageError {
    fn from(source: DryBoundaryRelaxationError) -> Self {
        Self::Relaxation(source)
    }
}

impl From<DryTendencyAssemblyError> for DryLargeStepBoundaryStageError {
    fn from(source: DryTendencyAssemblyError) -> Self {
        Self::Assembly(source)
    }
}

impl From<DryBoundaryTendencyError> for DryLargeStepBoundaryStageError {
    fn from(source: DryBoundaryTendencyError) -> Self {
        Self::BoundaryAssignment(source)
    }
}
