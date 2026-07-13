#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/dyn_em/module_positive_definite.F"
parity_directory="$repository_root/parity/positive-definite"
sheet_expected_output="$repository_root/crates/wrf-dynamics/test-data/positive_definite_sheet.out.correct"
slab_expected_output="$repository_root/crates/wrf-dynamics/test-data/positive_definite_slab.out.correct"

if ! command -v gfortran >/dev/null 2>&1; then
    echo "missing command required by positive-definite oracle: gfortran" >&2
    exit 1
fi
if [ ! -f "$upstream_module" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-positive-definite.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

gfortran -c -J "$build_directory" \
    "$parity_directory/module_wrf_error_stub.F90" \
    -o "$build_directory/module_wrf_error_stub.o"
gfortran -c -cpp -ffree-form -ffree-line-length-none \
    -I "$build_directory" -J "$build_directory" \
    "$upstream_module" \
    -o "$build_directory/module_positive_definite.o"
for routine in sheet slab; do
    gfortran -c -ffree-form -ffree-line-length-none \
        -I "$build_directory" -J "$build_directory" \
        "$parity_directory/positive_definite_${routine}_driver.F90" \
        -o "$build_directory/positive_definite_${routine}_driver.o"
    gfortran \
        "$build_directory/module_wrf_error_stub.o" \
        "$build_directory/module_positive_definite.o" \
        "$build_directory/positive_definite_${routine}_driver.o" \
        -o "$build_directory/positive_definite_${routine}_driver"
    "$build_directory/positive_definite_${routine}_driver" \
        > "$build_directory/${routine}_actual.out"
done

diff -u "$sheet_expected_output" "$build_directory/sheet_actual.out"
echo "PASS WRF positive_definite_sheet oracle"
diff -u "$slab_expected_output" "$build_directory/slab_actual.out"
echo "PASS WRF positive_definite_slab oracle"
