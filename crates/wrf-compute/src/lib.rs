//! Field storage and execution boundaries for WRF numerical kernels.
//!
//! [`CpuBackend`] is the reference implementation and uses all available CPU
//! parallelism by default. Numerical crates should define focused kernel
//! capability traits that consume this storage boundary so a future GPU backend
//! can provide native kernels without emulating CPU closures.

#![forbid(unsafe_code)]

mod backend_kind;
mod compute_backend;
mod compute_error;
mod cpu_backend;
mod cpu_field;
mod field_storage;
mod field_value;
mod grid_shape;
mod linear_chunk;

pub use backend_kind::BackendKind;
pub use compute_backend::ComputeBackend;
pub use compute_error::{ComputeError, ComputeResult, ParallelExecutionError};
pub use cpu_backend::CpuBackend;
pub use cpu_field::CpuField;
pub use field_storage::FieldStorage;
pub use field_value::FieldValue;
pub use grid_shape::GridShape;
pub use linear_chunk::LinearChunk;
