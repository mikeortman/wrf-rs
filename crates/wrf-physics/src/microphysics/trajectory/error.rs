use std::error::Error;
use std::fmt;

use wrf_compute::{ComputeError, GridShape};

use crate::{ArwMicrophysicsControl, ArwMicrophysicsField, MicrophysicsDriverError};

/// Failure while configuring or applying the ARW microphysics trajectory.
#[derive(Clone, Debug, PartialEq)]
pub enum ArwMicrophysicsError {
    /// A scalar control violates its finite-value or sign contract.
    InvalidControl {
        /// Rejected control.
        control: ArwMicrophysicsControl,
        /// Rejected value.
        value: f32,
    },
    /// A trajectory field does not match its mass- or W-level shape.
    FieldShapeMismatch {
        /// Field with the invalid shape.
        field: ArwMicrophysicsField,
        /// Required shape.
        expected: GridShape,
        /// Supplied shape.
        actual: GridShape,
    },
    /// A workspace belongs to a CPU backend with a different worker count.
    WorkspaceWorkerCountMismatch {
        /// Worker count captured when the workspace was created.
        workspace_worker_count: usize,
        /// Worker count of the backend requested for this step.
        backend_worker_count: usize,
    },
    /// Backend storage allocation or shape construction failed.
    Compute(ComputeError),
    /// Registry conversion, driver validation, or Kessler execution failed.
    Driver(MicrophysicsDriverError),
}

/// Result alias for ARW microphysics trajectory operations.
pub type ArwMicrophysicsResult<T> = Result<T, ArwMicrophysicsError>;

impl fmt::Display for ArwMicrophysicsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidControl { control, value } => {
                write!(
                    formatter,
                    "ARW microphysics control {control:?} has invalid value {value}"
                )
            }
            Self::FieldShapeMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "ARW {field} shape {actual:?} does not match expected shape {expected:?}"
            ),
            Self::WorkspaceWorkerCountMismatch {
                workspace_worker_count,
                backend_worker_count,
            } => write!(
                formatter,
                "ARW workspace has {workspace_worker_count} workers but the execution backend has {backend_worker_count}"
            ),
            Self::Compute(error) => write!(formatter, "ARW trajectory storage failed: {error}"),
            Self::Driver(error) => write!(formatter, "ARW microphysics driver failed: {error}"),
        }
    }
}

impl Error for ArwMicrophysicsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Compute(error) => Some(error),
            Self::Driver(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ComputeError> for ArwMicrophysicsError {
    fn from(error: ComputeError) -> Self {
        Self::Compute(error)
    }
}

impl From<MicrophysicsDriverError> for ArwMicrophysicsError {
    fn from(error: MicrophysicsDriverError) -> Self {
        Self::Driver(error)
    }
}
