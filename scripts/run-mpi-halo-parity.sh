#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

cargo build --quiet --release --manifest-path "$ROOT/Cargo.toml" \
    -p wrf-domain-mpi --example mpi_halo_parity
mpirun --oversubscribe -n 4 "$ROOT/target/release/examples/mpi_halo_parity"
