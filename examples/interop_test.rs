//! Rust MPI interop test program for MPMD launch with Python/Julia.
//!
//! This program participates in a cross-language MPI test:
//! - All ranks call MPI_Barrier to verify shared MPI_COMM_WORLD
//! - Rank 0 (this program) broadcasts a value to all ranks
//! - All ranks verify they received the broadcast
//!
//! Run via: tests/interop/run_interop.sh

use mpi::traits::*;

fn main() {
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let rank = world.rank();
    let size = world.size();

    // Phase 1: All ranks report in and synchronize
    println!("[Rust rank {}] MPI_COMM_WORLD size = {}", rank, size);
    world.barrier();

    // Phase 2: Rank 0 broadcasts a magic value (42) to all ranks
    let root = world.process_at_rank(0);
    let mut value: i32 = if rank == 0 { 42 } else { 0 };
    root.broadcast_into(&mut value);

    assert_eq!(value, 42, "Broadcast value mismatch on rank {}", rank);
    println!("[Rust rank {}] broadcast received: {}", rank, value);

    // Phase 3: Final barrier
    world.barrier();
    if rank == 0 {
        println!("[Rust rank 0] interop test PASSED");
    }
}
