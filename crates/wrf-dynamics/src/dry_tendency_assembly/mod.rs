//! Assembly of dry large-timestep tendencies for an ARW Runge–Kutta substep.
//!
//! This module translates WRF v4.7.1 `rk_addtend_dry`. It combines persistent
//! forward/physics tendencies with substep dynamics tendencies while preserving
//! WRF's C-grid staggering, map-factor placement, and single-precision order.

mod axis;
mod coefficient;
mod coefficients;
mod cpu;
mod error;
mod field;
mod fields;
mod kernels;
#[cfg(test)]
mod oracle_tests;
mod phase;
mod region;

pub use axis::DryTendencyAssemblyAxis;
pub use coefficient::DryTendencyAssemblyCoefficient;
pub use coefficients::DryTendencyAssemblyCoefficients;
pub use error::{DryTendencyAssemblyError, DryTendencyAssemblyResult};
pub use field::DryTendencyAssemblyField;
pub use fields::{
    DryTendencyAssemblyForwardTendencies, DryTendencyAssemblyMapFactors,
    DryTendencyAssemblyRungeKuttaTendencies, DryTendencyAssemblySavedTendencies,
    DryTendencyAssemblyThermodynamics,
};
pub use kernels::DryTendencyAssemblyKernels;
pub use phase::DryTendencyAssemblyPhase;
pub use region::DryTendencyAssemblyRegion;
