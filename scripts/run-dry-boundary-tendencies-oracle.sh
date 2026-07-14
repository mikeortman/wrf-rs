#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
shared_module="$repository_root/upstream/WRF/share/module_bc.F"
em_module="$repository_root/upstream/WRF/dyn_em/module_bc_em.F"
parity_directory="$repository_root/parity/dry-boundary-tendencies"
expected="$repository_root/crates/wrf-dynamics/test-data/dry_boundary_tendencies.out.correct"

command -v gfortran >/dev/null 2>&1 || {
    echo "missing command required by dry boundary-tendency oracle: gfortran" >&2
    exit 1
}
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-dry-boundary-tendencies.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM
{
    echo 'module extracted_specified_boundary_tendencies'
    echo 'use module_configure, only: grid_config_rec_type'
    echo 'implicit none'
    echo 'contains'
    sed -n '/^[[:space:]]*SUBROUTINE spec_bdytend[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE spec_bdytend/p' "$shared_module"
    echo 'end module extracted_specified_boundary_tendencies'
} > "$build_directory/extracted_shared.F90"
{
    echo 'module extracted_dry_boundary_tendencies'
    echo 'use module_configure, only: grid_config_rec_type'
    echo 'use extracted_specified_boundary_tendencies, only: spec_bdytend'
    echo 'implicit none'
    echo 'contains'
    sed -n '/^[[:space:]]*SUBROUTINE spec_bdy_dry[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE spec_bdy_dry/p' "$em_module"
    echo 'end module extracted_dry_boundary_tendencies'
} > "$build_directory/extracted_em.F90"
gfortran -O0 -ffp-contract=off -ffree-form -ffree-line-length-none \
    "$parity_directory/module_configure_stub.F90" \
    "$build_directory/extracted_shared.F90" "$build_directory/extracted_em.F90" \
    "$parity_directory/dry_boundary_tendencies_driver.F90" \
    -o "$build_directory/oracle"
"$build_directory/oracle" > "$build_directory/actual.out"
if [ "${UPDATE_GOLDEN:-0}" = "1" ]; then
    cp "$build_directory/actual.out" "$expected"
fi
diff -u "$expected" "$build_directory/actual.out"
echo "PASS WRF spec_bdy_dry oracle"
