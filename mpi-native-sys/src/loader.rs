//! Finding the host's MPI library and resolving symbols in it (D2, D3, D12).
//!
//! **Attach path.** Walk the loaded objects with `dl_iterate_phdr`; take a reference to each
//! with `dlopen(path, RTLD_NOLOAD | RTLD_LAZY)` (never maps anything new); build a
//! [`Candidate`] per object; hand the list to [`crate::detect::decide`]. The reference is
//! never `dlclose`d, so a host `dlclose` cannot unmap the library under a live object.
//!
//! **Standalone path.** Only when the walk finds nothing: load `MPI_NATIVE_LIB`, else a
//! list of sonames, then walk again.
//!
//! **Caching (D12).** Detection runs under a `Mutex`; only a *success* is stored.

use std::{
    ffi::{c_void, CStr, CString},
    os::raw::{c_char, c_int},
    sync::{Mutex, OnceLock},
};

use crate::{
    consts::{Consts, VERSION_BUFFER},
    detect::{decide, Candidate, Family, LoadError},
};

/// A detected, referenced MPI library. Process-global and never released.
#[derive(Debug)]
pub struct Loaded {
    /// The family.
    pub family: Family,
    /// The object's path as the dynamic linker reports it ("" for the main executable).
    pub path: String,
    /// `MPI_Get_library_version`.
    pub version: String,
    handle: usize,
    /// The family's constants.
    pub consts: Consts,
}

impl Loaded {
    /// Resolve `name` in the identified object (and its dependencies, as `dlsym` does).
    pub fn sym(&self, name: &CStr) -> Option<*mut c_void> {
        // SAFETY: `handle` is a live dlopen reference that is never closed.
        let p = unsafe { libc::dlsym(self.handle as *mut c_void, name.as_ptr()) };
        (!p.is_null()).then_some(p)
    }
}

static LIBRARY: OnceLock<Loaded> = OnceLock::new();
static DETECT: Mutex<()> = Mutex::new(());

/// One loaded object seen by the walk.
struct Object {
    path: String,
    segments: Vec<(usize, usize)>,
}

fn walk() -> Vec<Object> {
    unsafe extern "C" fn cb(
        info: *mut libc::dl_phdr_info,
        _size: usize,
        data: *mut c_void,
    ) -> c_int {
        let objects = &mut *(data as *mut Vec<Object>);
        let info = &*info;
        let path = if info.dlpi_name.is_null() {
            String::new()
        } else {
            CStr::from_ptr(info.dlpi_name)
                .to_string_lossy()
                .into_owned()
        };
        let mut segments = Vec::new();
        for i in 0..info.dlpi_phnum as usize {
            let ph = &*info.dlpi_phdr.add(i);
            if ph.p_type == libc::PT_LOAD {
                let start = info.dlpi_addr as usize + ph.p_vaddr as usize;
                segments.push((start, start + ph.p_memsz as usize));
            }
        }
        objects.push(Object { path, segments });
        0
    }
    let mut objects: Vec<Object> = Vec::new();
    // SAFETY: the callback only reads the info it is given and appends to `objects`.
    unsafe { libc::dl_iterate_phdr(Some(cb), (&mut objects as *mut Vec<Object>).cast()) };
    objects
}

/// A symbol defined *by this object*, not by one of its dependencies: `dlsym` on a handle
/// also searches dependencies, so a library that merely links `libmpi_abi` would otherwise
/// look like a second MPI. The address must fall inside one of the object's load segments.
unsafe fn own_symbol(handle: *mut c_void, object: &Object, name: &CStr) -> *mut c_void {
    let p = libc::dlsym(handle, name.as_ptr());
    if p.is_null() {
        return p;
    }
    let a = p as usize;
    if object.segments.iter().any(|&(s, e)| a >= s && a < e) {
        p
    } else {
        std::ptr::null_mut()
    }
}

unsafe fn library_version(f: *mut c_void) -> Option<String> {
    let f: unsafe extern "C" fn(*mut c_char, *mut c_int) -> c_int = std::mem::transmute(f);
    let mut buf = vec![0u8; VERSION_BUFFER + 1];
    let mut len: c_int = 0;
    if f(buf.as_mut_ptr().cast(), &mut len) != 0 {
        return None;
    }
    let n = (len.max(0) as usize).min(VERSION_BUFFER);
    Some(String::from_utf8_lossy(&buf[..n]).into_owned())
}

fn c(name: &str) -> CString {
    CString::new(name).expect("symbol names contain no NUL")
}

