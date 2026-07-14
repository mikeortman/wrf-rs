#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/dyn_em/module_big_step_utilities_em.F"
driver="$repository_root/parity/momentum-coupling/momentum_coupling_driver.F90"
expected="$repository_root/crates/wrf-dynamics/test-data/momentum_coupling.out.correct"

if ! command -v gfortran >/dev/null 2>&1; then
    echo "missing command required by momentum-coupling oracle: gfortran" >&2
    exit 1
fi
if [ ! -f "$upstream_module" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-momentum-coupling.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

sed -n '/^SUBROUTINE couple_momentum /,/^END SUBROUTINE couple_momentum$/p' \
    "$upstream_module" > "$build_directory/couple_momentum.F90"
gfortran -O0 -ffree-form -ffree-line-length-none \
    "$build_directory/couple_momentum.F90" "$driver" \
    -o "$build_directory/momentum_coupling_driver"
"$build_directory/momentum_coupling_driver" > "$build_directory/actual.out"

if [ "${UPDATE_GOLDEN:-0}" = "1" ]; then
    cp "$build_directory/actual.out" "$expected"
fi
if ! cmp -s "$build_directory/actual.out" "$expected"; then
    diff -u "$expected" "$build_directory/actual.out"
    exit 1
fi

echo "PASS WRF couple_momentum oracle"
