//! MPIABI type definitions, translated from mpiabi/mpiabi.h

use std::os::raw::c_int;

#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("mpi-rt-sys supports only 32-bit and 64-bit pointer targets");

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
    #[cfg(target_pointer_width = "64")]
    _internal: [usize; 3],
    #[cfg(target_pointer_width = "32")]
    _internal: [u32; 5],
    pub MPI_SOURCE: c_int,
    pub MPI_TAG: c_int,
    pub MPI_ERROR: c_int,
}

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(std::mem::size_of::<MPI_Status>() == 40);
    assert!(std::mem::align_of::<MPI_Status>() == 8);
    assert!(std::mem::offset_of!(MPI_Status, MPI_SOURCE) == 24);
    assert!(std::mem::offset_of!(MPI_Status, MPI_TAG) == 28);
    assert!(std::mem::offset_of!(MPI_Status, MPI_ERROR) == 32);
};

#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(std::mem::size_of::<MPI_Status>() == 32);
    assert!(std::mem::align_of::<MPI_Status>() == 4);
    assert!(std::mem::offset_of!(MPI_Status, MPI_SOURCE) == 20);
    assert!(std::mem::offset_of!(MPI_Status, MPI_TAG) == 24);
    assert!(std::mem::offset_of!(MPI_Status, MPI_ERROR) == 28);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_array_stride_matches_abi_size() {
        let status = MPI_Status {
            #[cfg(target_pointer_width = "64")]
            _internal: [0; 3],
            #[cfg(target_pointer_width = "32")]
            _internal: [0; 5],
            MPI_SOURCE: 0,
            MPI_TAG: 0,
            MPI_ERROR: 0,
        };
        let statuses = [status; 2];
        let first = statuses.as_ptr() as usize;
        let second = statuses.as_ptr().wrapping_add(1) as usize;

        assert_eq!(second - first, std::mem::size_of::<MPI_Status>());
        assert_eq!(
            std::mem::size_of::<MPI_Status>(),
            if cfg!(target_pointer_width = "64") {
                40
            } else {
                32
            }
        );
    }
}
