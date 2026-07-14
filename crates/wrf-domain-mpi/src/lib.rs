//! MPI transport for storage-neutral [`wrf_domain::HaloExchangePlan`] values.
//!
//! This adapter is deliberately separate from `wrf-domain`: scientific APIs
//! see topology and transfer descriptors, while only this crate depends on MPI.

#![forbid(unsafe_code)]

mod mpi_halo_exchange;
mod mpi_halo_exchange_error;

pub use mpi_halo_exchange::MpiHaloExchange;
pub use mpi_halo_exchange_error::{MpiHaloExchangeError, MpiHaloExchangeResult};
