use std::error::Error;
use std::fmt;

use wrf_compute::{ComputeError, GridShape};
use wrf_dynamics::{
    AcousticStepFinalizationError, AcousticTrajectoryError, ColumnMassStaggeringError,
    DryTendencyAssemblyError, RungeKuttaPreparationError,
};
use wrf_physics::ArwMicrophysicsError;
use wrf_registry::{RegistryResolutionError, RegistryValueType, StateStaggering};

use crate::{
    ArwColumnField, ArwGeopotentialField, ArwMapField, ArwMassField, ArwRestartVolumeField,
};

/// Registry state field expected by the accepted ARW trajectory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArwRegistryField {
    /// A three-dimensional mass-grid field.
    Mass(ArwMassField),
    /// A W-level geopotential field.
    Geopotential(ArwGeopotentialField),
    /// A horizontal column field.
    Column(ArwColumnField),
    /// A restart diagnostic or tendency field.
    RestartVolume(ArwRestartVolumeField),
    /// A horizontal map factor or terrain field.
    Map(ArwMapField),
}

impl fmt::Display for ArwRegistryField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mass(field) => field.fmt(formatter),
            Self::Geopotential(field) => field.fmt(formatter),
            Self::Column(field) => field.fmt(formatter),
            Self::RestartVolume(field) => field.fmt(formatter),
            Self::Map(field) => field.fmt(formatter),
        }
    }
}

/// Typed failures from Registry binding, setup, or trajectory execution.
#[derive(Debug)]
pub enum ArwModelError {
    /// A required ordinary Registry state declaration is absent.
    MissingRegistryState {
        /// Missing WRF Registry state name.
        name: &'static str,
    },
    /// A required ordinary Registry state declaration appears more than once.
    DuplicateRegistryState {
        /// Duplicated WRF Registry state name.
        name: &'static str,
    },
    /// A required state declaration is not single precision real.
    RegistryValueTypeMismatch {
        /// WRF Registry state name.
        name: &'static str,
        /// Declared Registry value type.
        actual: RegistryValueType,
    },
    /// A required state declaration has the wrong dimension count.
    RegistryDimensionCountMismatch {
        /// WRF Registry state name.
        name: &'static str,
        /// Required dimension count.
        expected: usize,
        /// Declared dimension count.
        actual: usize,
    },
    /// A required state declaration has the wrong memory-order dimensions.
    RegistryDimensionsMismatch {
        /// WRF Registry state name.
        name: &'static str,
        /// Required dimension symbols in memory order.
        expected: &'static [&'static str],
        /// Declared dimension symbols in memory order.
        actual: Vec<String>,
    },
    /// A required state declaration has the wrong time-level count.
    RegistryTimeLevelMismatch {
        /// Typed model field selected by the declaration.
        field: ArwRegistryField,
        /// Exact time-level count required by the accepted projection.
        expected: u8,
        /// Declared Registry time-level count.
        actual: u8,
    },
    /// A required state declaration has incompatible staggering.
    RegistryStaggeringMismatch {
        /// WRF Registry state name.
        name: &'static str,
        /// Required staggering description.
        expected: &'static str,
        /// Declared Registry staggering.
        actual: StateStaggering,
    },
    /// Registry package resolution failed.
    RegistryResolution(RegistryResolutionError),
    /// Kessler did not resolve exactly one active `moist` layout.
    MoistureLayoutCount {
        /// Number of resolved `moist` layouts.
        actual: usize,
    },
    /// A field has a shape that differs from the bound model geometry.
    FieldShapeMismatch {
        /// Typed model field with the incompatible allocation.
        field: ArwRegistryField,
        /// Shape required by the bound geometry.
        expected: GridShape,
        /// Shape supplied by the state.
        actual: GridShape,
    },
    /// A workspace was created for a different CPU worker count.
    WorkspaceWorkerCountMismatch {
        /// Worker count used when allocating the workspace.
        expected: usize,
        /// Worker count of the execution backend.
        actual: usize,
    },
    /// The requested model geometry cannot satisfy an accepted kernel.
    InvalidGeometry {
        /// Accepted component that rejected the derived geometry.
        component: &'static str,
    },
    /// Independently supplied component controls do not describe the pinned stage.
    IncompatibleControls {
        /// Control relationship that violated the accepted projection.
        component: &'static str,
    },
    /// A required vertical coefficient does not cover the padded column.
    CoefficientLengthMismatch {
        /// Scientific coefficient name.
        name: &'static str,
        /// Required number of values.
        expected: usize,
        /// Supplied number of values.
        actual: usize,
    },
    /// An internal fixed role table no longer matches its declared count.
    InternalRoleCountMismatch {
        /// Fixed field-role collection whose length changed.
        collection: &'static str,
    },
    /// Backend field allocation or execution failed.
    Compute(ComputeError),
    /// Runge-Kutta diagnostic preparation failed.
    RungeKuttaPreparation(RungeKuttaPreparationError),
    /// Dry tendency assembly failed.
    DryTendencyAssembly(DryTendencyAssemblyError),
    /// Acoustic trajectory execution failed.
    AcousticTrajectory(AcousticTrajectoryError),
    /// Final column-mass staggering failed.
    ColumnMassStaggering(ColumnMassStaggeringError),
    /// Reconstruction of full state after acoustic substeps failed.
    AcousticStepFinalization(AcousticStepFinalizationError),
    /// Kessler preparation, driver, or finish failed.
    Microphysics(ArwMicrophysicsError),
}

