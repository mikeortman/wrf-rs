use std::error::Error;
use std::fmt;

use crate::{
    AcousticFluxAccumulationError, AcousticHorizontalMomentumError, AcousticMassThetaError,
    AcousticPressureError, AcousticStepPreparationError, AcousticVerticalError,
    VerticalAcousticCoefficientError,
};

/// Failure reported while validating or executing an acoustic trajectory.
#[derive(Debug)]
pub enum AcousticTrajectoryError {
    /// The requested acoustic sequence contains no substeps.
    ZeroSubstepCount,
    /// Small-step preparation failed.
    Preparation(AcousticStepPreparationError),
    /// Pressure diagnosis failed.
    Pressure(AcousticPressureError),
    /// Vertical coefficient construction failed.
    VerticalCoefficients(VerticalAcousticCoefficientError),
    /// Horizontal momentum advancement failed.
    HorizontalMomentum(AcousticHorizontalMomentumError),
    /// Column mass and potential-temperature advancement failed.
    MassTheta(AcousticMassThetaError),
    /// Implicit vertical momentum advancement failed.
    VerticalMomentum(AcousticVerticalError),
    /// Time-averaged flux accumulation failed.
    FluxAccumulation(AcousticFluxAccumulationError),
}

impl PartialEq for AcousticTrajectoryError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ZeroSubstepCount, Self::ZeroSubstepCount) => true,
            (Self::Preparation(left), Self::Preparation(right)) => left == right,
            (Self::Pressure(left), Self::Pressure(right)) => left == right,
            (Self::VerticalCoefficients(left), Self::VerticalCoefficients(right)) => left == right,
            (Self::HorizontalMomentum(left), Self::HorizontalMomentum(right)) => left == right,
            (Self::MassTheta(left), Self::MassTheta(right)) => left == right,
            (Self::VerticalMomentum(left), Self::VerticalMomentum(right)) => left == right,
            (Self::FluxAccumulation(left), Self::FluxAccumulation(right)) => left == right,
            _ => false,
        }
    }
}

impl fmt::Display for AcousticTrajectoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroSubstepCount => {
                formatter.write_str("acoustic trajectory requires at least one substep")
            }
            Self::Preparation(error) => write!(formatter, "acoustic preparation failed: {error}"),
            Self::Pressure(error) => {
                write!(formatter, "acoustic pressure diagnosis failed: {error}")
            }
            Self::VerticalCoefficients(error) => {
                write!(formatter, "vertical acoustic coefficients failed: {error}")
            }
            Self::HorizontalMomentum(error) => {
                write!(formatter, "acoustic horizontal momentum failed: {error}")
            }
            Self::MassTheta(error) => {
                write!(formatter, "acoustic mass/theta advancement failed: {error}")
            }
            Self::VerticalMomentum(error) => {
                write!(formatter, "acoustic vertical momentum failed: {error}")
            }
            Self::FluxAccumulation(error) => {
                write!(formatter, "acoustic flux accumulation failed: {error}")
            }
        }
    }
}

impl Error for AcousticTrajectoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ZeroSubstepCount => None,
            Self::Preparation(error) => Some(error),
            Self::Pressure(error) => Some(error),
            Self::VerticalCoefficients(error) => Some(error),
            Self::HorizontalMomentum(error) => Some(error),
            Self::MassTheta(error) => Some(error),
            Self::VerticalMomentum(error) => Some(error),
            Self::FluxAccumulation(error) => Some(error),
        }
    }
}

impl From<AcousticStepPreparationError> for AcousticTrajectoryError {
    fn from(error: AcousticStepPreparationError) -> Self {
        Self::Preparation(error)
    }
}
impl From<AcousticPressureError> for AcousticTrajectoryError {
    fn from(error: AcousticPressureError) -> Self {
        Self::Pressure(error)
    }
}
impl From<VerticalAcousticCoefficientError> for AcousticTrajectoryError {
    fn from(error: VerticalAcousticCoefficientError) -> Self {
        Self::VerticalCoefficients(error)
    }
}
impl From<AcousticHorizontalMomentumError> for AcousticTrajectoryError {
    fn from(error: AcousticHorizontalMomentumError) -> Self {
        Self::HorizontalMomentum(error)
    }
}
impl From<AcousticMassThetaError> for AcousticTrajectoryError {
    fn from(error: AcousticMassThetaError) -> Self {
        Self::MassTheta(error)
    }
}
impl From<AcousticVerticalError> for AcousticTrajectoryError {
    fn from(error: AcousticVerticalError) -> Self {
        Self::VerticalMomentum(error)
    }
}
impl From<AcousticFluxAccumulationError> for AcousticTrajectoryError {
    fn from(error: AcousticFluxAccumulationError) -> Self {
        Self::FluxAccumulation(error)
    }
}

/// Result type for complete acoustic trajectory execution.
pub type AcousticTrajectoryResult<T> = Result<T, AcousticTrajectoryError>;
