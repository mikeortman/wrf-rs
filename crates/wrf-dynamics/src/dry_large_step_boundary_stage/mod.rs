//! The complete dry large-timestep boundary stage of an ARW Runge–Kutta step.
//!
//! WRF v4.7.1 `solve_em.F` runs `relax_bdy_dry` on specified or nested domains
//! when `rk_step == 1`, then `rk_addtend_dry`, then `spec_bdy_dry`. This module
//! preserves that order behind one failure-atomic validation boundary while
//! keeping the three numerical kernels independently reusable.

mod controls;
mod cpu;
mod error;
mod inputs;
mod kernels;
mod mode;
#[cfg(test)]
mod oracle_tests;
mod region;
mod relaxation;
mod relaxation_boundary_values;
mod saved;

pub use controls::DryLargeStepBoundaryStageControls;
pub use error::{DryLargeStepBoundaryStageError, DryLargeStepBoundaryStageResult};
pub use inputs::DryLargeStepBoundaryStageInputs;
pub use kernels::DryLargeStepBoundaryStageKernels;
pub use mode::{DryLargeStepBoundaryStageMode, DryLargeStepNestedVertical};
pub use region::DryLargeStepBoundaryStageRegions;
pub use relaxation::DryLargeStepRelaxationInputs;
pub use relaxation_boundary_values::DryLargeStepRelaxationBoundaryValues;
pub use saved::DryLargeStepSavedTendencies;
