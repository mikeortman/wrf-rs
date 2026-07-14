//! Dry-air-mass and map-factor coupling for C-grid momentum components.

mod axis;
mod coefficient;
mod coefficients;
mod cpu;
mod error;
mod field;
mod fields;
mod kernels;
mod region;

pub use axis::MomentumCouplingAxis;
pub use coefficient::MomentumCouplingCoefficient;
pub use coefficients::MomentumCouplingCoefficients;
pub use error::{MomentumCouplingError, MomentumCouplingResult};
pub use field::MomentumCouplingField;
pub use fields::{
    MomentumCouplingMapFactors, MomentumCouplingMasses, MomentumCouplingOutputs,
    MomentumCouplingVelocities,
};
pub use kernels::MomentumCouplingKernels;
pub use region::MomentumCouplingRegion;

pub(crate) use cpu::validate_borrowed_operation;
