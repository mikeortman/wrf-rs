#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/dyn_em/module_big_step_utilities_em.F"
driver="$repository_root/parity/moisture-coefficients/moisture_coefficient_benchmark.F90"

if ! command -v gfortran >/dev/null 2>&1; then
    echo "missing command required by moisture-coefficient benchmark: gfortran" >&2
    exit 1
fi
if [ ! -f "$upstream_module" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-moisture-coefficient-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

sed -n '/^SUBROUTINE calc_cq /,/^END SUBROUTINE calc_cq$/p' \
    "$upstream_module" > "$build_directory/calc_cq.F90"
gfortran -O3 -flto -cpp -DPARAM_FIRST_SCALAR=2 \
    -ffree-form -ffree-line-length-none \
    "$build_directory/calc_cq.F90" "$driver" \
    -o "$build_directory/moisture_coefficient_benchmark"

echo "compiler $(gfortran --version | sed -n '1p')"
echo "flags -O3 -flto (no fast-math or explicit native-CPU flag)"
"$build_directory/moisture_coefficient_benchmark"
