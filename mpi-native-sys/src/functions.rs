//! The carried MPI function set, with per-family thunks (D5).
//!
//! Above this layer every handle is a `usize`. Here each function exists twice, as separate
//! function-pointer types: `c_int` handles for the MPICH ABI, `*mut c_void` handles for the
//! Open MPI ABI and the standard ABI. The two pointer families share one thunk because they
//! differ only in their constant tables and status layout.
//!
//! The carried set is `docs/design/mpi-native-abi.md` §3's "may" list plus what rsmpi-rt's
//! communicator construction calls. Every other `ffi` name is a panic stub in
//! [`crate::stubs`].

use std::ffi::{c_void, CString};
use std::os::raw::{c_char, c_double, c_int};
use std::sync::OnceLock;

use crate::detect::Family;
use crate::loader::{loaded, Loaded};
use crate::types::*;

#[inline]
fn to_int(h: usize) -> c_int {
    // An MPICH-ABI handle is 32 bits. A non-zero upper half is an internal error, never
    // a silent truncation.
    if h > u32::MAX as usize {
        panic!("internal error: handle {h:#x} does not fit a 32-bit MPICH-ABI handle");
    }
    h as u32 as c_int
}
#[inline]
fn to_ptr(h: usize) -> *mut c_void {
    h as *mut c_void
}
#[inline]
fn from_int(c: c_int) -> usize {
    c as u32 as usize
}
#[inline]
fn from_ptr(p: *mut c_void) -> usize {
    p as usize
}

fn resolve(lib: &Loaded, name: &str) -> usize {
    let c = CString::new(name).expect("no NUL in symbol names");
    match lib.sym(&c) {
        Some(p) => p as usize,
        None => panic!(
            "tenferro-mpi: the MPI library {} does not export {name}",
            lib.path
        ),
    }
}

/// `MPI_Status *` as the family's C library expects it: a null neutral pointer is
/// `MPI_STATUS_IGNORE`, whose value differs per family.
#[inline]
fn status_arg(lib: &Loaded, status: *mut MPI_Status) -> *mut c_void {
    if status.is_null() {
        lib.consts.status_ignore as *mut c_void
    } else {
        // SAFETY: the caller passed a valid status.
        unsafe { (*status).raw_ptr() }
    }
}

/// After a call that wrote `raw`, copy source/tag/error out at the family's offsets.
#[inline]
unsafe fn read_status(lib: &Loaded, status: *mut MPI_Status) {
    if status.is_null() {
        return;
    }
    let (s, t, e) = lib.consts.status_offsets;
    let st = &mut *status;
    st.MPI_SOURCE = st.read_i32(s);
    st.MPI_TAG = st.read_i32(t);
    st.MPI_ERROR = st.read_i32(e);
}

macro_rules! neutral_ty {
    (h) => { usize };
    (cs) => { *const MPI_Status };
    (s) => { *mut MPI_Status };
    (v $t:ty) => { $t };
}
macro_rules! int_ty {
    (h) => { c_int };
    (cs) => { *const c_void };
    (s) => { *mut c_void };
    (v $t:ty) => { $t };
}
macro_rules! ptr_ty {
    (h) => { *mut c_void };
    (cs) => { *const c_void };
    (s) => { *mut c_void };
    (v $t:ty) => { $t };
}
macro_rules! int_conv {
    ($lib:ident, $a:ident, h) => { to_int($a) };
    ($lib:ident, $a:ident, cs) => { (*$a).raw_const_ptr() };
    ($lib:ident, $a:ident, s) => { status_arg($lib, $a) };
    ($lib:ident, $a:ident, v) => { $a };
}
macro_rules! ptr_conv {
    ($lib:ident, $a:ident, h) => { to_ptr($a) };
    ($lib:ident, $a:ident, cs) => { (*$a).raw_const_ptr() };
    ($lib:ident, $a:ident, s) => { status_arg($lib, $a) };
    ($lib:ident, $a:ident, v) => { $a };
}

