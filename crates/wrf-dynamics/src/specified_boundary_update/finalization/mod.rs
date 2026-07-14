//! Boundary-file state reconstruction after WRF acoustic integration.
//!
//! The capability shares specified-zone geometry while making WRF's field
//! location, column-mass normalization, and momentum map-factor coupling
//! explicit. Boundary arrays retain backend-native storage for future GPU use.

mod boundary_fields;
mod cpu;
mod error;
mod field_location;
mod inputs;
mod kernels;
mod parameters;
mod region;

pub use boundary_fields::SpecifiedBoundaryFinalizationBoundaryFields;
pub use error::{SpecifiedBoundaryFinalizationError, SpecifiedBoundaryFinalizationResult};
pub use field_location::SpecifiedBoundaryFinalizationFieldLocation;
pub use inputs::SpecifiedBoundaryFinalizationInputs;
pub use kernels::SpecifiedBoundaryFinalizationKernels;
pub use parameters::SpecifiedBoundaryFinalizationParameters;
pub use region::SpecifiedBoundaryFinalizationRegion;
