use std::error::Error;
use std::fmt;

use crate::{
    AcousticBoundaryRegionRole, AcousticTrajectoryError, PhysicalBoundaryError,
    SpecifiedBoundaryFieldLocation, SpecifiedBoundaryGeopotentialError,
    SpecifiedBoundaryUpdateError, SpecifiedBoundaryZeroGradientError,
};

/// Failure while preflighting or executing the complete acoustic boundary stage.
#[derive(Debug, PartialEq)]
pub enum AcousticBoundaryStageError {
    /// The local stage cannot reproduce WRF's omitted `pxft` polar filter.
    PolarFilteringUnsupported,
    /// This composed window owns WRF's nonhydrostatic vertical solve.
    HydrostaticModeUnsupported,
    /// A specified-boundary region has the wrong C-grid location.
    RegionLocationMismatch {
        /// Scientific stage role using the region.
        role: AcousticBoundaryRegionRole,
        /// Location required by the pinned WRF call.
        expected: SpecifiedBoundaryFieldLocation,
        /// Location supplied by the caller.
        actual: SpecifiedBoundaryFieldLocation,
    },
    /// The underlying local acoustic trajectory failed.
    Trajectory(AcousticTrajectoryError),
    /// A physical boundary assignment failed.
    PhysicalBoundary(PhysicalBoundaryError),
    /// A specified-zone tendency update failed.
    SpecifiedBoundary(SpecifiedBoundaryUpdateError),
    /// A mass-normalized geopotential update failed.
    SpecifiedGeopotential(SpecifiedBoundaryGeopotentialError),
    /// A specified vertical-momentum zero-gradient copy failed.
    SpecifiedZeroGradient(SpecifiedBoundaryZeroGradientError),
}

impl fmt::Display for AcousticBoundaryStageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PolarFilteringUnsupported => formatter.write_str(
                "the local acoustic boundary stage does not include WRF polar filtering",
            ),
            Self::HydrostaticModeUnsupported => {
                formatter.write_str("the acoustic boundary stage requires WRF nonhydrostatic mode")
            }
            Self::RegionLocationMismatch {
                role,
                expected,
                actual,
            } => write!(
                formatter,
                "acoustic {role} boundary region uses {actual:?}, expected {expected:?}"
            ),
            Self::Trajectory(error) => write!(formatter, "acoustic trajectory failed: {error}"),
            Self::PhysicalBoundary(error) => {
                write!(formatter, "physical boundary assignment failed: {error}")
            }
            Self::SpecifiedBoundary(error) => {
                write!(formatter, "specified boundary update failed: {error}")
            }
            Self::SpecifiedGeopotential(error) => {
                write!(formatter, "specified geopotential update failed: {error}")
            }
            Self::SpecifiedZeroGradient(error) => {
                write!(formatter, "specified zero-gradient update failed: {error}")
            }
        }
    }
}

impl Error for AcousticBoundaryStageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::PolarFilteringUnsupported
            | Self::HydrostaticModeUnsupported
            | Self::RegionLocationMismatch { .. } => None,
            Self::Trajectory(error) => Some(error),
            Self::PhysicalBoundary(error) => Some(error),
            Self::SpecifiedBoundary(error) => Some(error),
            Self::SpecifiedGeopotential(error) => Some(error),
            Self::SpecifiedZeroGradient(error) => Some(error),
        }
    }
}

impl From<AcousticTrajectoryError> for AcousticBoundaryStageError {
    fn from(error: AcousticTrajectoryError) -> Self {
        Self::Trajectory(error)
    }
}

impl From<PhysicalBoundaryError> for AcousticBoundaryStageError {
    fn from(error: PhysicalBoundaryError) -> Self {
        Self::PhysicalBoundary(error)
    }
}

impl From<SpecifiedBoundaryUpdateError> for AcousticBoundaryStageError {
    fn from(error: SpecifiedBoundaryUpdateError) -> Self {
        Self::SpecifiedBoundary(error)
    }
}

impl From<SpecifiedBoundaryGeopotentialError> for AcousticBoundaryStageError {
    fn from(error: SpecifiedBoundaryGeopotentialError) -> Self {
        Self::SpecifiedGeopotential(error)
    }
}

impl From<SpecifiedBoundaryZeroGradientError> for AcousticBoundaryStageError {
    fn from(error: SpecifiedBoundaryZeroGradientError) -> Self {
        Self::SpecifiedZeroGradient(error)
    }
}

/// Result type for complete acoustic boundary-stage execution.
pub type AcousticBoundaryStageResult<T> = Result<T, AcousticBoundaryStageError>;
