#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/dyn_em/module_bc_em.F"
parity_directory="$repository_root/parity/specified-boundary-geopotential"
expected="$repository_root/crates/wrf-dynamics/test-data/specified_boundary_geopotential.out.correct"
normalizer="$repository_root/scripts/normalize-fortran-single-nans.awk"

command -v gfortran >/dev/null 2>&1 || {
    echo "missing command required by specified-boundary geopotential oracle: gfortran" >&2
    exit 1
}
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-specified-geopotential.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM
{
    echo 'module extracted_specified_boundary_geopotential'
    echo 'use module_configure, only: grid_config_rec_type'
    echo 'implicit none'
    echo 'contains'
    sed -n '/^[[:space:]]*SUBROUTINE spec_bdyupdate_ph(/,/^[[:space:]]*END SUBROUTINE spec_bdyupdate_ph/p' "$upstream_module"
    echo 'end module extracted_specified_boundary_geopotential'
} > "$build_directory/extracted.F90"
gfortran -O0 -ffp-contract=off -ffree-form -ffree-line-length-none \
    "$parity_directory/module_configure_stub.F90" "$build_directory/extracted.F90" \
    "$parity_directory/specified_boundary_geopotential_driver.F90" \
    -o "$build_directory/oracle"
"$build_directory/oracle" > "$build_directory/actual.raw"
awk -f "$normalizer" "$build_directory/actual.raw" > "$build_directory/actual.out"
if [ "${UPDATE_GOLDEN:-0}" = "1" ]; then
    cp "$build_directory/actual.out" "$expected"
fi
awk -f "$normalizer" "$expected" > "$build_directory/expected.out"
diff -u "$build_directory/expected.out" "$build_directory/actual.out"
echo "PASS WRF spec_bdyupdate_ph oracle"
