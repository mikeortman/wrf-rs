#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
shared_module="$repository_root/upstream/WRF/share/module_bc.F"
em_boundary_module="$repository_root/upstream/WRF/dyn_em/module_bc_em.F"
em_module="$repository_root/upstream/WRF/dyn_em/module_em.F"
parity_directory="$repository_root/parity/dry-large-step-boundary-stage"
expected="$repository_root/crates/wrf-dynamics/test-data/dry_large_step_boundary_stage.out.correct"
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-dry-large-step.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

command -v gfortran >/dev/null 2>&1 || { echo "missing gfortran" >&2; exit 1; }
sed -n '/^SUBROUTINE rk_addtend_dry /,/^END SUBROUTINE rk_addtend_dry/p' "$em_module" > "$build_directory/rk_addtend_dry.F90"
{
  echo 'module extracted_specified_boundary_relaxation'
  echo 'use module_configure, only: grid_config_rec_type'
  echo 'implicit none';echo 'contains'
  sed -n '/^[[:space:]]*SUBROUTINE relax_bdytend[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE relax_bdytend_core/p' "$shared_module"
  echo 'end module extracted_specified_boundary_relaxation'
} > "$build_directory/extracted_relaxation_shared.F90"
{
  echo 'module extracted_specified_boundary_tendencies'
  echo 'use module_configure, only: grid_config_rec_type'
  echo 'implicit none';echo 'contains'
  sed -n '/^[[:space:]]*SUBROUTINE spec_bdytend[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE spec_bdytend/p' "$shared_module"
  echo 'end module extracted_specified_boundary_tendencies'
} > "$build_directory/extracted_tendencies_shared.F90"
{
  echo 'module extracted_dry_boundary_relaxation'
  echo 'use module_configure, only: grid_config_rec_type'
  echo 'use extracted_specified_boundary_relaxation, only: relax_bdytend, relax_bdytend_tile'
  echo 'implicit none';echo 'contains'
  sed -n '/^[[:space:]]*SUBROUTINE relax_bdy_dry[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE relax_bdy_dry/p' "$em_boundary_module"
  sed -n '/^[[:space:]]*SUBROUTINE mass_weight[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE mass_weight/p' "$em_boundary_module"
  echo 'end module extracted_dry_boundary_relaxation'
} > "$build_directory/extracted_relaxation_em.F90"
{
  echo 'module extracted_dry_boundary_tendencies'
  echo 'use module_configure, only: grid_config_rec_type'
  echo 'use extracted_specified_boundary_tendencies, only: spec_bdytend'
  echo 'implicit none';echo 'contains'
  sed -n '/^[[:space:]]*SUBROUTINE spec_bdy_dry[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE spec_bdy_dry/p' "$em_boundary_module"
  echo 'end module extracted_dry_boundary_tendencies'
} > "$build_directory/extracted_tendencies_em.F90"
gfortran -O0 -ffp-contract=off -fno-range-check -ffree-form -ffree-line-length-none \
  "$parity_directory/module_configure_stub.F90" \
  "$build_directory/extracted_relaxation_shared.F90" \
  "$build_directory/extracted_tendencies_shared.F90" \
  "$build_directory/extracted_relaxation_em.F90" \
  "$build_directory/extracted_tendencies_em.F90" \
  "$parity_directory/dry_large_step_boundary_stage_driver.F90" \
  "$build_directory/rk_addtend_dry.F90" -o "$build_directory/oracle"
"$build_directory/oracle" > "$build_directory/actual.out"
if [ "${UPDATE_GOLDEN:-0}" = "1" ]; then cp "$build_directory/actual.out" "$expected"; fi
diff -u "$expected" "$build_directory/actual.out"
echo "PASS dry large-step boundary-stage oracle"
