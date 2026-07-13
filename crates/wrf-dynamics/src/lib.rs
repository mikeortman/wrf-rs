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

mod cpu_positive_definite_kernels;
mod positive_definite_error;
mod positive_definite_kernels;
mod positive_definite_slab_axis;
mod positive_definite_slab_region;

pub use positive_definite_error::{PositiveDefiniteError, PositiveDefiniteResult};
pub use positive_definite_kernels::PositiveDefiniteKernels;
pub use positive_definite_slab_axis::PositiveDefiniteSlabAxis;
pub use positive_definite_slab_region::PositiveDefiniteSlabRegion;
