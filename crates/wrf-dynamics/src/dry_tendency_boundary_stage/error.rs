use std::error::Error;
use std::fmt;

use crate::{DryBoundaryTendencyError, DryTendencyAssemblyError};

/// Result returned by the coupled dry-tendency boundary stage.
pub type DryTendencyBoundaryStageResult<Value> = Result<Value, DryTendencyBoundaryStageError>;

/// Validation or execution failure from either source-order stage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DryTendencyBoundaryStageError {
    /// Dry-tendency assembly rejected its region, inputs, or execution.
    Assembly(DryTendencyAssemblyError),
    /// Specified-boundary assignment rejected its region, inputs, or execution.
    BoundaryAssignment(DryBoundaryTendencyError),
}

impl fmt::Display for DryTendencyBoundaryStageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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

impl Error for DryTendencyBoundaryStageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Assembly(source) => Some(source),
            Self::BoundaryAssignment(source) => Some(source),
        }
    }
}

impl From<DryTendencyAssemblyError> for DryTendencyBoundaryStageError {
    fn from(source: DryTendencyAssemblyError) -> Self {
        Self::Assembly(source)
    }
}

impl From<DryBoundaryTendencyError> for DryTendencyBoundaryStageError {
    fn from(source: DryBoundaryTendencyError) -> Self {
        Self::BoundaryAssignment(source)
    }
}
