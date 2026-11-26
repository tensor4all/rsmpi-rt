//! Family-neutral MPI types.
//!
//! Every handle is a `usize` above the sys layer (D5). The conversion to the family's real
//! `int` or pointer happens in one place, [`crate::functions`], through separate
//! function-pointer types per family.

use std::os::raw::c_int;

#[cfg(not(target_pointer_width = "64"))]
compile_error!("mpi-native-sys supports 64-bit targets only (handles are carried in a usize)");

/// `MPI_Aint`.
pub type MPI_Aint = isize;
/// `MPI_Count`.
pub type MPI_Count = i64;
/// `MPI_Fint`.
pub type MPI_Fint = c_int;
/// `MPI_Offset`.
pub type MPI_Offset = i64;

/// Neutral communicator handle: a zero-extended `int` (MPICH ABI) or an address.
pub type MPI_Comm = usize;
/// Neutral datatype handle.
pub type MPI_Datatype = usize;
/// Neutral error-handler handle.
pub type MPI_Errhandler = usize;
/// Neutral file handle.
pub type MPI_File = usize;
/// Neutral group handle.
pub type MPI_Group = usize;
/// Neutral info handle.
pub type MPI_Info = usize;
/// Neutral matched-message handle.
pub type MPI_Message = usize;
/// Neutral reduction-operation handle.
pub type MPI_Op = usize;
/// Neutral request handle.
pub type MPI_Request = usize;
/// Neutral window handle.
pub type MPI_Win = usize;

/// Bytes of the opaque status buffer handed to C. It is larger than every family's
/// `MPI_Status` (20, 24 and 32 bytes) and than MPIABI's 40.
pub const STATUS_BYTES: usize = 64;

/// A family-neutral `MPI_Status`.
///
/// `raw` is the buffer the library writes; its layout is the family's and is never read
/// here except through `MPI_Get_count`. The three public fields are copied out of `raw` at
/// the family's offsets after `MPI_Mprobe`/`MPI_Mrecv`, so rsmpi-rt's field reads work.
#[repr(C, align(8))]
#[derive(Copy, Clone)]
pub struct MPI_Status {
    raw: [u8; STATUS_BYTES],
    /// Source rank, copied from the family's layout.
    pub MPI_SOURCE: c_int,
    /// Tag, copied from the family's layout.
    pub MPI_TAG: c_int,
    /// Error code, copied from the family's layout.
    pub MPI_ERROR: c_int,
}

impl MPI_Status {
    pub(crate) fn raw_ptr(&mut self) -> *mut std::ffi::c_void {
        self.raw.as_mut_ptr().cast()
    }
    pub(crate) fn raw_const_ptr(&self) -> *const std::ffi::c_void {
        self.raw.as_ptr().cast()
    }
    pub(crate) fn read_i32(&self, offset: usize) -> c_int {
        let mut b = [0u8; 4];
        b.copy_from_slice(&self.raw[offset..offset + 4]);
        c_int::from_ne_bytes(b)
    }
}

// The layout table as const assertions (docs/design/mpi-native-abi.md §4).
const _: () = {
    // MPICH ABI: count_lo, count_hi_and_cancelled, SOURCE@8, TAG@12, ERROR@16 (20 bytes).
    assert!(16 + 4 <= STATUS_BYTES);
    // Open MPI ABI: SOURCE@0, TAG@4, ERROR@8, cancelled@12, ucount@16 (24 bytes).
    assert!(16 + 8 <= STATUS_BYTES);
    // Standard ABI: SOURCE@0, TAG@4, ERROR@8, internal[5] (32 bytes).
    assert!(12 + 20 <= STATUS_BYTES);
    assert!(std::mem::align_of::<MPI_Status>() == 8);
};
