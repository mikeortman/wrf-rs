//! Specified-zone tendency updates for ARW prognostic fields.
//!
//! This module translates WRF v4.7.1 `spec_bdyupdate`. Typed field locations
//! replace the source character selector, and the validated region owns the
//! resulting staggered execution ranges.

mod cpu;
mod error;
mod field_location;
mod kernels;
mod parameters;
mod periodicity;
mod region;

pub use error::{SpecifiedBoundaryUpdateError, SpecifiedBoundaryUpdateResult};
pub use field_location::SpecifiedBoundaryFieldLocation;
pub use kernels::SpecifiedBoundaryUpdateKernels;
pub use parameters::SpecifiedBoundaryUpdateParameters;
pub use periodicity::SpecifiedBoundaryWestEastPeriodicity;
pub use region::{SpecifiedBoundaryUpdateAxis, SpecifiedBoundaryUpdateRegion};