fn probe(object: &Object) -> Option<(Candidate, usize)> {
    // SAFETY: dlopen with RTLD_NOLOAD returns a reference only to an object that is
    // already mapped; every call below is a documented pre-init-safe MPI query or a
    // symbol lookup.
    unsafe {
        let handle = if object.path.is_empty() {
            libc::dlopen(std::ptr::null(), libc::RTLD_LAZY)
        } else {
            let p = c(&object.path);
            libc::dlopen(p.as_ptr(), libc::RTLD_NOLOAD | libc::RTLD_LAZY)
        };
        if handle.is_null() {
            return None;
        }
        let own = |n: &str| own_symbol(handle, object, &c(n));
        let mut cand = Candidate {
            path: object.path.clone(),
            ..Default::default()
        };
        cand.mpiabi_marker = !own("MPIABI_Get_library_version").is_null();
        let abi = own("MPI_Abi_get_version");
        let pmpi = own("PMPI_Get_library_version");
        cand.pmpi_version = !pmpi.is_null();
        if !abi.is_null() {
            // The return value is the signal; the symbol's presence is not (A5).
            let f: unsafe extern "C" fn(*mut c_int, *mut c_int) -> c_int = std::mem::transmute(abi);
            let (mut major, mut minor) = (-1, -1);
            if f(&mut major, &mut minor) == 0 {
                cand.abi_major = Some(major);
                cand.abi_minor = Some(minor);
            }
        }
        if !pmpi.is_null() {
            cand.version = library_version(pmpi);
        } else if cand.abi_major.is_some_and(|m| m >= 1) {
            let f = own("MPI_Get_library_version");
            if !f.is_null() {
                cand.version = library_version(f);
            }
        }
        cand.ompi_witness = !own("ompi_mpi_comm_world").is_null();
        cand.mpich_witness = !own("MPIR_Err_return_comm").is_null();
        Some((cand, handle as usize))
    }
}

/// Run detection on the objects loaded right now. Does not touch the cache.
pub fn detect_in_process() -> Result<Loaded, LoadError> {
    let objects = walk();
    let mut candidates = Vec::new();
    let mut handles = Vec::new();
    let mut origin = Vec::new();
    for (i, o) in objects.iter().enumerate() {
        // Only objects that could be MPI are probed, so unrelated libraries are never called.
        if let Some((cand, handle)) = probe(o) {
            let interesting = cand.mpiabi_marker
                || cand.pmpi_version
                || cand.abi_major.is_some()
                || cand.ompi_witness;
            if interesting {
                candidates.push(cand);
                handles.push(handle);
                origin.push(i);
            }
        }
    }
    let decision = decide(&candidates)?;
    let cand = &candidates[decision.index];
    let handle = handles[decision.index];
    let consts = Consts::for_family(decision.family, |name| {
        // SAFETY: `handle` is a live reference; a data symbol's address is its value.
        let p = unsafe { libc::dlsym(handle as *mut c_void, c(name).as_ptr()) };
        assert!(
            !p.is_null(),
            "Open MPI library {} does not export {name}",
            cand.path
        );
        p as usize
    });
    Ok(Loaded {
        family: decision.family,
        path: cand.path.clone(),
        version: cand.version.clone().unwrap_or_default(),
        handle,
        consts,
    })
}

/// The host's MPI library, detected in the objects already loaded. Failure (for example
/// "no MPI loaded yet") is returned and **not** cached (D12); success is final.
pub fn library() -> Result<&'static Loaded, LoadError> {
    if let Some(l) = LIBRARY.get() {
        return Ok(l);
    }
    let _guard = DETECT.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(l) = LIBRARY.get() {
        return Ok(l);
    }
    let loaded = detect_in_process()?;
    Ok(LIBRARY.get_or_init(|| loaded))
}

/// Names tried by the standalone path, in order, when `MPI_NATIVE_LIB` is not set.
pub fn default_sonames() -> &'static [&'static str] {
    if cfg!(target_os = "macos") {
        &["libmpi.12.dylib", "libmpi.40.dylib"]
    } else {
        &[
            "libmpi.so.12",
            "libmpi.so.40",
            "libmpi_cray.so.12",
            "libmpi_abi.so.1",
        ]
    }
}

/// Standalone path: [`library`], and if nothing is loaded, load `MPI_NATIVE_LIB` or a
/// default soname and detect again. For a Rust program that calls `mpi::initialize()`
/// itself. `MPI_RT_LIB` is not read.
pub fn standalone() -> Result<&'static Loaded, LoadError> {
    match library() {
        Err(LoadError::NoMpiInProcess) => {}
        other => return other,
    }
    let names: Vec<String> = match std::env::var("MPI_NATIVE_LIB") {
        Ok(p) if !p.is_empty() => vec![p],
        _ => default_sonames().iter().map(|s| s.to_string()).collect(),
    };
    let mut loaded_any = false;
    for name in &names {
        let n = c(name);
        // SAFETY: loading the library the user (or the default soname list) named. RTLD_GLOBAL
        // because Open MPI's component plugins resolve libmpi symbols from the global scope.
        let h = unsafe { libc::dlopen(n.as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL) };
        if !h.is_null() {
            loaded_any = true;
            break;
        }
    }
    if !loaded_any {
        return Err(LoadError::NotFound { tried: names });
    }
    library()
}

/// The library for the thunks: standalone detection, panicking with the error text if it
/// fails. The C entry never reaches here before [`library`] has succeeded (D10).
pub(crate) fn loaded() -> &'static Loaded {
    if let Some(l) = LIBRARY.get() {
        return l;
    }
    match standalone() {
        Ok(l) => l,
        Err(e) => panic!("{e}"),
    }
}
