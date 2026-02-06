//! Backend abstraction layer.
//!
//! Re-exports types and functions from either mpi-sys or mpitrampoline-sys
//! depending on the selected feature flag.

#[cfg(feature = "mpi-sys-backend")]
pub use mpi_sys::*;
#[cfg(feature = "mpi-sys-backend")]
pub use mpi_sys::constant_accessors::*;

#[cfg(feature = "mpitrampoline")]
pub use mpitrampoline_sys::*;
