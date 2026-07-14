//! Tridiagonal coefficients for WRF's vertically implicit acoustic solve.
//!
//! This module translates WRF v4.7.1 `calc_coef_w`. Its public capability
//! separates backend-native fields from one-dimensional coordinate data so a
//! future GPU backend can construct the same coefficients without exposing CPU
//! storage or host closures.

mod axis;
mod coefficient;
mod cpu;
mod error;
mod field;
mod inputs;
mod kernels;
mod mass_coefficients;
mod metrics;
mod outputs;
mod parameters;
mod region;
mod top_boundary;

pub use axis::VerticalAcousticCoefficientAxis;
pub use coefficient::VerticalAcousticCoefficient;
pub(crate) use cpu::validate_vertical_acoustic_coefficients;
pub use error::{VerticalAcousticCoefficientError, VerticalAcousticCoefficientResult};
pub use field::VerticalAcousticCoefficientField;
pub use inputs::VerticalAcousticCoefficientInputs;
pub use kernels::VerticalAcousticCoefficientKernels;
pub use mass_coefficients::VerticalAcousticMassCoefficients;
pub use metrics::VerticalAcousticMetrics;
pub use outputs::VerticalAcousticSolveCoefficients;
pub use parameters::VerticalAcousticCoefficientParameters;
pub use region::VerticalAcousticCoefficientRegion;
pub use top_boundary::VerticalAcousticTopBoundary;