impl fmt::Display for ArwModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRegistryState { name } => {
                write!(formatter, "required Registry state `{name}` is missing")
            }
            Self::DuplicateRegistryState { name } => {
                write!(formatter, "required Registry state `{name}` is duplicated")
            }
            Self::RegistryValueTypeMismatch { name, actual } => write!(
                formatter,
                "Registry state `{name}` must be real, found {actual}"
            ),
            Self::RegistryDimensionCountMismatch {
                name,
                expected,
                actual,
            } => write!(
                formatter,
                "Registry state `{name}` must have {expected} dimensions, found {actual}"
            ),
            Self::RegistryDimensionsMismatch {
                name,
                expected,
                actual,
            } => write!(
                formatter,
                "Registry state `{name}` dimensions must be {expected:?}, found {actual:?}"
            ),
            Self::RegistryTimeLevelMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "Registry state for {field} must declare {expected} time levels, found {actual}"
            ),
            Self::RegistryStaggeringMismatch {
                name,
                expected,
                actual,
            } => write!(
                formatter,
                "Registry state `{name}` staggering mismatch: expected {expected}, found {actual:?}"
            ),
            Self::RegistryResolution(error) => error.fmt(formatter),
            Self::MoistureLayoutCount { actual } => write!(
                formatter,
                "Kessler requires exactly one resolved `moist` layout, found {actual}"
            ),
            Self::FieldShapeMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "field {field} shape mismatch: expected {expected:?}, found {actual:?}"
            ),
            Self::WorkspaceWorkerCountMismatch { expected, actual } => write!(
                formatter,
                "workspace worker count mismatch: expected {expected}, found {actual}"
            ),
            Self::InvalidGeometry { component } => {
                write!(formatter, "model geometry is invalid for {component}")
            }
            Self::IncompatibleControls { component } => {
                write!(formatter, "model controls are incompatible for {component}")
            }
            Self::CoefficientLengthMismatch {
                name,
                expected,
                actual,
            } => write!(
                formatter,
                "coefficient `{name}` has {actual} values, expected {expected}"
            ),
            Self::InternalRoleCountMismatch { collection } => {
                write!(formatter, "internal role count mismatch in {collection}")
            }
            Self::Compute(error) => error.fmt(formatter),
            Self::RungeKuttaPreparation(error) => error.fmt(formatter),
            Self::DryTendencyAssembly(error) => error.fmt(formatter),
            Self::AcousticTrajectory(error) => error.fmt(formatter),
            Self::ColumnMassStaggering(error) => error.fmt(formatter),
            Self::AcousticStepFinalization(error) => error.fmt(formatter),
            Self::Microphysics(error) => error.fmt(formatter),
        }
    }
}

impl Error for ArwModelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RegistryResolution(error) => Some(error),
            Self::Compute(error) => Some(error),
            Self::RungeKuttaPreparation(error) => Some(error),
            Self::DryTendencyAssembly(error) => Some(error),
            Self::AcousticTrajectory(error) => Some(error),
            Self::ColumnMassStaggering(error) => Some(error),
            Self::AcousticStepFinalization(error) => Some(error),
            Self::Microphysics(error) => Some(error),
            _ => None,
        }
    }
}

impl From<RegistryResolutionError> for ArwModelError {
    fn from(error: RegistryResolutionError) -> Self {
        Self::RegistryResolution(error)
    }
}

impl From<ComputeError> for ArwModelError {
    fn from(error: ComputeError) -> Self {
        Self::Compute(error)
    }
}

impl From<RungeKuttaPreparationError> for ArwModelError {
    fn from(error: RungeKuttaPreparationError) -> Self {
        Self::RungeKuttaPreparation(error)
    }
}

impl From<DryTendencyAssemblyError> for ArwModelError {
    fn from(error: DryTendencyAssemblyError) -> Self {
        Self::DryTendencyAssembly(error)
    }
}

impl From<AcousticTrajectoryError> for ArwModelError {
    fn from(error: AcousticTrajectoryError) -> Self {
        Self::AcousticTrajectory(error)
    }
}

impl From<ColumnMassStaggeringError> for ArwModelError {
    fn from(error: ColumnMassStaggeringError) -> Self {
        Self::ColumnMassStaggering(error)
    }
}

impl From<AcousticStepFinalizationError> for ArwModelError {
    fn from(error: AcousticStepFinalizationError) -> Self {
        Self::AcousticStepFinalization(error)
    }
}

impl From<ArwMicrophysicsError> for ArwModelError {
    fn from(error: ArwMicrophysicsError) -> Self {
        Self::Microphysics(error)
    }
}

/// Result returned by Registry-backed model operations.
pub type ArwModelResult<Value> = Result<Value, ArwModelError>;