/// A thunk whose arguments are handles (`h`), statuses (`s`, `cs`) or plain values
/// (`v name: ty`). Out-parameter handles are written by hand below.
macro_rules! thunk {
    ($name:ident ( $( $k:ident $arg:ident $(: $ty:ty)? ),* ) -> $ret:ty) => {
        pub unsafe fn $name($( $arg: neutral_ty!($k $($ty)?) ),*) -> $ret {
            let lib = loaded();
            let csym = &stringify!($name)[..];
            match lib.family {
                Family::Mpich => {
                    type F = unsafe extern "C" fn($( int_ty!($k $($ty)?) ),*) -> $ret;
                    static FN: OnceLock<usize> = OnceLock::new();
                    let p = *FN.get_or_init(|| resolve(lib, csym));
                    let f: F = std::mem::transmute::<usize, F>(p);
                    f($( int_conv!(lib, $arg, $k) ),*)
                }
                Family::OpenMpi | Family::StandardAbi => {
                    type F = unsafe extern "C" fn($( ptr_ty!($k $($ty)?) ),*) -> $ret;
                    static FN: OnceLock<usize> = OnceLock::new();
                    let p = *FN.get_or_init(|| resolve(lib, csym));
                    let f: F = std::mem::transmute::<usize, F>(p);
                    f($( ptr_conv!(lib, $arg, $k) ),*)
                }
            }
        }
    };
}

thunk!(MPI_Init_thread(v argc: *mut c_int, v argv: *mut *mut *mut c_char, v required: c_int, v provided: *mut c_int) -> c_int);
thunk!(MPI_Initialized(v flag: *mut c_int) -> c_int);
thunk!(MPI_Finalize() -> c_int);
thunk!(MPI_Finalized(v flag: *mut c_int) -> c_int);
thunk!(MPI_Query_thread(v provided: *mut c_int) -> c_int);
thunk!(MPI_Is_thread_main(v flag: *mut c_int) -> c_int);
thunk!(MPI_Get_version(v version: *mut c_int, v subversion: *mut c_int) -> c_int);
thunk!(MPI_Get_library_version(v version: *mut c_char, v resultlen: *mut c_int) -> c_int);
thunk!(MPI_Comm_rank(h comm, v rank: *mut c_int) -> c_int);
thunk!(MPI_Comm_size(h comm, v size: *mut c_int) -> c_int);
thunk!(MPI_Comm_test_inter(h comm, v flag: *mut c_int) -> c_int);
thunk!(MPI_Comm_compare(h comm1, h comm2, v result: *mut c_int) -> c_int);
thunk!(MPI_Comm_get_attr(h comm, v comm_keyval: c_int, v attribute_val: *mut c_void, v flag: *mut c_int) -> c_int);
thunk!(MPI_Send(v buf: *const c_void, v count: c_int, h datatype, v dest: c_int, v tag: c_int, h comm) -> c_int);
thunk!(MPI_Get_count(cs status, h datatype, v count: *mut c_int) -> c_int);
thunk!(MPI_Error_string(v errorcode: c_int, v string: *mut c_char, v resultlen: *mut c_int) -> c_int);
thunk!(MPI_Barrier(h comm) -> c_int);
thunk!(MPI_Recv(v buf: *mut c_void, v count: c_int, h datatype, v source: c_int, v tag: c_int, h comm, s status) -> c_int);
thunk!(MPI_Wtime() -> c_double);
thunk!(MPI_Wtick() -> c_double);

/// `MPI_Comm_get_parent(MPI_Comm *parent)`.
pub unsafe fn MPI_Comm_get_parent(parent: *mut MPI_Comm) -> c_int {
    let lib = loaded();
    match lib.family {
        Family::Mpich => {
            type F = unsafe extern "C" fn(*mut c_int) -> c_int;
            static FN: OnceLock<usize> = OnceLock::new();
            let f: F = std::mem::transmute(*FN.get_or_init(|| resolve(lib, "MPI_Comm_get_parent")));
            let mut slot: c_int = 0;
            let rc = f(&mut slot);
            *parent = from_int(slot);
            rc
        }
        Family::OpenMpi | Family::StandardAbi => {
            type F = unsafe extern "C" fn(*mut *mut c_void) -> c_int;
            static FN: OnceLock<usize> = OnceLock::new();
            let f: F = std::mem::transmute(*FN.get_or_init(|| resolve(lib, "MPI_Comm_get_parent")));
            let mut slot: *mut c_void = std::ptr::null_mut();
            let rc = f(&mut slot);
            *parent = from_ptr(slot);
            rc
        }
    }
}

