#!/usr/bin/env bash
set -euo pipefail

repository_root=$(cd "$(dirname "$0")/.." && pwd)
small_step_module="$repository_root/upstream/WRF/dyn_em/module_small_step_em.F"
boundary_module="$repository_root/upstream/WRF/share/module_bc.F"
em_boundary_module="$repository_root/upstream/WRF/dyn_em/module_bc_em.F"
parity_directory="$repository_root/parity/acoustic-boundary-stage"
expected="$repository_root/crates/wrf-dynamics/test-data/acoustic_boundary_stage.out.correct"
normalizer="$repository_root/scripts/normalize-fortran-single-nans.awk"

command -v gfortran >/dev/null || {
  echo "missing command required by acoustic-boundary-stage oracle: gfortran" >&2
  exit 1
}

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-acoustic-boundary-stage.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM
{
  echo 'module module_model_constants'
  echo 'real, parameter :: r_d=287., cp=7.*r_d/2., cpovcv=cp/(cp-r_d), g=9.81'
  echo 'end module module_model_constants'
  echo 'module extracted_acoustic_boundary_stage'
  echo 'use module_configure, only: grid_config_rec_type'
  echo 'use module_model_constants'
  echo 'implicit none'
  echo 'integer, parameter :: bdyzone=4'
  echo 'contains'
  for routine in small_step_prep calc_p_rho calc_coef_w advance_uv advance_mu_t advance_w sumflux; do
    sed -n "/^SUBROUTINE $routine/,/^END SUBROUTINE $routine/p" "$small_step_module"
  done
  for routine in spec_bdyupdate zero_grad_bdy set_physical_bc2d set_physical_bc3d; do
    sed -n "/^[[:space:]]*SUBROUTINE $routine[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE $routine/p" "$boundary_module"
  done
  sed -n '/^[[:space:]]*SUBROUTINE spec_bdyupdate_ph[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE spec_bdyupdate_ph/p' "$em_boundary_module"
  echo 'end module extracted_acoustic_boundary_stage'
} > "$build_directory/extracted.F90"

gfortran -O0 -ffp-contract=off -fno-range-check -fallow-argument-mismatch \
  -cpp -ffree-form -ffree-line-length-none \
  "$parity_directory/module_configure_stub.F90" "$build_directory/extracted.F90" \
  "$parity_directory/acoustic_boundary_stage_driver.F90" -o "$build_directory/oracle"
"$build_directory/oracle" > "$build_directory/actual.raw"
LC_ALL=C awk -f "$normalizer" "$build_directory/actual.raw" \
  > "$build_directory/actual.out"

if [[ "${1:-}" == "--accept" ]]; then
  cp "$build_directory/actual.out" "$expected"
else
  LC_ALL=C awk -f "$normalizer" "$expected" \
    > "$build_directory/expected.out"
  diff -u "$build_directory/expected.out" "$build_directory/actual.out"
fi
echo "PASS WRF complete acoustic boundary-stage oracle"
