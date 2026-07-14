#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/phys/module_mp_kessler.F"
driver="$repository_root/parity/microphysics-driver/microphysics_driver_kessler.F90"
normalize_nans="$repository_root/scripts/normalize-fortran-single-nans.awk"

if ! command -v gfortran >/dev/null 2>&1; then
    echo "missing command required by microphysics driver oracle: gfortran" >&2
    exit 1
fi
if [ ! -f "$upstream_module" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-mp-driver.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

gfortran -O0 -ffree-form -ffree-line-length-none \
    "$upstream_module" "$driver" -o "$build_directory/microphysics-driver-oracle"
"$build_directory/microphysics-driver-oracle" \
    | awk -f "$normalize_nans" >"$build_directory/fortran.txt"
cargo run --quiet --release -p wrf-physics --example microphysics_driver_oracle \
    | awk -f "$normalize_nans" >"$build_directory/rust.txt"

if ! cmp -s "$build_directory/fortran.txt" "$build_directory/rust.txt"; then
    diff -u "$build_directory/fortran.txt" "$build_directory/rust.txt" | sed -n '1,120p'
    echo "Rust microphysics driver output differs from pinned WRF" >&2
    exit 1
fi

echo "PASS microphysics driver oracle: Rust matches pinned WRF exactly."
