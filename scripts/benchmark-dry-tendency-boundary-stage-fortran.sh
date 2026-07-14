#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
shared_module="$repository_root/upstream/WRF/share/module_bc.F"
em_boundary_module="$repository_root/upstream/WRF/dyn_em/module_bc_em.F"
em_module="$repository_root/upstream/WRF/dyn_em/module_em.F"
parity_directory="$repository_root/parity/dry-tendency-boundary-stage"
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-dry-stage-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

command -v gfortran >/dev/null 2>&1 || { echo "missing gfortran" >&2; exit 1; }
sed -n '/^SUBROUTINE rk_addtend_dry /,/^END SUBROUTINE rk_addtend_dry/p' "$em_module" > "$build_directory/rk_addtend_dry.F90"
{
  echo 'module extracted_specified_boundary_tendencies'
  echo 'use module_configure, only: grid_config_rec_type'
  echo 'implicit none';echo 'contains'
  sed -n '/^[[:space:]]*SUBROUTINE spec_bdytend[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE spec_bdytend/p' "$shared_module"
  echo 'end module extracted_specified_boundary_tendencies'
} > "$build_directory/extracted_shared.F90"
{
  echo 'module extracted_dry_boundary_tendencies'
  echo 'use module_configure, only: grid_config_rec_type'
  echo 'use extracted_specified_boundary_tendencies, only: spec_bdytend'
  echo 'implicit none';echo 'contains'
  sed -n '/^[[:space:]]*SUBROUTINE spec_bdy_dry[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE spec_bdy_dry/p' "$em_boundary_module"
  echo 'end module extracted_dry_boundary_tendencies'
} > "$build_directory/extracted_em.F90"
gfortran -O3 -flto -ffp-contract=off -fno-range-check -ffree-form -ffree-line-length-none \
  "$parity_directory/../dry-boundary-tendencies/module_configure_stub.F90" \
  "$build_directory/extracted_shared.F90" "$build_directory/extracted_em.F90" \
  "$parity_directory/dry_tendency_boundary_stage_benchmark.F90" \
  "$build_directory/rk_addtend_dry.F90" -o "$build_directory/benchmark"
"$build_directory/benchmark"
