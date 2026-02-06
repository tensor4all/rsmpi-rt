//! Accessor functions for MPI constants.
//!
//! These wrap the extern const RSMPI_* values as functions,
//! providing a uniform API with mpitrampoline-sys.

use super::*;

// Datatypes
pub fn RSMPI_C_BOOL_fn() -> MPI_Datatype {
    unsafe { RSMPI_C_BOOL }
}
pub fn RSMPI_FLOAT_fn() -> MPI_Datatype {
    unsafe { RSMPI_FLOAT }
}
pub fn RSMPI_DOUBLE_fn() -> MPI_Datatype {
    unsafe { RSMPI_DOUBLE }
}
pub fn RSMPI_INT8_T_fn() -> MPI_Datatype {
    unsafe { RSMPI_INT8_T }
}
pub fn RSMPI_INT16_T_fn() -> MPI_Datatype {
    unsafe { RSMPI_INT16_T }
}
pub fn RSMPI_INT32_T_fn() -> MPI_Datatype {
    unsafe { RSMPI_INT32_T }
}
pub fn RSMPI_INT64_T_fn() -> MPI_Datatype {
    unsafe { RSMPI_INT64_T }
}
pub fn RSMPI_UINT8_T_fn() -> MPI_Datatype {
    unsafe { RSMPI_UINT8_T }
}
pub fn RSMPI_UINT16_T_fn() -> MPI_Datatype {
    unsafe { RSMPI_UINT16_T }
}
pub fn RSMPI_UINT32_T_fn() -> MPI_Datatype {
    unsafe { RSMPI_UINT32_T }
}
pub fn RSMPI_UINT64_T_fn() -> MPI_Datatype {
    unsafe { RSMPI_UINT64_T }
}
pub fn RSMPI_FLOAT_COMPLEX_fn() -> MPI_Datatype {
    unsafe { RSMPI_FLOAT_COMPLEX }
}
pub fn RSMPI_DOUBLE_COMPLEX_fn() -> MPI_Datatype {
    unsafe { RSMPI_DOUBLE_COMPLEX }
}
pub fn RSMPI_DATATYPE_NULL_fn() -> MPI_Datatype {
    unsafe { RSMPI_DATATYPE_NULL }
}

// Communicators
pub fn RSMPI_COMM_WORLD_fn() -> MPI_Comm {
    unsafe { RSMPI_COMM_WORLD }
}
pub fn RSMPI_COMM_NULL_fn() -> MPI_Comm {
    unsafe { RSMPI_COMM_NULL }
}
pub fn RSMPI_COMM_SELF_fn() -> MPI_Comm {
    unsafe { RSMPI_COMM_SELF }
}

// Comm type
pub fn RSMPI_COMM_TYPE_SHARED_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_COMM_TYPE_SHARED }
}

// Groups
pub fn RSMPI_GROUP_EMPTY_fn() -> MPI_Group {
    unsafe { RSMPI_GROUP_EMPTY }
}
pub fn RSMPI_GROUP_NULL_fn() -> MPI_Group {
    unsafe { RSMPI_GROUP_NULL }
}

// Special values
pub fn RSMPI_UNDEFINED_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_UNDEFINED }
}
pub fn RSMPI_PROC_NULL_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_PROC_NULL }
}
pub fn RSMPI_ANY_SOURCE_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_ANY_SOURCE }
}
pub fn RSMPI_ANY_TAG_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_ANY_TAG }
}

// Messages
pub fn RSMPI_MESSAGE_NULL_fn() -> MPI_Message {
    unsafe { RSMPI_MESSAGE_NULL }
}
pub fn RSMPI_MESSAGE_NO_PROC_fn() -> MPI_Message {
    unsafe { RSMPI_MESSAGE_NO_PROC }
}

// Requests
pub fn RSMPI_REQUEST_NULL_fn() -> MPI_Request {
    unsafe { RSMPI_REQUEST_NULL }
}

