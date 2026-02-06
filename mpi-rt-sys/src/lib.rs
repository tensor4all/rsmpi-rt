//! Low-level Rust bindings to MPI via MPIABI dynamic loading.
//!
//! This crate provides MPI FFI bindings that load an MPI implementation at runtime
//! using the [MPIABI](https://github.com/eschnett/MPItrampoline) standard, rather
//! than linking at compile time.
//!
//! # Usage
//!
//! This crate is typically not used directly. Instead, use the `mpi` crate with
//! the `mpi-rt-sys-backend` feature:
//!
//! ```toml
//! [dependencies]
//! mpi = { version = "0.1.0", default-features = false, features = ["mpi-rt-sys-backend"] }
//! ```
//!
//! At runtime, set the `MPI_RT_LIB` environment variable to the path of an
//! MPIABI-compatible wrapper library (e.g., [MPIwrapper](https://github.com/eschnett/MPIwrapper)):
//!
//! ```bash
//! export MPI_RT_LIB=/path/to/libmpiwrapper.so
//! ```
//!
//! # Architecture
//!
//! - **`loader`**: Loads the shared library via `libloading`
//! - **`functions`**: MPI function wrappers with lazy symbol resolution
//! - **`constants`**: MPI constant accessors loaded from MPIABI symbols
//! - **`types`**: MPI type definitions (all handles are `usize` per MPIABI)
//! - **`callback_types`**: MPI callback function pointer type aliases

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(
    clippy::missing_safety_doc,
    clippy::crosspointer_transmute,
    clippy::too_many_arguments
)]

pub mod callback_types;
pub mod constants;
pub mod functions;
pub mod loader;
pub mod types;

pub use callback_types::*;
pub use constants::*;
pub use functions::*;
pub use types::*;
