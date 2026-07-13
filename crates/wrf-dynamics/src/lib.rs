//! Numerical kernels from WRF's Advanced Research WRF dynamical core.
//!
//! Each kernel family exposes a focused backend capability so CPU and future
//! GPU implementations can use native storage and execution strategies.
//!
//! The crate preserves WRF's observable numerical behavior, not its Fortran
//! implementation structure. Safe in-place mutation, persistent parallelism,
//! and typed shape checks replace temporary arrays and implicit contracts when
//! those changes retain parity.

#![forbid(unsafe_code)]

mod held_suarez;
mod positive_definite;

pub use held_suarez::{
    HeldSuarezDampingAxis, HeldSuarezDampingError, HeldSuarezDampingField, HeldSuarezDampingFields,
    HeldSuarezDampingKernels, HeldSuarezDampingRegion, HeldSuarezDampingResult,
};
pub use positive_definite::{
    PositiveDefiniteError, PositiveDefiniteKernels, PositiveDefiniteResult,
    PositiveDefiniteSlabAxis, PositiveDefiniteSlabRegion,
};
