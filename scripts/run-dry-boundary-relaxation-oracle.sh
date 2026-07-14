#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
shared_module="$repository_root/upstream/WRF/share/module_bc.F"
em_module="$repository_root/upstream/WRF/dyn_em/module_bc_em.F"
parity_directory="$repository_root/parity/dry-boundary-relaxation"
expected="$repository_root/crates/wrf-dynamics/test-data/dry_boundary_relaxation.out.correct"
normalizer="$repository_root/scripts/normalize-fortran-single-nans.awk"

command -v gfortran >/dev/null 2>&1 || {
    echo "missing command required by dry boundary-relaxation oracle: gfortran" >&2
    exit 1
}
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-dry-boundary-relaxation.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM
{
    echo 'module extracted_specified_boundary_relaxation'
    echo 'use module_configure, only: grid_config_rec_type'
    echo 'implicit none'
    echo 'contains'
    sed -n '/^[[:space:]]*SUBROUTINE relax_bdytend[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE relax_bdytend_core/p' "$shared_module"
    echo 'end module extracted_specified_boundary_relaxation'
} > "$build_directory/extracted_shared.F90"
{
    echo 'module extracted_dry_boundary_relaxation'
    echo 'use module_configure, only: grid_config_rec_type'
    echo 'use extracted_specified_boundary_relaxation, only: relax_bdytend, relax_bdytend_tile'
    echo 'implicit none'
    echo 'contains'
    sed -n '/^[[:space:]]*SUBROUTINE relax_bdy_dry[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE relax_bdy_dry/p' "$em_module"
    sed -n '/^[[:space:]]*SUBROUTINE mass_weight[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE mass_weight/p' "$em_module"
    echo 'end module extracted_dry_boundary_relaxation'
} > "$build_directory/extracted_em.F90"
gfortran -O0 -ffp-contract=off -ffree-form -ffree-line-length-none \
    "$parity_directory/module_configure_stub.F90" \
    "$build_directory/extracted_shared.F90" "$build_directory/extracted_em.F90" \
    "$parity_directory/dry_boundary_relaxation_driver.F90" \
    -o "$build_directory/oracle"
"$build_directory/oracle" | LC_ALL=C awk -f "$normalizer" > "$build_directory/actual.out"
if [ "${UPDATE_GOLDEN:-0}" = "1" ]; then
    cp "$build_directory/actual.out" "$expected"
fi
diff -u "$expected" "$build_directory/actual.out"
echo "PASS WRF relax_bdy_dry oracle"
