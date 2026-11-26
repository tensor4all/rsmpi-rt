# rsmpi-rt

[![GitHub Actions][actions-shield]][actions]
[![Documentation][doc-shield]][doc]
[![License: Apache License 2.0 or MIT][license-shield]][license]

**Rust [MPI] bindings with runtime library loading.** Fork of [rsmpi] v0.8.1.

> **Status: Experimental / Proof of Concept.**
> This project aims to eventually merge its runtime-loading backend upstream into rsmpi.

The key addition over upstream rsmpi is the `mpi-native-sys-backend`: a backend that finds the MPI library already loaded in the process (or loads one by name) and calls it with **that library's own handles**, requiring **no C compiler, system MPI headers, libclang or wrapper library at build or run time**. The MPICH ABI, the Open MPI ABI and the MPI 5.0 standard ABI are supported.

### Motivation

The primary goal is to enable **calling Rust MPI code from Julia and Python** without build-time MPI dependencies. By loading MPI at runtime, Rust libraries can share the same MPI communicator with [MPI.jl] and [mpi4py], enabling seamless multi-language HPC workflows.

[actions-shield]: https://github.com/tensor4all/rsmpi-rt/workflows/Test/badge.svg
[actions]: https://github.com/tensor4all/rsmpi-rt/actions
[doc-shield]: https://img.shields.io/badge/docs-GitHub%20Pages-blue
[doc]: https://tensor4all.github.io/rsmpi-rt/mpi/
[license-shield]: https://img.shields.io/badge/license-Apache_License_2.0_or_MIT-blue.svg?style=flat-square
[license]: https://github.com/tensor4all/rsmpi-rt#license
[MPI]: http://www.mpi-forum.org
[rsmpi]: https://github.com/rsmpi/rsmpi
[MPI.jl]: https://github.com/JuliaParallel/MPI.jl
[mpi4py]: https://mpi4py.readthedocs.io/

## Quick Start

```toml
[dependencies]
mpi = { package = "rsmpi-rt", version = "0.9.0", default-features = false, features = ["mpi-native-sys-backend"] }
```

```rust
use mpi::traits::*;

fn main() {
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    println!("Hello from rank {} of {}", world.rank(), world.size());
}
```

```bash
mpiexec -n 4 cargo run   # the site libmpi is found by soname; MPI_NATIVE_LIB overrides
```

## Backend Selection

| Feature | Description | Default |
|---------|-------------|---------|
| `mpi-sys-backend` | bindgen against system MPI headers (needs libclang) | Yes |
| `mpi-native-sys-backend` | native handles of the MPI library found at run time (no build-time MPI deps) | No |

Exactly one backend must be enabled.

### mpi-native-sys-backend

```toml
mpi = { package = "rsmpi-rt", version = "0.9.0", default-features = false, features = ["mpi-native-sys-backend"] }
```

**Run time.** When a host (MPI.jl, mpi4py) has already loaded MPI, the library is found by walking the
loaded objects and nothing is loaded a second time. A standalone Rust program calls `mpi::initialize()`;
the backend then loads `MPI_NATIVE_LIB` if set, else `libmpi.so.12`, `libmpi.so.40`,
`libmpi_cray.so.12` or `libmpi_abi.so.1`. The family is identified before any handle is used
(MPIwrapper marker, then the return value of `MPI_Abi_get_version`, then the version string and a
symbol witness). An MPI outside the supported families fails with a named error instead of
passing foreign handles. There is no wrapper fallback: MPItrampoline support was removed in 0.9.0
(`mpi-rt-sys 0.1.0` stays on crates.io).

Only a carried subset of MPI is real (initialization, communicator queries, blocking send,
matched probe/receive, error strings). Every other name in `ffi` is a stub that panics with an
`UnsupportedMpiCall` payload and increments `mpi_native_sys::stub_hits()`.

## Optional Features

| Feature | Description |
|---------|-------------|
| `user-operations` | User-defined reduction operations via `libffi` |
| `derive` | `#[derive(Equivalence)]` for sending structs over MPI |
| `complex` | Support for `num-complex` types |

## Interoperability Tests

Integration tests verify that Rust MPI code can run alongside [mpi4py] and [MPI.jl] under the same `mpiexec`, sharing `MPI_COMM_WORLD` via MPMD launch:

```bash
# Run the cross-language interop test (requires Python/mpi4py and Julia/MPI.jl)
bash tests/interop/run_interop.sh
```

See [tests/interop/](tests/interop/) for details.

## Documentation

- [API docs (GitHub Pages)][doc]
- [Architecture overview](docs/architecture.md)
- [Backend comparison](docs/backend-comparison.md)
- [Code generation](docs/codegen.md)

Generate docs locally:

```bash
cargo doc --workspace --no-deps --features mpi-sys-backend,user-operations,derive,complex
```

## Examples

See [examples/](https://github.com/tensor4all/rsmpi-rt/tree/main/examples).

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
