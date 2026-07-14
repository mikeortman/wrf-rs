#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/dyn_em/module_big_step_utilities_em.F"
driver="$repository_root/parity/inverse-density/inverse_density_benchmark.F90"

if ! command -v gfortran >/dev/null 2>&1; then
    echo "missing command required by inverse-density benchmark: gfortran" >&2
    exit 1
fi
if [ ! -f "$upstream_module" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-inverse-density-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

sed -n '/^SUBROUTINE calc_alt /,/^END SUBROUTINE calc_alt$/p' \
    "$upstream_module" > "$build_directory/calc_alt.F90"
gfortran -O3 -flto -ffree-form -ffree-line-length-none \
    "$build_directory/calc_alt.F90" "$driver" \
    -o "$build_directory/inverse_density_benchmark"

echo "compiler $(gfortran --version | sed -n '1p')"
echo "flags -O3 -flto (no fast-math or explicit native-CPU flag)"
"$build_directory/inverse_density_benchmark"
