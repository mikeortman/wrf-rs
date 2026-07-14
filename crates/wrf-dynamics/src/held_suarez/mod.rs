//! Held-Suarez idealized momentum damping.

mod axis;
mod cpu;
mod error;
mod field;
mod fields;
mod kernels;
mod line_layout;
mod region;
mod simd;

pub use axis::HeldSuarezDampingAxis;
pub use error::{HeldSuarezDampingError, HeldSuarezDampingResult};
pub use field::HeldSuarezDampingField;
pub use fields::HeldSuarezDampingFields;
pub use kernels::HeldSuarezDampingKernels;
pub use region::HeldSuarezDampingRegion;
