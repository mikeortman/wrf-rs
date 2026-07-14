#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
WRF_TOOLS="$ROOT/upstream/WRF/tools"
FIXTURE="$ROOT/parity/registry/fixtures/registry_arw_slice"
GOLDEN="$ROOT/parity/registry/golden"
WORK=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-registry.XXXXXX")
trap 'rm -rf "$WORK"' EXIT

make -C "$WRF_TOOLS" registry >/dev/null
mkdir -p "$WORK/wrf/inc" "$WORK/wrf/frame" "$WORK/wrf/Registry" "$WORK/rust"
cp "$FIXTURE" "$WORK/wrf/Registry/Registry.slice"

(
    cd "$WORK/wrf"
    "$WRF_TOOLS/registry" Registry/Registry.slice >/dev/null
)

cargo run --quiet --release --manifest-path "$ROOT/Cargo.toml" \
    -p wrf-registry --bin wrf-registry-generate -- \
    "$FIXTURE" "$WORK/rust"

for artifact in \
    state_struct.inc \
    namelist_defines.inc \
    namelist_defaults.inc \
    namelist_statements.inc \
    model_data_order.inc
do
    diff -u "$GOLDEN/$artifact" "$WORK/wrf/inc/$artifact"
    diff -u "$GOLDEN/$artifact" "$WORK/rust/$artifact"
done

find "$WORK/wrf/inc" -name 'allocs_*.F' -print0 \
    | sort -zV \
    | xargs -0 awk '
        /tail_statevars%VarName = '\''/ {
            value=$0; sub(/^.*= '\''/, "", value); sub(/'\''$/, "", value); var_name=value
        }
        /tail_statevars%DataName = '\''/ {
            value=$0; sub(/^.*= '\''/, "", value); sub(/'\''$/, "", value); data_name=value
        }
        /tail_statevars%Description = '\''/ {
            value=$0; sub(/^.*= '\''/, "", value); sub(/'\''$/, "", value); description=value
        }
        /tail_statevars%Units = '\''/ {
            value=$0; sub(/^.*= '\''/, "", value); sub(/'\''$/, "", value); units=value
        }
        /tail_statevars%MemoryOrder  = '\''/ {
            value=$0; sub(/^.*= '\''/, "", value); sub(/'\''$/, "", value); memory_order=value
        }
        /tail_statevars%Ntl     = / {
            value=$0; sub(/^.*= /, "", value); gsub(/[[:space:]]/, "", value); time_level=value
        }
        /tail_statevars%Ndim    = / {
            value=$0; sub(/^.*= /, "", value); gsub(/[[:space:]]/, "", value)
            printf "VarName=%s|DataName=%s|Description=%s|Units=%s|MemoryOrder=%s|Ntl=%s|Ndim=%s\n", var_name, data_name, description, units, memory_order, time_level, value
        }
    ' >"$WORK/wrf_state_metadata.txt"

diff -u "$GOLDEN/state_metadata.txt" "$WORK/wrf_state_metadata.txt"
diff -u "$GOLDEN/state_metadata.txt" "$WORK/rust/state_metadata.txt"

echo "WRF Registry fixture, generated includes, and metadata projection match exactly."
