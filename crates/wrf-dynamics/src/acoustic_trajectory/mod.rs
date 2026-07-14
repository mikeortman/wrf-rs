//! Complete local acoustic small-step trajectory.
//!
//! This module composes the translated WRF v4.7.1 kernels in `solve_em.F`
//! order. Distributed communication, polar filtering, and specified/nested
//! boundary forcing remain orchestration boundaries outside this capability.

mod coefficients;
mod controls;
mod cpu;
mod diagnostics;
mod error;
mod inputs;
mod kernels;
mod regions;
mod saved_state;
mod time_levels;
mod workspace;

pub use coefficients::AcousticTrajectoryCoefficients;
pub use controls::AcousticTrajectoryControls;
pub(crate) use cpu::{AcousticTrajectoryExecution, validate_acoustic_trajectory};
pub use diagnostics::AcousticTrajectoryDiagnostics;
pub use error::{AcousticTrajectoryError, AcousticTrajectoryResult};
pub use inputs::{
    AcousticTrajectoryInputs, AcousticTrajectoryMapFactors, AcousticTrajectoryMassInputs,
    AcousticTrajectoryMoistureCoefficients, AcousticTrajectoryPressureInputs,
    AcousticTrajectoryTendencies,
};
pub use kernels::AcousticTrajectoryKernels;
pub use regions::AcousticTrajectoryRegions;
pub use saved_state::AcousticTrajectorySavedState;
pub use time_levels::AcousticTrajectoryTimeLevels;
pub use workspace::AcousticTrajectoryWorkspace;
