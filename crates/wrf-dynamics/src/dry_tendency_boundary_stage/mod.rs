//! Coupled dry-tendency assembly and specified-boundary assignment.
//!
//! WRF v4.7.1 calls `rk_addtend_dry` immediately followed by `spec_bdy_dry`.
//! This module preserves that order behind one failure-atomic validation
//! boundary while keeping the two numerical kernels independently reusable.

mod controls;
mod cpu;
mod error;
mod inputs;
mod kernels;
#[cfg(test)]
mod oracle_tests;
mod region;
mod vertical;

pub use controls::DryTendencyBoundaryStageControls;
pub use error::{DryTendencyBoundaryStageError, DryTendencyBoundaryStageResult};
pub use inputs::DryTendencyBoundaryStageInputs;
pub use kernels::DryTendencyBoundaryStageKernels;
pub use region::DryTendencyBoundaryStageRegions;
pub use vertical::DryTendencyBoundaryStageVertical;