/// `MPI_Comm_dup(comm, MPI_Comm *newcomm)`. A standalone Rust host that gives a library its own
/// communicator needs it; the tenferro transport itself never calls it.
pub unsafe fn MPI_Comm_dup(comm: MPI_Comm, newcomm: *mut MPI_Comm) -> c_int {
    let lib = loaded();
    match lib.family {
        Family::Mpich => {
            type F = unsafe extern "C" fn(c_int, *mut c_int) -> c_int;
            static FN: OnceLock<usize> = OnceLock::new();
            let f: F = std::mem::transmute(*FN.get_or_init(|| resolve(lib, "MPI_Comm_dup")));
            let mut slot: c_int = 0;
            let rc = f(to_int(comm), &mut slot);
            *newcomm = from_int(slot);
            rc
        }
        Family::OpenMpi | Family::StandardAbi => {
            type F = unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> c_int;
            static FN: OnceLock<usize> = OnceLock::new();
            let f: F = std::mem::transmute(*FN.get_or_init(|| resolve(lib, "MPI_Comm_dup")));
            let mut slot: *mut c_void = std::ptr::null_mut();
            let rc = f(to_ptr(comm), &mut slot);
            *newcomm = from_ptr(slot);
            rc
        }
    }
}

/// `MPI_Comm_free(MPI_Comm *comm)`: reads the handle and writes back `MPI_COMM_NULL`.
pub unsafe fn MPI_Comm_free(comm: *mut MPI_Comm) -> c_int {
    let lib = loaded();
    match lib.family {
        Family::Mpich => {
            type F = unsafe extern "C" fn(*mut c_int) -> c_int;
            static FN: OnceLock<usize> = OnceLock::new();
            let f: F = std::mem::transmute(*FN.get_or_init(|| resolve(lib, "MPI_Comm_free")));
            let mut slot: c_int = to_int(*comm);
            let rc = f(&mut slot);
            *comm = from_int(slot);
            rc
        }
        Family::OpenMpi | Family::StandardAbi => {
            type F = unsafe extern "C" fn(*mut *mut c_void) -> c_int;
            static FN: OnceLock<usize> = OnceLock::new();
            let f: F = std::mem::transmute(*FN.get_or_init(|| resolve(lib, "MPI_Comm_free")));
            let mut slot: *mut c_void = to_ptr(*comm);
            let rc = f(&mut slot);
            *comm = from_ptr(slot);
            rc
        }
    }
}

/// `MPI_Comm_get_errhandler(comm, MPI_Errhandler *errhandler)`.
pub unsafe fn MPI_Comm_get_errhandler(comm: MPI_Comm, errhandler: *mut MPI_Errhandler) -> c_int {
    let lib = loaded();
    match lib.family {
        Family::Mpich => {
            type F = unsafe extern "C" fn(c_int, *mut c_int) -> c_int;
            static FN: OnceLock<usize> = OnceLock::new();
            let f: F = std::mem::transmute(*FN.get_or_init(|| resolve(lib, "MPI_Comm_get_errhandler")));
            let mut slot: c_int = 0;
            let rc = f(to_int(comm), &mut slot);
            *errhandler = from_int(slot);
            rc
        }
        Family::OpenMpi | Family::StandardAbi => {
            type F = unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> c_int;
            static FN: OnceLock<usize> = OnceLock::new();
            let f: F = std::mem::transmute(*FN.get_or_init(|| resolve(lib, "MPI_Comm_get_errhandler")));
            let mut slot: *mut c_void = std::ptr::null_mut();
            let rc = f(to_ptr(comm), &mut slot);
            *errhandler = from_ptr(slot);
            rc
        }
    }
}

