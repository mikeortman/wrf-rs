#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
WORK=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-tiles.XXXXXX")
trap 'rm -rf "$WORK"' EXIT

gfortran -cpp -ffree-form -O2 -J"$WORK" -c \
    "$ROOT/parity/domain/module_driver_constants_stub.F90" -o "$WORK/constants.o"
gfortran -cpp -ffree-form -O2 -DIWORDSIZE=4 -DRWORDSIZE=4 \
    -DDWORDSIZE=8 -DLWORDSIZE=4 -J"$WORK" -I"$WORK" -c \
    "$ROOT/upstream/WRF/frame/module_machine.F" -o "$WORK/module_machine.o"
gfortran -cpp -ffree-form -O2 -J"$WORK" -I"$WORK" -c \
    "$ROOT/parity/domain/clipped_tiles_oracle.F90" -o "$WORK/driver.o"
gfortran "$WORK/module_machine.o" "$WORK/driver.o" -o "$WORK/wrf-clipped-tiles"

"$WORK/wrf-clipped-tiles" >"$WORK/wrf.txt"
cargo run --quiet --release --manifest-path "$ROOT/Cargo.toml" \
    -p wrf-domain --example clipped_tiles_oracle >"$WORK/rust.txt"

diff -u "$WORK/wrf.txt" "$WORK/rust.txt"
echo "Rust clipped tile bounds match WRF region_bounds and set_tiles2 semantics."
