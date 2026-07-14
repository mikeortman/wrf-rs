use std::error::Error;
use std::fmt;

use super::DryBoundaryTendencyTarget;
use crate::{SpecifiedBoundaryTendencyError, SpecifiedBoundaryUpdateError};

/// Failure while preparing or assigning complete dry boundary tendencies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DryBoundaryTendencyError {
    /// One location-specific boundary region is invalid.
    InvalidRegion {
        /// Field whose staggered region could not be represented.
        target: DryBoundaryTendencyTarget,
        /// Typed region validation failure.
        source: SpecifiedBoundaryUpdateError,
    },
    /// One scalar assignment rejected its output or boundary contract.
    SpecifiedTendency {
        /// Dry field being preflighted or updated.
        target: DryBoundaryTendencyTarget,
        /// Typed scalar-kernel failure.
        source: SpecifiedBoundaryTendencyError,
    },
}

impl fmt::Display for DryBoundaryTendencyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRegion { target, source } => {
                write!(
                    formatter,
                    "invalid {target} boundary-tendency region: {source}"
                )
            }
            Self::SpecifiedTendency { target, source } => {
                write!(
                    formatter,
                    "{target} boundary-tendency assignment failed: {source}"
                )
            }
        }
    }
}

impl Error for DryBoundaryTendencyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidRegion { source, .. } => Some(source),
            Self::SpecifiedTendency { source, .. } => Some(source),
        }
    }
}

/// Result type for complete dry specified-boundary tendency assignment.
pub type DryBoundaryTendencyResult<T> = Result<T, DryBoundaryTendencyError>;
