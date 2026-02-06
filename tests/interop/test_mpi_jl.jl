#!/usr/bin/env julia
#
# MPI.jl interop test for MPMD launch with Rust and Python.
#
# This script participates in a cross-language MPI test:
# - All ranks call MPI_Barrier to verify shared MPI_COMM_WORLD
# - Rank 0 broadcasts a value (42) to all ranks
# - All ranks verify they received the broadcast
#
# Run via: tests/interop/run_interop.sh

using MPI

function main()
    MPI.Init()

    comm = MPI.COMM_WORLD
    rank = MPI.Comm_rank(comm)
    size = MPI.Comm_size(comm)

    # Phase 1: All ranks report in and synchronize
    println("[Julia rank $rank] MPI_COMM_WORLD size = $size")
    MPI.Barrier(comm)

    # Phase 2: Rank 0 broadcasts a magic value (42) to all ranks
    value = MPI.bcast(rank == 0 ? Int32(42) : Int32(0), comm; root=0)

    @assert value == 42 "Broadcast value mismatch on rank $rank: got $value"
    println("[Julia rank $rank] broadcast received: $value")

    # Phase 3: Final barrier
    MPI.Barrier(comm)

    MPI.Finalize()
end

main()
