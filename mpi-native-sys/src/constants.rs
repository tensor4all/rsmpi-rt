//! The carried constant accessors. Each exists as `RSMPI_X()` and `RSMPI_X_fn()`, the two
//! spellings rsmpi-rt's `ffi` module uses. Values come from the identified family's table.

use std::os::raw::c_int;

use crate::loader::loaded;
use crate::types::*;

macro_rules! constant {
    ($plain:ident, $fnname:ident, $ty:ty, $field:ident) => {
        pub fn $plain() -> $ty {
            loaded().consts.$field as $ty
        }
        pub fn $fnname() -> $ty {
            $plain()
        }
    };
}

constant!(RSMPI_COMM_WORLD, RSMPI_COMM_WORLD_fn, MPI_Comm, comm_world);
constant!(RSMPI_COMM_SELF, RSMPI_COMM_SELF_fn, MPI_Comm, comm_self);
constant!(RSMPI_COMM_NULL, RSMPI_COMM_NULL_fn, MPI_Comm, comm_null);
constant!(RSMPI_DATATYPE_NULL, RSMPI_DATATYPE_NULL_fn, MPI_Datatype, datatype_null);
constant!(RSMPI_INFO_NULL, RSMPI_INFO_NULL_fn, MPI_Info, info_null);
constant!(RSMPI_MESSAGE_NULL, RSMPI_MESSAGE_NULL_fn, MPI_Message, message_null);
constant!(RSMPI_MESSAGE_NO_PROC, RSMPI_MESSAGE_NO_PROC_fn, MPI_Message, message_no_proc);
constant!(RSMPI_REQUEST_NULL, RSMPI_REQUEST_NULL_fn, MPI_Request, request_null);
constant!(RSMPI_GROUP_NULL, RSMPI_GROUP_NULL_fn, MPI_Group, group_null);
constant!(RSMPI_GROUP_EMPTY, RSMPI_GROUP_EMPTY_fn, MPI_Group, group_empty);
constant!(RSMPI_ERRORS_ARE_FATAL, RSMPI_ERRORS_ARE_FATAL_fn, MPI_Errhandler, errors_are_fatal);
constant!(RSMPI_ERRORS_RETURN, RSMPI_ERRORS_RETURN_fn, MPI_Errhandler, errors_return);

constant!(RSMPI_ANY_SOURCE, RSMPI_ANY_SOURCE_fn, c_int, any_source);
constant!(RSMPI_ANY_TAG, RSMPI_ANY_TAG_fn, c_int, any_tag);
constant!(RSMPI_PROC_NULL, RSMPI_PROC_NULL_fn, c_int, proc_null);
constant!(RSMPI_UNDEFINED, RSMPI_UNDEFINED_fn, c_int, undefined);
constant!(RSMPI_THREAD_SINGLE, RSMPI_THREAD_SINGLE_fn, c_int, thread_single);
constant!(RSMPI_THREAD_FUNNELED, RSMPI_THREAD_FUNNELED_fn, c_int, thread_funneled);
constant!(RSMPI_THREAD_SERIALIZED, RSMPI_THREAD_SERIALIZED_fn, c_int, thread_serialized);
constant!(RSMPI_THREAD_MULTIPLE, RSMPI_THREAD_MULTIPLE_fn, c_int, thread_multiple);
constant!(RSMPI_TAG_UB, RSMPI_TAG_UB_fn, c_int, tag_ub);
constant!(RSMPI_APPNUM, RSMPI_APPNUM_fn, c_int, appnum);
constant!(RSMPI_UNIVERSE_SIZE, RSMPI_UNIVERSE_SIZE_fn, c_int, universe_size);
constant!(RSMPI_IDENT, RSMPI_IDENT_fn, c_int, ident);
constant!(RSMPI_CONGRUENT, RSMPI_CONGRUENT_fn, c_int, congruent);
constant!(RSMPI_SIMILAR, RSMPI_SIMILAR_fn, c_int, similar);
constant!(RSMPI_UNEQUAL, RSMPI_UNEQUAL_fn, c_int, unequal);
constant!(RSMPI_CART, RSMPI_CART_fn, c_int, cart);
constant!(RSMPI_GRAPH, RSMPI_GRAPH_fn, c_int, graph);
constant!(RSMPI_DIST_GRAPH, RSMPI_DIST_GRAPH_fn, c_int, dist_graph);
constant!(RSMPI_COMM_TYPE_SHARED, RSMPI_COMM_TYPE_SHARED_fn, c_int, comm_type_shared);
constant!(RSMPI_MAX_PROCESSOR_NAME, RSMPI_MAX_PROCESSOR_NAME_fn, c_int, max_processor_name);
constant!(
    RSMPI_MAX_LIBRARY_VERSION_STRING,
    RSMPI_MAX_LIBRARY_VERSION_STRING_fn,
    c_int,
    max_library_version_string
);

constant!(RSMPI_BYTE, RSMPI_BYTE_fn, MPI_Datatype, byte);
constant!(RSMPI_INT8_T, RSMPI_INT8_T_fn, MPI_Datatype, int8_t);
constant!(RSMPI_INT16_T, RSMPI_INT16_T_fn, MPI_Datatype, int16_t);
constant!(RSMPI_INT32_T, RSMPI_INT32_T_fn, MPI_Datatype, int32_t);
constant!(RSMPI_INT64_T, RSMPI_INT64_T_fn, MPI_Datatype, int64_t);
constant!(RSMPI_UINT8_T, RSMPI_UINT8_T_fn, MPI_Datatype, uint8_t);
constant!(RSMPI_UINT16_T, RSMPI_UINT16_T_fn, MPI_Datatype, uint16_t);
constant!(RSMPI_UINT32_T, RSMPI_UINT32_T_fn, MPI_Datatype, uint32_t);
constant!(RSMPI_UINT64_T, RSMPI_UINT64_T_fn, MPI_Datatype, uint64_t);
constant!(RSMPI_FLOAT, RSMPI_FLOAT_fn, MPI_Datatype, float);
constant!(RSMPI_DOUBLE, RSMPI_DOUBLE_fn, MPI_Datatype, double);
constant!(RSMPI_C_BOOL, RSMPI_C_BOOL_fn, MPI_Datatype, c_bool);

/// `MPI_SUCCESS` is 0 in every family; this needs no library.
pub fn RSMPI_SUCCESS() -> c_int {
    crate::consts::SUCCESS
}
/// See [`RSMPI_SUCCESS`].
pub fn RSMPI_SUCCESS_fn() -> c_int {
    crate::consts::SUCCESS
}

/// The neutral `MPI_STATUS_IGNORE` is a null pointer; the thunks substitute the family's
/// own sentinel.
pub fn RSMPI_STATUS_IGNORE() -> *mut MPI_Status {
    std::ptr::null_mut()
}
/// See [`RSMPI_STATUS_IGNORE`].
pub fn RSMPI_STATUS_IGNORE_fn() -> *mut MPI_Status {
    std::ptr::null_mut()
}

/// Upper bound over the families (Open MPI 64, MPICH and the standard ABI 128).
pub const MPI_MAX_OBJECT_NAME: usize = 128;
/// See [`MPI_MAX_OBJECT_NAME`].
pub const RSMPI_MAX_OBJECT_NAME: usize = MPI_MAX_OBJECT_NAME;

/// `RSMPI_Wtime`.
pub unsafe fn RSMPI_Wtime() -> std::os::raw::c_double {
    crate::functions::MPI_Wtime()
}
/// `RSMPI_Wtick`.
pub unsafe fn RSMPI_Wtick() -> std::os::raw::c_double {
    crate::functions::MPI_Wtick()
}
