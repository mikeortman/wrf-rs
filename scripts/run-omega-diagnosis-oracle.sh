#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/dyn_em/module_big_step_utilities_em.F"
driver="$repository_root/parity/omega-diagnosis/omega_diagnosis_driver.F90"
expected="$repository_root/crates/wrf-dynamics/test-data/omega_diagnosis.out.correct"

if ! command -v gfortran >/dev/null 2>&1; then
    echo "missing command required by omega-diagnosis oracle: gfortran" >&2
    exit 1
fi
if [ ! -f "$upstream_module" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-omega-diagnosis.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

sed -n '/^SUBROUTINE calc_ww_cp /,/^END SUBROUTINE calc_ww_cp$/p' \
    "$upstream_module" > "$build_directory/calc_ww_cp.F90"
gfortran -O0 -ffree-form -ffree-line-length-none \
    "$build_directory/calc_ww_cp.F90" "$driver" \
    -o "$build_directory/omega_diagnosis_driver"
"$build_directory/omega_diagnosis_driver" > "$build_directory/actual.out"

if [ "${UPDATE_GOLDEN:-0}" = "1" ]; then
    cp "$build_directory/actual.out" "$expected"
fi
if ! cmp -s "$build_directory/actual.out" "$expected"; then
    diff -u "$expected" "$build_directory/actual.out"
    exit 1
fi

echo "PASS WRF calc_ww_cp oracle"
