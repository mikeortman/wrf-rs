//! Typed WRF NetCDF schema, I/O, and restart-equivalence support.
//!
//! The first writer targets WRF's supported NetCDF-3 64-bit-offset mode. The
//! reader uses the thread-safe GeoRust wrapper over NetCDF-C and therefore also
//! accepts NetCDF-4 files. All local code is safe Rust.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod dataset;
mod error;
mod netcdf;
mod restart;
mod schema;

pub use dataset::{WrfDatasetView, WrfVariableValues, WrfVariableView};
pub use error::{WrfIoError, WrfIoResult};
pub use netcdf::{WrfNetcdfReader, WrfNetcdfWriter};
pub use restart::WrfRestartComparer;
pub use schema::{
    WrfAttribute, WrfAttributeValue, WrfDataType, WrfDimension, WrfDimensionName, WrfFileKind,
    WrfFileSchema, WrfGridDimensions, WrfTimestamp, WrfVariableName, WrfVariableSchema,
};
