# Architecture Overview

rsmpi-rt provides Rust bindings to MPI with two interchangeable backends selected via Cargo feature flags.

## Crate Structure

```
rsmpi-rt/
  mpi (root crate)         -- High-level Rust MPI API
  mpi-sys/                 -- Bindgen-based FFI (compile-time linking)
  mpi-native-sys/          -- native-handle FFI (runtime loading)
  mpi-derive/              -- Proc-macro for #[derive(Equivalence)]
  build-probe-mpi/         -- Build-time MPI detection
```

## Dependency Diagram

```
                  mpi (src/lib.rs)
                 /                \
    [mpi-sys-backend]      [mpi-native-sys-backend]
          |                        |
       mpi-sys               mpi-native-sys
          |                        |
    bindgen + cc             libloading
          |                        |
    system MPI libs       host MPI (runtime)
```

## Backend Selection

The two backends are **mutually exclusive**, enforced by compile-time checks in `src/lib.rs`:

```rust
#[cfg(all(feature = "mpi-sys-backend", feature = "mpi-native-sys-backend"))]
compile_error!("...");
```

### Feature Flags

| Feature | Backend Crate | Default |
|---------|---------------|---------|
| `mpi-sys-backend` | `mpi-sys` | Yes |
| `mpi-native-sys-backend` | `mpi-native-sys` | No |

## The `ffi` Module Abstraction

Both backends export the same public API surface. The `mpi` crate's `ffi` module re-exports from whichever backend is active:

```rust
pub mod ffi {
    #[cfg(feature = "mpi-sys-backend")]
    pub use mpi_sys::*;
    #[cfg(feature = "mpi-native-sys-backend")]
    pub use mpi_rt_sys::*;
}
```

This means all higher-level code in the `mpi` crate uses `ffi::MPI_*` types and functions without knowing which backend is active.

### API Compatibility Layer

Both backends provide:
- **MPI types**: `MPI_Comm`, `MPI_Datatype`, etc.
- **MPI functions**: `MPI_Init`, `MPI_Send`, etc.
- **Constant accessors**: `RSMPI_COMM_WORLD_fn()`, `RSMPI_DATATYPE_NULL_fn()`, etc.

The `RSMPI_*_fn()` accessor pattern is used because:
- `mpi-sys`: MPI constants are `extern` globals that cannot be used as Rust constants directly. The `_fn()` wrappers in `mpi-sys/src/constant_accessors.rs` provide a uniform function-call interface.
- `mpi-native-sys`: Constants are loaded from the shared library at runtime, so they are naturally functions.

## mpi-sys Backend

Uses `rust-bindgen` to generate FFI bindings at build time from the system MPI headers. Requires:
- A C compiler
- System MPI installation (OpenMPI, MPICH, MS-MPI, etc.)
- `libclang` (for bindgen)

See `mpi-sys/build.rs` and `build-probe-mpi/` for the build-time detection logic.

## mpi-native-sys Backend

Resolves MPI symbols at runtime in the library the host already loaded. Requires:
- No C compiler at build time
- An MPI library of the MPICH ABI, Open MPI ABI or MPI 5.0 standard ABI family, loaded in the process or named by `MPI_NATIVE_LIB`

See [mpiabi-dynamic-loading.md](mpiabi-dynamic-loading.md) for details on the runtime loading mechanism.
