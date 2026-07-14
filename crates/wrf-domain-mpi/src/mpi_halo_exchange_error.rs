use std::fmt;

use wrf_domain::{HaloExchangeError, PatchId};

/// A failure at the MPI halo-transport boundary.
#[derive(Debug)]
pub enum MpiHaloExchangeError {
    /// Communicator size did not match the process topology.
    CommunicatorSizeMismatch {
        /// Required process count.
        expected: usize,
        /// Actual communicator size.
        actual: usize,
    },
    /// MPI rank could not be represented as a patch identifier.
    InvalidRank {
        /// Invalid rank or communicator size reported by MPI.
        rank: i32,
    },
    /// The supplied field did not belong to the calling MPI rank.
    FieldRankMismatch {
        /// Patch identifier carried by the field.
        patch_id: PatchId,
        /// Calling communicator rank.
        rank: i32,
    },
    /// Core packing, validation, or unpacking failed.
    Domain(HaloExchangeError),
}

/// Result returned by MPI halo operations.
pub type MpiHaloExchangeResult<T> = Result<T, MpiHaloExchangeError>;

impl fmt::Display for MpiHaloExchangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommunicatorSizeMismatch { expected, actual } => write!(
                formatter,
                "MPI communicator has {actual} ranks but topology needs {expected}"
            ),
            Self::InvalidRank { rank } => write!(formatter, "MPI rank {rank} is invalid"),
            Self::FieldRankMismatch { patch_id, rank } => write!(
                formatter,
                "field patch {} does not match MPI rank {rank}",
                patch_id.value()
            ),
            Self::Domain(error) => write!(formatter, "domain halo operation failed: {error}"),
        }
    }
}

impl std::error::Error for MpiHaloExchangeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Domain(error) => Some(error),
            _ => None,
        }
    }
}

impl From<HaloExchangeError> for MpiHaloExchangeError {
    fn from(error: HaloExchangeError) -> Self {
        Self::Domain(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_reports_communicator_size_mismatch() {
        let error = MpiHaloExchangeError::CommunicatorSizeMismatch {
            expected: 4,
            actual: 2,
        };

        assert_eq!(
            error.to_string(),
            "MPI communicator has 2 ranks but topology needs 4"
        );
    }
}
