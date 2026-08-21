# MPIABI data-symbol loading fix

## Problem

`mpi-rt-sys` currently generates `libloading::Library::get::<T>` for each
`MPIABI_*` constant. MPIwrapper exports these constants as C data symbols. A
`libloading::Symbol<T>` models the address returned by `dlsym` as `T`; it does
not read the value stored at a data symbol.

For a non-pointer value such as `MPIABI_ANY_SOURCE`, `T = c_int` is rejected by
`libloading` because `c_int` is not pointer-sized (`IncompatibleSize`). For
pointer-sized handles and pointer/callback constants, the same code can appear
to load successfully while returning the data symbol's address instead of its
stored value.

This affects every generated MPIABI constant, not only the first constant that
panics. Generated files must not be edited by hand.

After correcting data-symbol loading, a real two-rank receive exposed two
related MPIABI defects. First, on 64-bit targets the C `MPIABI_Status` internal
union has 8-byte alignment and occupies 24 bytes, making the complete struct 40
bytes. Rust currently represents that union as `[u8; 24]`, leaving the struct
with 4-byte alignment and size 36. The field offsets happen to match for one
value, but arrays, stack allocation, and C ABI calls use the wrong stride and
alignment. The 32-bit C status is 32 bytes with 4-byte alignment.

Second, MPIwrapper exports ABI-adapting functions as `MPIABI_*` data-independent
code symbols (for example `MPIABI_Iprobe` and `MPIABI_Comm_dup`). The generator
currently resolves the native names `MPI_Iprobe` and `MPI_Comm_dup`. Because
MPIwrapper itself has undefined references to those native functions, dynamic
lookup can resolve them from the linked native MPI library and bypass every
MPIABI conversion. This passes MPIABI integer handles and status layouts
straight to an implementation-specific ABI.

## Design

1. Add one crate-private unsafe helper to `mpi-rt-sys/src/loader.rs`:
   `get_constant<T: Copy>(name: &[u8]) -> T`.
2. Load each data symbol as `libloading::Symbol<*const T>` and read the value at
   that address exactly once. `T: Copy` expresses that the generated scalar,
   handle, pointer, and optional callback-pointer representations are copied
   out of immutable process-lifetime MPIwrapper storage.
3. Preserve the current symbol-specific panic diagnostics.
4. Change `mpi-rt-sys/gen/gen_rust.py` to emit calls to this helper for every
   `MPIABI_*` constant and regenerate `mpi-rt-sys/src/constants.rs`.
5. Change the same generator to resolve every function through its MPIwrapper
   ABI-adapting name: standard `MPI_Foo` becomes `MPIABI_Foo`; the four current
   extension entries `MPIX_Query_*` become `MPIXABI_Query_*`. Reject any other
   prefix instead of silently retaining an unadapted symbol name. Keep the
   public Rust wrapper function names and FFI signatures unchanged. Regenerate
   `mpi-rt-sys/src/functions.rs`; do not modify `loader::get_symbol` semantics.
6. Correct the private `MPI_Status` storage representation in
   `mpi-rt-sys/src/types.rs`: use pointer-aligned 24-byte storage on 64-bit
   targets (`[usize; 3]`) and 4-byte-aligned 20-byte storage on 32-bit targets
   (`[u32; 5]`). Keep the three public MPI fields and `repr(C)`. Add compile-time
   size/alignment/field-offset assertions for each pointer width: 40 bytes,
   alignment 8, fields at 24/28/32 on 64-bit; 32 bytes, alignment 4, fields at
   20/24/28 on 32-bit. Replace the stale 36-byte assertion and use stable
   `std::mem::offset_of!` (available before the crate's Rust 1.78 MSRV) for all
   compile-time field-offset assertions. Add an explicit compile error for
   unsupported pointer widths and a runtime unit test for two-element array
   stride.
7. Do not change public APIs, MPI handle mappings, environment-variable
   behavior, backend selection, or dependency features.

## Safety contract

MPIwrapper defines every `MPIABI_*` export in the generator's constant table as
a data object whose C type matches the corresponding generated Rust FFI type.
The shared library remains loaded in a process-global `OnceLock<Library>`, so
the symbol storage remains valid. Reading through `*const T` is valid only
under that ABI/type contract; the generated table and helper invocation make
that assumption explicit. Values are copied into the existing
`OnceLock<MpiConstants>` and are then immutable.

Function exports are not dereferenced as data. They continue to use
`get_symbol`, which returns their callable address, but the resolved name must
be the `MPIABI_*` adapter export rather than the adapter's `MPI_*` native
library dependency.

`MPI_Status` is passed by pointer to MPIABI functions. Its internal storage must
therefore reproduce both the C union's size and alignment, not only the visible
field offsets. The chosen integer arrays are opaque storage; their values are
never interpreted by Rust.

## Verification

- Regenerate bindings and verify a second generator run produces no diff.
- Assert generated `constants.rs` contains no direct `lib.get::<...>` constant
  loads and routes all generated constants through `loader::get_constant`.
- Assert every generated function resolves an `MPIABI_*` or `MPIXABI_*`
  adapter symbol and no generated lookup requests an unprefixed native
  `MPI_*`/`MPIX_*` symbol. Confirm every generated lookup exists in the
  MPIwrapper dynamic export table.
- `cargo fmt --all --check`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` for
  supported feature combinations (mutually exclusive backends checked
  separately where required).
- `cargo test --workspace` with the normal system-MPI backend.
- Build the `mpi-rt-sys-backend` without build-time MPI discovery.
- Test `MPI_Status` size, alignment, visible-field offsets, and two-element
  array stride on the host; cross-compile-check the 32-bit representation when
  that target is available.
- Against an MPIwrapper fixture with a recorded commit, run runtime examples
  under `mpiexec` and specifically exercise integer constants, communicator
  handles, datatype handles, operation handles, status/pointer constants, and
  callback constants where the safe rsmpi API exposes them.
- Re-run Hataori's `rsmpi-rt` n=1,2,4 pmap smoke; this is the downstream
  acceptance that originally exposed the bug.

## Non-goals

- No fallback to a linked system MPI library.
- No MPIwrapper ABI adaptation or version detection.
- No lazy per-constant redesign.
- No handwritten generated-source patch.
- No redesign of MPI status access or wrapper conversion.
- No direct calls to native MPI symbols from the runtime backend.
- No changes to Hataori's scheduler or protocol.

## Review record

- Pre-implementation review: `reviewer-flash-opencode-go` (read-only, high),
  including follow-up rounds for the status layout, adapter function prefixes,
  and `MPIXABI_*` naming; final verdict **Correct-to-merge**.
- Implementation: Luna (high), integrated and independently verified by the
  parent agent.
- Post-implementation full-diff review: `reviewer-flash-opencode-go`
  (read-only, high); final verdict **Correct-to-merge**.
- Runtime fixture: MPIwrapper commit
  `966f4231c96153a08295fc7d0bcbd65e916a73fd`.