/// `MPI_Errhandler_free(MPI_Errhandler *errhandler)`: reads the handle, and writes back what
/// the library leaves (`MPI_ERRHANDLER_NULL`).
pub unsafe fn MPI_Errhandler_free(errhandler: *mut MPI_Errhandler) -> c_int {
    let lib = loaded();
    match lib.family {
        Family::Mpich => {
            type F = unsafe extern "C" fn(*mut c_int) -> c_int;
            static FN: OnceLock<usize> = OnceLock::new();
            let f: F = std::mem::transmute(*FN.get_or_init(|| resolve(lib, "MPI_Errhandler_free")));
            let mut slot: c_int = to_int(*errhandler);
            let rc = f(&mut slot);
            *errhandler = from_int(slot);
            rc
        }
        Family::OpenMpi | Family::StandardAbi => {
            type F = unsafe extern "C" fn(*mut *mut c_void) -> c_int;
            static FN: OnceLock<usize> = OnceLock::new();
            let f: F = std::mem::transmute(*FN.get_or_init(|| resolve(lib, "MPI_Errhandler_free")));
            let mut slot: *mut c_void = to_ptr(*errhandler);
            let rc = f(&mut slot);
            *errhandler = from_ptr(slot);
            rc
        }
    }
}

/// `MPI_Mprobe(source, tag, comm, MPI_Message *message, MPI_Status *status)`.
pub unsafe fn MPI_Mprobe(
    source: c_int,
    tag: c_int,
    comm: MPI_Comm,
    message: *mut MPI_Message,
    status: *mut MPI_Status,
) -> c_int {
    let lib = loaded();
    let rc = match lib.family {
        Family::Mpich => {
            type F = unsafe extern "C" fn(c_int, c_int, c_int, *mut c_int, *mut c_void) -> c_int;
            static FN: OnceLock<usize> = OnceLock::new();
            let f: F = std::mem::transmute(*FN.get_or_init(|| resolve(lib, "MPI_Mprobe")));
            let mut slot: c_int = 0;
            let rc = f(source, tag, to_int(comm), &mut slot, status_arg(lib, status));
            *message = from_int(slot);
            rc
        }
        Family::OpenMpi | Family::StandardAbi => {
            type F = unsafe extern "C" fn(c_int, c_int, *mut c_void, *mut *mut c_void, *mut c_void) -> c_int;
            static FN: OnceLock<usize> = OnceLock::new();
            let f: F = std::mem::transmute(*FN.get_or_init(|| resolve(lib, "MPI_Mprobe")));
            let mut slot: *mut c_void = std::ptr::null_mut();
            let rc = f(source, tag, to_ptr(comm), &mut slot, status_arg(lib, status));
            *message = from_ptr(slot);
            rc
        }
    };
    read_status(lib, status);
    rc
}

/// `MPI_Mrecv(buf, count, datatype, MPI_Message *message, MPI_Status *status)`: the message
/// handle is in/out (the library sets it to `MPI_MESSAGE_NULL`).
pub unsafe fn MPI_Mrecv(
    buf: *mut c_void,
    count: c_int,
    datatype: MPI_Datatype,
    message: *mut MPI_Message,
    status: *mut MPI_Status,
) -> c_int {
    let lib = loaded();
    let rc = match lib.family {
        Family::Mpich => {
            type F = unsafe extern "C" fn(*mut c_void, c_int, c_int, *mut c_int, *mut c_void) -> c_int;
            static FN: OnceLock<usize> = OnceLock::new();
            let f: F = std::mem::transmute(*FN.get_or_init(|| resolve(lib, "MPI_Mrecv")));
            let mut slot: c_int = to_int(*message);
            let rc = f(buf, count, to_int(datatype), &mut slot, status_arg(lib, status));
            *message = from_int(slot);
            rc
        }
        Family::OpenMpi | Family::StandardAbi => {
            type F = unsafe extern "C" fn(*mut c_void, c_int, *mut c_void, *mut *mut c_void, *mut c_void) -> c_int;
            static FN: OnceLock<usize> = OnceLock::new();
            let f: F = std::mem::transmute(*FN.get_or_init(|| resolve(lib, "MPI_Mrecv")));
            let mut slot: *mut c_void = to_ptr(*message);
            let rc = f(buf, count, to_ptr(datatype), &mut slot, status_arg(lib, status));
            *message = from_ptr(slot);
            rc
        }
    };
    read_status(lib, status);
    rc
}
