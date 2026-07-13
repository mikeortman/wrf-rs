//! Positive-definite scalar-field correction.

mod cpu;
mod error;
mod kernels;
mod slab_axis;
mod slab_region;

pub use error::{PositiveDefiniteError, PositiveDefiniteResult};
pub use kernels::PositiveDefiniteKernels;
pub use slab_axis::PositiveDefiniteSlabAxis;
pub use slab_region::PositiveDefiniteSlabRegion;
