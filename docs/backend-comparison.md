# Backend Comparison

## Summary Table

| Aspect | `mpi-sys-backend` | `mpi-rt-sys-backend` |
|--------|-------------------|----------------------|
| **Feature flag** | `mpi-sys-backend` (default) | `mpi-rt-sys-backend` |
| **Crate** | `mpi-sys` | `mpi-rt-sys` |
| **Binding generation** | `rust-bindgen` at build time | `gen_rust.py` (pre-generated) |
| **Build dependencies** | C compiler, `libclang`, system MPI headers | None (pure Rust) |
| **Runtime dependencies** | System MPI library (linked at compile time) | MPIwrapper shared library |
| **Symbol resolution** | Link-time (static or dynamic linking) | Runtime (`libloading` / `dlopen`) |
| **MPI handle types** | Implementation-defined (pointers, ints, structs) | Always `usize` (MPIABI standard) |
| **MPI constants** | `extern` globals from C header | Loaded from shared library symbols |
| **Configuration** | `MPI_PKG_CONFIG`, `MPICC`, or platform-specific env vars | `MPI_RT_LIB` env var at runtime |
| **Cross-compilation** | Requires target MPI headers | Binary is MPI-agnostic |
| **MPI implementation switch** | Requires rebuild | Change `MPI_RT_LIB` at runtime |
| **Platform support** | Linux, macOS, Windows | Linux, macOS (where `dlopen` works) |
| **Maturity** | Production (upstream rsmpi) | Experimental |

## When to Use Each Backend

### Use `mpi-sys-backend` when:
- You have a stable MPI installation on your build system
- You need maximum compatibility with existing rsmpi code
- You're on Windows with MS-MPI or Intel MPI
- You want the most battle-tested path

### Use `mpi-rt-sys-backend` when:
- You want to build without MPI installed (e.g., CI/CD, containers)
- You need to switch MPI implementations without recompiling
- You're distributing pre-built binaries that should work with any MPI
- You're cross-compiling and don't have target MPI headers
- You want to avoid the `libclang` build dependency

## API Compatibility

Both backends expose the same public API to the `mpi` crate. The `ffi` module in `src/lib.rs` conditionally re-exports from the active backend. User code that depends on the `mpi` crate does not need to change when switching backends.

The key abstraction that enables this is the `RSMPI_*_fn()` accessor pattern for constants. Both backends provide these functions, but the implementation differs:
- `mpi-sys`: wraps `extern` globals in trivial accessor functions
- `mpi-rt-sys`: loads values from the shared library on first call
