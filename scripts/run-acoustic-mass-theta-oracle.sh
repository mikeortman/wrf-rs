#!/usr/bin/env bash
set -euo pipefail
repository_root=$(cd "$(dirname "$0")/.." && pwd)
upstream="$repository_root/upstream/WRF/dyn_em/module_small_step_em.F"
parity="$repository_root/parity/acoustic-mass-theta"
expected="$repository_root/crates/wrf-dynamics/test-data/acoustic_mass_theta.out.correct"
command -v gfortran >/dev/null || { echo "missing command required by oracle: gfortran" >&2; exit 1; }
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-acoustic-mass-theta-oracle.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT
{
  echo 'module extracted_acoustic_mass_theta'
  echo 'use module_configure, only: grid_config_rec_type'
  echo 'implicit none'
  echo 'contains'
  sed -n '/^SUBROUTINE advance_mu_t/,/^END SUBROUTINE advance_mu_t/p' "$upstream"
  echo 'end module extracted_acoustic_mass_theta'
} > "$build_directory/extracted.F90"
gfortran -O0 -ffp-contract=off -fno-range-check -ffree-line-length-none \
  "$parity/module_configure_stub.F90" "$build_directory/extracted.F90" \
  "$parity/acoustic_mass_theta_driver.F90" -o "$build_directory/oracle"
"$build_directory/oracle" > "$build_directory/actual.out"
if [[ "${1:-}" == "--accept" ]]; then
  cp "$build_directory/actual.out" "$expected"
else
  diff -u "$expected" "$build_directory/actual.out"
fi
