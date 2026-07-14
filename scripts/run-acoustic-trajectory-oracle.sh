#!/usr/bin/env bash
set -euo pipefail
repository_root=$(cd "$(dirname "$0")/.." && pwd)
upstream="$repository_root/upstream/WRF/dyn_em/module_small_step_em.F"
parity="$repository_root/parity/acoustic-trajectory"
expected="$repository_root/crates/wrf-dynamics/test-data/acoustic_trajectory.out.correct"
command -v gfortran >/dev/null || { echo "missing command required by oracle: gfortran" >&2; exit 1; }
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-acoustic-trajectory-oracle.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT
{
  echo 'module module_model_constants'
  echo 'real, parameter :: r_d=287., cp=7.*r_d/2., cpovcv=cp/(cp-r_d), g=9.81'
  echo 'end module module_model_constants'
  echo 'module extracted_acoustic_trajectory'
  echo 'use module_configure, only: grid_config_rec_type'
  echo 'use module_model_constants'
  echo 'implicit none'
  echo 'contains'
  for routine in small_step_prep calc_p_rho calc_coef_w advance_uv advance_mu_t advance_w sumflux; do
    sed -n "/^SUBROUTINE $routine/,/^END SUBROUTINE $routine/p" "$upstream"
  done
  echo 'end module extracted_acoustic_trajectory'
} > "$build_directory/extracted.F90"
gfortran -O0 -ffp-contract=off -fno-range-check -ffree-line-length-none \
  "$parity/module_configure_stub.F90" "$build_directory/extracted.F90" \
  "$parity/acoustic_trajectory_driver.F90" -o "$build_directory/oracle"
"$build_directory/oracle" > "$build_directory/actual.out"
if [[ "${1:-}" == "--accept" ]]; then
  cp "$build_directory/actual.out" "$expected"
else
  diff -u "$expected" "$build_directory/actual.out"
fi
