#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
RSL_LITE="$ROOT/upstream/WRF/external/RSL_LITE"
WORK=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-domain.XXXXXX")
trap 'rm -rf "$WORK"' EXIT

cc -std=c11 -O2 -Wno-format -Wno-return-type -DNOUNDERSCORE -DWRFPLUS=0 \
    -I"$RSL_LITE" \
    "$RSL_LITE/task_for_point.c" \
    "$ROOT/parity/domain/domain_topology_oracle.c" \
    -o "$WORK/wrf-domain-topology"

"$WORK/wrf-domain-topology" >"$WORK/wrf.txt"
cargo run --quiet --release --manifest-path "$ROOT/Cargo.toml" \
    -p wrf-domain --example domain_topology_oracle >"$WORK/rust.txt"

diff -u "$WORK/wrf.txt" "$WORK/rust.txt"
echo "Rust domain decomposition matches WRF task_for_point exactly."
