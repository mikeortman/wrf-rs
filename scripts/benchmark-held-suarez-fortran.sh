#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/dyn_em/module_damping_em.F"
driver="$repository_root/parity/held-suarez/held_suarez_damp_benchmark.F90"
error_stub="$repository_root/parity/positive-definite/module_wrf_error_stub.F90"

if ! command -v gfortran >/dev/null 2>&1; then
    echo "missing command required by Held-Suarez benchmark: gfortran" >&2
    exit 1
fi
if [ ! -f "$upstream_module" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-held-suarez-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

gfortran -O3 -flto -c -J "$build_directory" "$error_stub" \
    -o "$build_directory/module_wrf_error_stub.o"
gfortran -O3 -flto -c -cpp -ffree-form -ffree-line-length-none \
    -I "$build_directory" -J "$build_directory" \
    "$upstream_module" -o "$build_directory/module_damping_em.o"
gfortran -O3 -flto -c -ffree-form -ffree-line-length-none \
    -I "$build_directory" -J "$build_directory" \
    "$driver" -o "$build_directory/held_suarez_damp_benchmark.o"
gfortran -O3 -flto \
    "$build_directory/module_wrf_error_stub.o" \
    "$build_directory/module_damping_em.o" \
    "$build_directory/held_suarez_damp_benchmark.o" \
    -o "$build_directory/held_suarez_damp_benchmark"

echo "compiler $(gfortran --version | sed -n '1p')"
echo "flags -O3 -flto (no fast-math or native-CPU flag)"
"$build_directory/held_suarez_damp_benchmark"
