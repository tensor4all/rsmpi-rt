# Backend comparison

| Aspect | `mpi-sys-backend` | `mpi-native-sys-backend` |
|--------|-------------------|--------------------------|
| **Feature flag** | `mpi-sys-backend` (default) | `mpi-native-sys-backend` |
| **Crate** | `mpi-sys` | `mpi-native-sys` |
| **Build-time dependencies** | C compiler, MPI headers, libclang | none |
| **Run-time dependencies** | the MPI library linked at build time | an MPI library of the MPICH ABI, Open MPI ABI or MPI 5.0 standard ABI family |
| **Handles** | the build-time library's | the host library's, found in the process |
| **Surface** | all of MPI | a carried subset; the rest panics with `UnsupportedMpiCall` |
| **MPI implementation switch** | rebuild | none; whatever the host loaded, or `MPI_NATIVE_LIB` |

Use `mpi-native-sys-backend` to share a communicator with MPI.jl or mpi4py without build-time MPI.
Use `mpi-sys-backend` when the whole MPI surface is needed.
