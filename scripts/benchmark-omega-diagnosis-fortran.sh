#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/dyn_em/module_big_step_utilities_em.F"
driver="$repository_root/parity/omega-diagnosis/omega_diagnosis_benchmark.F90"

if ! command -v gfortran >/dev/null 2>&1; then
    echo "missing command required by omega-diagnosis benchmark: gfortran" >&2
    exit 1
fi
if [ ! -f "$upstream_module" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-omega-diagnosis-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

sed -n '/^SUBROUTINE calc_ww_cp /,/^END SUBROUTINE calc_ww_cp$/p' \
    "$upstream_module" > "$build_directory/calc_ww_cp.F90"
gfortran -O3 -flto -ffree-form -ffree-line-length-none \
    "$build_directory/calc_ww_cp.F90" "$driver" \
    -o "$build_directory/omega_diagnosis_benchmark"

echo "compiler $(gfortran --version | sed -n '1p')"
echo "flags -O3 -flto (no fast-math or explicit native-CPU flag)"
"$build_directory/omega_diagnosis_benchmark"
