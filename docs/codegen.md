# Code Generation (`gen_rust.py`)

The `mpi-rt-sys` crate uses a Python code generator to produce Rust FFI bindings from MPIABI definitions.

## Overview

```
mpi-rt-sys/gen/
  gen_rust.py              -- Main generator script
  mpiabi/
    mpi_functions.py       -- Function signatures (from MPIABI spec)
    mpi_constants.py       -- Constant definitions (from MPIABI spec)
```

## Input

### `mpiabi/mpi_functions.py`

Defines all MPI functions as a list of tuples:

```python
functions = [
    ("int", "MPI_Send", [("const void *", "buf"), ("int", "count"), ...], "tag"),
    ...
]
```

Each entry: `(return_type, function_name, [(param_type, param_name), ...], tag)`

### `mpiabi/mpi_constants.py`

Defines all MPI constants:

```python
constants = [
    ("MPI_Comm", "MPI_COMM_WORLD"),
    ("MPI_Comm", "MPI_COMM_NULL"),
    ...
]
```

Each entry: `(c_type, constant_name)`

## Output

The generator produces three files in `mpi-rt-sys/src/`:

### `functions.rs`

Dynamic-dispatch wrappers for each MPI function. Each function:
1. Defines the C function pointer type
2. Uses `OnceLock` to lazily load the symbol from the shared library
3. Calls through the function pointer

### `constants.rs`

Constant loading and accessor functions:
- A `MpiConstants` struct holding all constant values
- `get_constants()` that loads all constants from the library on first call
- `RSMPI_*_fn()` accessor functions matching the `mpi-sys` API
- `RSMPI_*()` convenience aliases

### `callback_types.rs`

MPI callback function pointer type aliases (e.g., `MPI_User_function`, `MPI_Comm_copy_attr_function`). All wrapped in `Option<...>` to match bindgen's convention for nullable C function pointers.

## Type Mapping

| C Type | Rust Type |
|--------|-----------|
| `int` | `c_int` |
| `double` | `c_double` |
| `void *` | `*mut c_void` |
| `const void *` | `*const c_void` |
| `char *` | `*mut c_char` |
| `const char *` | `*const c_char` |
| `MPI_Comm` | `MPI_Comm` (= `usize`) |
| `MPI_Datatype` | `MPI_Datatype` (= `usize`) |
| `MPI_Aint` | `MPI_Aint` (= `isize`) |
| `MPI_Count` | `MPI_Count` (= `isize`) |

## Regenerating

To regenerate the Rust bindings after modifying the MPIABI definitions:

```bash
cd mpi-rt-sys/gen
python3 gen_rust.py
```

This overwrites `../src/functions.rs`, `../src/constants.rs`, and `../src/callback_types.rs`. Do not edit those files manually.
