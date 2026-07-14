#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
RSL_LITE="$ROOT/upstream/WRF/external/RSL_LITE"
WORK=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-periodic.XXXXXX")
trap 'rm -rf "$WORK"' EXIT

PLATFORM_DEFINE=()
if [[ "$(uname -s)" == "Darwin" ]]; then
    PLATFORM_DEFINE=(-DMACOS)
fi

mpicc -std=c11 -O2 -Wno-return-type "${PLATFORM_DEFINE[@]}" \
    -I"$RSL_LITE" -c "$RSL_LITE/period.c" -o "$WORK/period.o"
mpicc -std=c11 -O2 -Wno-return-type "${PLATFORM_DEFINE[@]}" \
    -I"$RSL_LITE" -c "$RSL_LITE/buf_for_proc.c" -o "$WORK/buf_for_proc.o"
mpicc -std=c11 -O2 -Wno-return-type "${PLATFORM_DEFINE[@]}" \
    -I"$RSL_LITE" -c "$ROOT/parity/domain/periodic_halo_oracle.c" \
    -o "$WORK/oracle.o"
mpifort -cpp -O2 -J"$WORK" -c "$RSL_LITE/f_pack.F90" -o "$WORK/f_pack.o"
mpifort "$WORK/period.o" "$WORK/buf_for_proc.o" "$WORK/oracle.o" \
    "$WORK/f_pack.o" -o "$WORK/wrf-periodic-halo"

mpirun -n 4 "$WORK/wrf-periodic-halo" >"$WORK/wrf.txt"
cargo run --quiet --release --manifest-path "$ROOT/Cargo.toml" \
    -p wrf-domain --example periodic_halo_oracle >"$WORK/rust.txt"

diff -u "$WORK/wrf.txt" "$WORK/rust.txt"
echo "Rust periodic halo destinations match WRF period.c exactly."
