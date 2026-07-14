#!/usr/bin/env bash
set -euo pipefail
repository_root=$(cd "$(dirname "$0")/.." && pwd)
upstream="$repository_root/upstream/WRF/dyn_em/module_small_step_em.F"
driver="$repository_root/parity/acoustic-pressure/acoustic_pressure_driver.F90"
expected="$repository_root/crates/wrf-dynamics/test-data/acoustic_pressure.out.correct"
command -v gfortran >/dev/null || { echo "missing command required by oracle: gfortran" >&2; exit 1; }
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-acoustic-pressure-oracle.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT
{
  echo 'module extracted_acoustic_pressure'
  echo 'contains'
  sed -n '/^SUBROUTINE calc_p_rho/,/^END SUBROUTINE calc_p_rho/p' "$upstream"
  echo 'end module extracted_acoustic_pressure'
} > "$build_directory/extracted.F90"
gfortran -O0 -ffp-contract=off -fno-range-check -ffree-line-length-none \
  "$build_directory/extracted.F90" "$driver" -o "$build_directory/oracle"
"$build_directory/oracle" > "$build_directory/actual.out"
if [[ "${1:-}" == "--accept" ]]; then
  cp "$build_directory/actual.out" "$expected"
else
  diff -u "$expected" "$build_directory/actual.out"
fi
