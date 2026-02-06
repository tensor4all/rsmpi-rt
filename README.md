# rsmpi-rt

[![GitHub Actions][actions-shield]][actions]
[![Documentation][doc-shield]][doc]
[![License: Apache License 2.0 or MIT][license-shield]][license]

**Rust [MPI] bindings with runtime library loading.** Fork of [rsmpi] v0.8.1.

> **Status: Experimental / Proof of Concept.**
> This project aims to eventually merge its runtime-loading backend upstream into rsmpi.

The key addition over upstream rsmpi is the `mpi-rt-sys-backend`: an [MPIABI]-based backend that loads MPI at runtime via `dlopen`, requiring **no C compiler, system MPI headers, or libclang at build time**.

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
[MPIABI]: https://github.com/eschnett/MPItrampoline
[MPI.jl]: https://github.com/JuliaParallel/MPI.jl
[mpi4py]: https://mpi4py.readthedocs.io/

## Quick Start

```toml
[dependencies]
mpi = { git = "https://github.com/tensor4all/rsmpi-rt", default-features = false, features = ["mpi-rt-sys-backend"] }
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
export MPI_RT_LIB=/path/to/libmpiwrapper.so
mpiexec -n 4 cargo run
```

## Backend Selection

| Feature | Description | Default |
|---------|-------------|---------|
| `mpi-sys-backend` | Bindgen-based (requires C compiler, system MPI, libclang) | Yes |
| `mpi-rt-sys-backend` | MPIABI-based runtime loading (no build-time MPI deps) | No |

The two backends are **mutually exclusive**.

### mpi-rt-sys-backend (recommended for this fork)

No build-time MPI dependencies. The MPI library is loaded at runtime via [MPIwrapper].

```toml
[dependencies]
mpi = { git = "https://github.com/tensor4all/rsmpi-rt", default-features = false, features = ["mpi-rt-sys-backend"] }
```

With optional features:

```toml
[dependencies]
mpi = { git = "https://github.com/tensor4all/rsmpi-rt", default-features = false, features = ["mpi-rt-sys-backend", "user-operations", "derive"] }
```

**Runtime setup:** Set `MPI_RT_LIB` to the path of an [MPIwrapper] library:

```bash
export MPI_RT_LIB=/path/to/libmpiwrapper.so
mpiexec -n 4 ./my_program
```

To build MPIwrapper (requires a system MPI):

```bash
git clone https://github.com/eschnett/MPIwrapper
cd MPIwrapper
cmake -S . -B build -DCMAKE_INSTALL_PREFIX=$HOME/.local
cmake --build build
cmake --install build
export MPI_RT_LIB=$HOME/.local/lib/libmpiwrapper.so
```

[MPIwrapper]: https://github.com/eschnett/MPIwrapper

### mpi-sys-backend

Traditional compile-time binding via `rust-bindgen`. Same as upstream [rsmpi]. Requires a C compiler, system MPI installation, and `libclang`.

```toml
[dependencies]
mpi = { git = "https://github.com/tensor4all/rsmpi-rt" }
```

See the [upstream rsmpi README](https://github.com/rsmpi/rsmpi#requirements) for detailed build requirements.

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
- [MPIABI and dynamic loading](docs/mpiabi-dynamic-loading.md)
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
