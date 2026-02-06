#!/bin/bash
# Cross-language MPI interoperability test.
#
# Launches Rust, Python, and Julia MPI programs under a single mpiexec
# using MPMD (Multiple Program Multiple Data) mode, sharing MPI_COMM_WORLD.
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
check_cmd() {
    if ! command -v "$1" &>/dev/null; then
        echo "SKIP: $1 not found"
        return 1
    fi
    return 0
}

HAS_PYTHON=false
HAS_JULIA=false

if check_cmd python3; then
    if python3 -c "import mpi4py" 2>/dev/null; then
        HAS_PYTHON=true
    else
        echo "SKIP: mpi4py not installed (pip install mpi4py)"
    fi
fi

if check_cmd julia; then
    if julia -e 'using MPI' 2>/dev/null; then
        HAS_JULIA=true
    else
        echo "SKIP: MPI.jl not installed (julia -e 'using Pkg; Pkg.add(\"MPI\")')"
    fi
fi

# --- Test 1: Rust only (baseline) ---
echo ""
echo "--- Test 1: Rust-only MPI (2 ranks) ---"
mpiexec -n 2 "$RUST_BIN"
echo "PASSED"

# --- Test 2: Rust + Python (MPMD) ---
if [ "$HAS_PYTHON" = true ]; then
    echo ""
    echo "--- Test 2: Rust + Python MPMD (2 ranks) ---"
    mpiexec -n 1 "$RUST_BIN" : -n 1 python3 "$SCRIPT_DIR/test_mpi4py.py"
    echo "PASSED"
else
    echo ""
    echo "--- Test 2: Rust + Python MPMD --- SKIPPED"
fi

# --- Test 3: Rust + Julia (MPMD) ---
if [ "$HAS_JULIA" = true ]; then
    echo ""
    echo "--- Test 3: Rust + Julia MPMD (2 ranks) ---"
    mpiexec -n 1 "$RUST_BIN" : -n 1 julia "$SCRIPT_DIR/test_mpi_jl.jl"
    echo "PASSED"
else
    echo ""
    echo "--- Test 3: Rust + Julia MPMD --- SKIPPED"
fi

# --- Test 4: All three languages (MPMD) ---
if [ "$HAS_PYTHON" = true ] && [ "$HAS_JULIA" = true ]; then
    echo ""
    echo "--- Test 4: Rust + Python + Julia MPMD (3 ranks) ---"
    mpiexec -n 1 "$RUST_BIN" : -n 1 python3 "$SCRIPT_DIR/test_mpi4py.py" : -n 1 julia "$SCRIPT_DIR/test_mpi_jl.jl"
    echo "PASSED"
else
    echo ""
    echo "--- Test 4: Rust + Python + Julia MPMD --- SKIPPED"
fi

echo ""
echo "=== All interop tests completed ==="
