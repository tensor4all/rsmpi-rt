//! Low-level MPI bindings through the **host's native handles**, resolved at run time.
//!
//! This is the backend behind rsmpi-rt's `mpi-native-sys-backend` feature. It replaces the
//! MPItrampoline/MPIwrapper path: there is no wrapper library and no `MPI_RT_LIB`. The MPI
//! library already loaded in the process (or, for a standalone Rust program, one found by
//! soname or `MPI_NATIVE_LIB`) is identified by [`detect`], and every call goes to that very
//! library object with that family's handles and constants.
//!
//! Families: the MPICH ABI, the Open MPI ABI, and the MPI 5.0 standard ABI.
//!
//! Above this crate every handle is a `usize`; [`functions`] converts to the family's real
//! `int` or pointer. The carried function set is real; every other name rsmpi-rt's `ffi`
//! module references is a panic stub ([`stubs`]) that a C boundary must catch.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc, clippy::too_many_arguments, clippy::type_complexity)]

pub mod callback_types;
pub mod consts;
pub mod constants;
pub mod detect;
pub mod functions;
pub mod loader;
mod stubs;
pub mod types;

use std::sync::atomic::{AtomicUsize, Ordering};

pub use callback_types::*;
pub use constants::*;
pub use functions::*;
pub use stubs::*;
pub use types::*;

/// The panic payload of a stub: the named MPI function is outside the carried surface.
///
/// ```
/// let p = mpi_native_sys::UnsupportedMpiCall { name: "Alltoall" };
/// assert_eq!(p.to_string(), "MPI_Alltoall is outside tenferro's native MPI surface");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnsupportedMpiCall {
    /// The function's name without the `MPI_` prefix where it has one.
    pub name: &'static str,
}

impl std::fmt::Display for UnsupportedMpiCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MPI_{} is outside tenferro's native MPI surface", self.name)
    }
}

static STUB_HITS: AtomicUsize = AtomicUsize::new(0);

/// How many stubs have been entered in this process. The scenarios assert this stays zero,
/// so a tenferro path that reaches a stub fails CI instead of a user's session.
pub fn stub_hits() -> usize {
    STUB_HITS.load(Ordering::SeqCst)
}

/// Enter a stub: count it and panic with a typed payload.
pub(crate) fn stub(name: &'static str) -> ! {
    STUB_HITS.fetch_add(1, Ordering::SeqCst);
    let short = name.strip_prefix("MPI_").or_else(|| name.strip_prefix("RSMPI_")).unwrap_or(name);
    std::panic::panic_any(UnsupportedMpiCall { name: short })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stub_panics_with_a_typed_payload_and_is_counted() {
        let before = stub_hits();
        let err = std::panic::catch_unwind(|| unsafe {
            MPI_Abort(0, 1);
        })
        .unwrap_err();
        let hit = err.downcast::<UnsupportedMpiCall>().expect("typed payload");
        assert_eq!(hit.name, "Abort");
        assert_eq!(stub_hits(), before + 1);
    }
}
