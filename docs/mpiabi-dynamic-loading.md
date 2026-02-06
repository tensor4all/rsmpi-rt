# MPIABI and Dynamic Loading

## What is MPIABI?

[MPIABI](https://github.com/eschnett/MPItrampoline) defines a **standardized Application Binary Interface** for MPI. The core problem it solves: different MPI implementations (OpenMPI, MPICH, Intel MPI, etc.) define MPI types differently at the ABI level, making it impossible to compile once and run with any MPI library.

MPIABI solves this by:
1. Defining all MPI handle types as `uintptr_t` (integer handles, not opaque pointers)
2. Fixing the layout of `MPI_Status`
3. Exporting all MPI constants as named symbols (not compile-time macros)
4. Prefixing ABI-stable symbols with `MPIABI_` (e.g., `MPIABI_COMM_WORLD`)

## Handle Types

In `mpi-rt-sys`, all MPI handle types are defined as `usize`:

```rust
// mpi-rt-sys/src/types.rs
pub type MPI_Comm = usize;
pub type MPI_Datatype = usize;
pub type MPI_Group = usize;
pub type MPI_Op = usize;
pub type MPI_Request = usize;
// ...
```

This matches the MPIABI specification where handles are integer values, in contrast to `mpi-sys` where handle types may be pointers or structs depending on the MPI implementation.

## Dynamic Loading Architecture

```
Application
    |
    v
mpi-rt-sys (Rust)
    |  uses libloading
    v
libmpiwrapper.so (MPIABI wrapper)
    |  links to
    v
libmpi.so (native MPI: OpenMPI, MPICH, etc.)
```

### The Loader (`mpi-rt-sys/src/loader.rs`)

The loader uses `libloading` + `OnceLock` for thread-safe lazy initialization:

```rust
static LIBRARY: OnceLock<Library> = OnceLock::new();

pub fn library() -> &'static Library {
    LIBRARY.get_or_init(|| {
        let path = std::env::var("MPI_RT_LIB").expect("...");
        unsafe { Library::new(&path) }.expect("...")
    })
}
```

The library path comes from the `MPI_RT_LIB` environment variable.

### Function Loading

Each MPI function is loaded lazily on first call using `OnceLock`:

```rust
pub unsafe fn MPI_Send(
    buf: *const c_void,
    count: c_int,
    datatype: MPI_Datatype,
    dest: c_int,
    tag: c_int,
    comm: MPI_Comm,
) -> c_int {
    type F = unsafe extern "C" fn(...) -> c_int;
    static FN: OnceLock<F> = OnceLock::new();
    let f = FN.get_or_init(|| {
        let ptr = loader::get_symbol::<F>(b"MPI_Send\0");
        std::mem::transmute(ptr)
    });
    f(buf, count, datatype, dest, tag, comm)
}
```

### Constant Loading

MPI constants (like `MPI_COMM_WORLD`) cannot be compile-time constants because their values come from the shared library. They are loaded in bulk on first access:

```rust
struct MpiConstants {
    mpi_comm_world: MPI_Comm,
    mpi_comm_null: MPI_Comm,
    // ...
}

static CONSTANTS: OnceLock<MpiConstants> = OnceLock::new();
```

Constants are read from MPIABI-prefixed symbols (e.g., `MPIABI_COMM_WORLD`) and exposed through `RSMPI_*_fn()` accessor functions to match the `mpi-sys` API.

## MPIwrapper

[MPIwrapper](https://github.com/eschnett/MPIwrapper) is a shared library that implements the MPIABI interface by translating calls to a native MPI implementation. It:

1. Is compiled against a specific MPI installation (OpenMPI, MPICH, etc.)
2. Exports all MPI functions with their standard names
3. Exports all MPI constants as `MPIABI_*` symbols
4. Translates between MPIABI integer handles and native MPI handles

### Installation

```bash
git clone https://github.com/eschnett/MPIwrapper
cd MPIwrapper
cmake -S . -B build -DCMAKE_INSTALL_PREFIX=$HOME/.local
cmake --build build
cmake --install build
```

### Usage

```bash
export MPI_RT_LIB=$HOME/.local/lib/libmpiwrapper.so
mpiexec -n 4 ./my_program
```
