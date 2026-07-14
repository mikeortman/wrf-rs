#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/dyn_em/module_big_step_utilities_em.F"
driver="$repository_root/parity/inverse-density/inverse_density_driver.F90"
expected="$repository_root/crates/wrf-dynamics/test-data/inverse_density.out.correct"

if ! command -v gfortran >/dev/null 2>&1; then
    echo "missing command required by inverse-density oracle: gfortran" >&2
    exit 1
fi
if [ ! -f "$upstream_module" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-inverse-density.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

sed -n '/^SUBROUTINE calc_alt /,/^END SUBROUTINE calc_alt$/p' \
    "$upstream_module" > "$build_directory/calc_alt.F90"
gfortran -O0 -ffree-form -ffree-line-length-none \
    "$build_directory/calc_alt.F90" "$driver" \
    -o "$build_directory/inverse_density_driver"
"$build_directory/inverse_density_driver" > "$build_directory/actual.out"

if [ "${UPDATE_GOLDEN:-0}" = "1" ]; then
    cp "$build_directory/actual.out" "$expected"
fi
if ! cmp -s "$build_directory/actual.out" "$expected"; then
    diff -u "$expected" "$build_directory/actual.out"
    exit 1
fi

echo "PASS WRF calc_alt oracle"