// Status
pub fn RSMPI_STATUS_IGNORE_fn() -> *mut MPI_Status {
    unsafe { RSMPI_STATUS_IGNORE }
}
pub fn RSMPI_STATUSES_IGNORE_fn() -> *mut MPI_Status {
    unsafe { RSMPI_STATUSES_IGNORE }
}

// Comparison results
pub fn RSMPI_IDENT_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_IDENT }
}
pub fn RSMPI_CONGRUENT_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_CONGRUENT }
}
pub fn RSMPI_SIMILAR_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_SIMILAR }
}
pub fn RSMPI_UNEQUAL_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_UNEQUAL }
}

// Threading
pub fn RSMPI_THREAD_SINGLE_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_THREAD_SINGLE }
}
pub fn RSMPI_THREAD_FUNNELED_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_THREAD_FUNNELED }
}
pub fn RSMPI_THREAD_SERIALIZED_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_THREAD_SERIALIZED }
}
pub fn RSMPI_THREAD_MULTIPLE_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_THREAD_MULTIPLE }
}

// Topology
pub fn RSMPI_GRAPH_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_GRAPH }
}
pub fn RSMPI_CART_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_CART }
}
pub fn RSMPI_DIST_GRAPH_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_DIST_GRAPH }
}

// Limits
pub fn RSMPI_MAX_LIBRARY_VERSION_STRING_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_MAX_LIBRARY_VERSION_STRING }
}
pub fn RSMPI_MAX_PROCESSOR_NAME_fn() -> std::os::raw::c_int {
    unsafe { RSMPI_MAX_PROCESSOR_NAME }
}

// Operations
pub fn RSMPI_MAX_fn() -> MPI_Op {
    unsafe { RSMPI_MAX }
}
pub fn RSMPI_MIN_fn() -> MPI_Op {
    unsafe { RSMPI_MIN }
}
pub fn RSMPI_SUM_fn() -> MPI_Op {
    unsafe { RSMPI_SUM }
}
pub fn RSMPI_PROD_fn() -> MPI_Op {
    unsafe { RSMPI_PROD }
}
pub fn RSMPI_LAND_fn() -> MPI_Op {
    unsafe { RSMPI_LAND }
}
pub fn RSMPI_BAND_fn() -> MPI_Op {
    unsafe { RSMPI_BAND }
}
pub fn RSMPI_LOR_fn() -> MPI_Op {
    unsafe { RSMPI_LOR }
}
pub fn RSMPI_BOR_fn() -> MPI_Op {
    unsafe { RSMPI_BOR }
}
pub fn RSMPI_LXOR_fn() -> MPI_Op {
    unsafe { RSMPI_LXOR }
}
pub fn RSMPI_BXOR_fn() -> MPI_Op {
    unsafe { RSMPI_BXOR }
}

// Error handlers
pub fn RSMPI_ERRORS_ARE_FATAL_fn() -> MPI_Errhandler {
    unsafe { RSMPI_ERRORS_ARE_FATAL }
}
pub fn RSMPI_ERRORS_RETURN_fn() -> MPI_Errhandler {
    unsafe { RSMPI_ERRORS_RETURN }
}

// File
pub fn RSMPI_FILE_NULL_fn() -> MPI_File {
    unsafe { RSMPI_FILE_NULL }
}

// Info
pub fn RSMPI_INFO_NULL_fn() -> MPI_Info {
    unsafe { RSMPI_INFO_NULL }
}

// Win
pub fn RSMPI_WIN_NULL_fn() -> MPI_Win {
    unsafe { RSMPI_WIN_NULL }
}

// Error codes and misc integer constants
pub fn RSMPI_SUCCESS_fn() -> std::os::raw::c_int {
    MPI_SUCCESS as std::os::raw::c_int
}
pub fn RSMPI_UNIVERSE_SIZE_fn() -> std::os::raw::c_int {
    MPI_UNIVERSE_SIZE as std::os::raw::c_int
}
pub fn RSMPI_APPNUM_fn() -> std::os::raw::c_int {
    MPI_APPNUM as std::os::raw::c_int
}

// Compile-time size constants
pub const RSMPI_MAX_OBJECT_NAME: usize = MPI_MAX_OBJECT_NAME as usize;
