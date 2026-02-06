//! Dynamic loading of MPIwrapper library via MPITRAMPOLINE_LIB

use libloading::Library;
use std::sync::OnceLock;

static LIBRARY: OnceLock<Library> = OnceLock::new();

/// Get or load the MPIwrapper library.
///
/// The library path is read from the `MPITRAMPOLINE_LIB` environment variable.
/// Panics if the variable is not set or the library cannot be loaded.
pub fn library() -> &'static Library {
    LIBRARY.get_or_init(|| {
        let path = std::env::var("MPITRAMPOLINE_LIB")
            .expect("MPITRAMPOLINE_LIB environment variable not set");
        unsafe { Library::new(&path) }
            .unwrap_or_else(|e| panic!("Failed to load MPIwrapper library '{}': {}", path, e))
    })
}

/// Load a symbol from the MPIwrapper library.
///
/// # Safety
/// The caller must ensure that the symbol type matches the actual symbol.
pub unsafe fn get_symbol<T>(name: &[u8]) -> *const T {
    let lib = library();
    let sym: libloading::Symbol<*const T> = lib.get(name).unwrap_or_else(|e| {
        panic!(
            "Failed to load symbol '{}': {}",
            String::from_utf8_lossy(name),
            e
        )
    });
    *sym
}
