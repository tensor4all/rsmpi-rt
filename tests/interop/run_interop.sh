#!/bin/bash
# Cross-language MPI interoperability test.
#
# Verifies that Rust, Python (mpi4py), and Julia (MPI.jl) all work
# correctly with the same MPI implementation by running each language's
# MPI program under mpiexec -n 2.
#
# Prerequisites:
#   - MPI implementation (MPICH or OpenMPI)
#   - Python 3 with mpi4py: pip install mpi4py
#   - Julia with MPI.jl: julia -e 'using Pkg; Pkg.add("MPI")'
#   - For mpi-rt-sys-backend: MPI_RT_LIB environment variable set
#
# Usage:
#   bash tests/interop/run_interop.sh [--backend mpi-sys-backend|mpi-rt-sys-backend]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

BACKEND="mpi-sys-backend"
while [[ $# -gt 0 ]]; do
    case $1 in
        --backend) BACKEND="$2"; shift 2 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

echo "=== Cross-language MPI interop test (backend: $BACKEND) ==="

# Build Rust test binary
echo "Building Rust test binary..."
FEATURES="$BACKEND"
if [ "$BACKEND" = "mpi-rt-sys-backend" ]; then
    DEFAULT_FEATURES="--no-default-features"
else
    DEFAULT_FEATURES=""
fi
cargo build --manifest-path "$PROJECT_DIR/Cargo.toml" \
    $DEFAULT_FEATURES --features "$FEATURES" \
    --example interop_test 2>&1

RUST_BIN="$PROJECT_DIR/target/debug/examples/interop_test"

# Check prerequisites
HAS_PYTHON=false
HAS_JULIA=false

if command -v python3 &>/dev/null && python3 -c "import mpi4py" 2>/dev/null; then
    HAS_PYTHON=true
else
    echo "NOTE: mpi4py not available, Python tests will be skipped"
fi

if command -v julia &>/dev/null && julia -e 'using MPI' 2>/dev/null; then
    HAS_JULIA=true
else
    echo "NOTE: MPI.jl not available, Julia tests will be skipped"
fi

FAILED=0

# --- Test 1: Rust MPI (2 ranks) ---
echo ""
echo "--- Test 1: Rust MPI (2 ranks) ---"
if mpiexec -n 2 "$RUST_BIN"; then
    echo "PASSED"
else
    echo "FAILED"
    FAILED=1
fi

# --- Test 2: Python/mpi4py (2 ranks) ---
if [ "$HAS_PYTHON" = true ]; then
    echo ""
    echo "--- Test 2: Python/mpi4py (2 ranks) ---"
    if mpiexec -n 2 python3 "$SCRIPT_DIR/test_mpi4py.py"; then
        echo "PASSED"
    else
        echo "FAILED"
        FAILED=1
    fi
else
    echo ""
    echo "--- Test 2: Python/mpi4py --- SKIPPED"
fi

# --- Test 3: Julia/MPI.jl (2 ranks) ---
if [ "$HAS_JULIA" = true ]; then
    echo ""
    echo "--- Test 3: Julia/MPI.jl (2 ranks) ---"
    if timeout 300 mpiexec -n 2 julia --project="$SCRIPT_DIR" "$SCRIPT_DIR/test_mpi_jl.jl"; then
        echo "PASSED"
    else
        echo "FAILED (or timed out after 300s)"
        FAILED=1
    fi
else
    echo ""
    echo "--- Test 3: Julia/MPI.jl --- SKIPPED"
fi

echo ""
if [ "$FAILED" -eq 0 ]; then
    echo "=== All interop tests completed ==="
else
    echo "=== Some interop tests FAILED ==="
    exit 1
fi
