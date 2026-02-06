//! MPIABI type definitions, translated from mpiabi/mpiabi.h

use std::os::raw::c_int;

// Basic types
pub type MPI_Aint = isize; // intptr_t
pub type MPI_Count = i64; // int64_t
pub type MPI_Fint = c_int; // int
pub type MPI_Offset = i64; // int64_t

// All handles are integer types (MPItrampoline ABI design)
pub type MPI_Comm = usize;
pub type MPI_Datatype = usize;
pub type MPI_Errhandler = usize;
pub type MPI_File = usize;
pub type MPI_Group = usize;
pub type MPI_Info = usize;
pub type MPI_Message = usize;
pub type MPI_Op = usize;
pub type MPI_Request = usize;
pub type MPI_Win = usize;

/// MPI_Status structure compatible with MPIABI spec.
/// The internal union accommodates both OpenMPI and MPICH layouts.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MPI_Status {
    // Internal fields (union of OpenMPI/MPICH layouts)
    _internal: [u8; MPIABI_STATUS_INTERNAL_SIZE],
    pub MPI_SOURCE: c_int,
    pub MPI_TAG: c_int,
    pub MPI_ERROR: c_int,
}

// Size of the internal union (max of OpenMPI: 4*int + size_t, MPICH: 5*int)
// On 64-bit: OpenMPI = 4*4 + 8 = 24, MPICH = 5*4 = 20 => 24
// On 32-bit: OpenMPI = 4*4 + 4 = 20, MPICH = 5*4 = 20 => 20
#[cfg(target_pointer_width = "64")]
const MPIABI_STATUS_INTERNAL_SIZE: usize = 24;
#[cfg(target_pointer_width = "32")]
const MPIABI_STATUS_INTERNAL_SIZE: usize = 20;

const _: () = assert!(
    std::mem::size_of::<MPI_Status>()
        == MPIABI_STATUS_INTERNAL_SIZE + 3 * std::mem::size_of::<c_int>()
);
