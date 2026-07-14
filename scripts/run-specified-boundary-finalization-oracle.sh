#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/share/module_bc.F"
parity_directory="$repository_root/parity/specified-boundary-finalization"
expected="$repository_root/crates/wrf-dynamics/test-data/specified_boundary_finalization.out.correct"

command -v gfortran >/dev/null 2>&1 || {
    echo "missing command required by specified-boundary finalization oracle: gfortran" >&2
    exit 1
}
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-specified-boundary-finalization.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM
{
    echo 'module extracted_specified_boundary_finalization'
    echo 'use module_configure, only: grid_config_rec_type'
    echo 'implicit none'
    echo 'contains'
    sed -n '/^[[:space:]]*SUBROUTINE spec_bdy_final[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE spec_bdy_final/p' "$upstream_module"
    echo 'end module extracted_specified_boundary_finalization'
} > "$build_directory/extracted.F90"
gfortran -O0 -ffp-contract=off -ffree-form -ffree-line-length-none \
    "$parity_directory/module_configure_stub.F90" "$build_directory/extracted.F90" \
    "$parity_directory/specified_boundary_finalization_driver.F90" \
    -o "$build_directory/oracle"
"$build_directory/oracle" > "$build_directory/actual.out"
if [ "${UPDATE_GOLDEN:-0}" = "1" ]; then
    cp "$build_directory/actual.out" "$expected"
fi
diff -u "$expected" "$build_directory/actual.out"
echo "PASS WRF specified-boundary finalization oracle"
