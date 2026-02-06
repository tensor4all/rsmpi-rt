#!/usr/bin/env python3
"""mpi4py interop test for MPMD launch with Rust and Julia.

This script participates in a cross-language MPI test:
- All ranks call MPI_Barrier to verify shared MPI_COMM_WORLD
- Rank 0 broadcasts a value (42) to all ranks
- All ranks verify they received the broadcast

Run via: tests/interop/run_interop.sh
"""

from mpi4py import MPI


def main():
    comm = MPI.COMM_WORLD
    rank = comm.Get_rank()
    size = comm.Get_size()

    # Phase 1: All ranks report in and synchronize
    print(f"[Python rank {rank}] MPI_COMM_WORLD size = {size}")
    comm.Barrier()

    # Phase 2: Rank 0 broadcasts a magic value (42) to all ranks
    value = comm.bcast(42 if rank == 0 else None, root=0)

    assert value == 42, f"Broadcast value mismatch on rank {rank}: got {value}"
    print(f"[Python rank {rank}] broadcast received: {value}")

    # Phase 3: Final barrier
    comm.Barrier()


if __name__ == "__main__":
    main()
