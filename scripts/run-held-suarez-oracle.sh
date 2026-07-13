#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/dyn_em/module_damping_em.F"
driver="$repository_root/parity/held-suarez/held_suarez_damp_driver.F90"
error_stub="$repository_root/parity/positive-definite/module_wrf_error_stub.F90"
expected_output="$repository_root/crates/wrf-dynamics/test-data/held_suarez_damp.out.correct"

if ! command -v gfortran >/dev/null 2>&1; then
    echo "missing command required by Held-Suarez oracle: gfortran" >&2
    exit 1
fi
if [ ! -f "$upstream_module" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-held-suarez.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

gfortran -c -J "$build_directory" "$error_stub" \
    -o "$build_directory/module_wrf_error_stub.o"
gfortran -c -cpp -ffree-form -ffree-line-length-none \
    -I "$build_directory" -J "$build_directory" \
    "$upstream_module" -o "$build_directory/module_damping_em.o"
gfortran -c -ffree-form -ffree-line-length-none \
    -I "$build_directory" -J "$build_directory" \
    "$driver" -o "$build_directory/held_suarez_damp_driver.o"
gfortran \
    "$build_directory/module_wrf_error_stub.o" \
    "$build_directory/module_damping_em.o" \
    "$build_directory/held_suarez_damp_driver.o" \
    -o "$build_directory/held_suarez_damp_driver"

"$build_directory/held_suarez_damp_driver" > "$build_directory/actual.out"
if [ ! -f "$expected_output" ]; then
    echo "Held-Suarez golden output is not initialized:" >&2
    cat "$build_directory/actual.out"
    exit 2
fi
diff -u "$expected_output" "$build_directory/actual.out"
echo "PASS WRF held_suarez_damp oracle"
