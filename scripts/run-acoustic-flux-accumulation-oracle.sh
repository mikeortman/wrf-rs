#!/usr/bin/env bash
set -euo pipefail
repository_root=$(cd "$(dirname "$0")/.." && pwd)
upstream="$repository_root/upstream/WRF/dyn_em/module_small_step_em.F"
parity="$repository_root/parity/acoustic-flux-accumulation"
expected="$repository_root/crates/wrf-dynamics/test-data/acoustic_flux_accumulation.out.correct"
command -v gfortran >/dev/null || { echo "missing command required by oracle: gfortran" >&2; exit 1; }
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-acoustic-flux-oracle.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT
{
  echo 'module extracted_acoustic_flux_accumulation'
  echo 'implicit none'
  echo 'contains'
  sed -n '/^SUBROUTINE sumflux/,/^END SUBROUTINE sumflux/p' "$upstream"
  echo 'end module extracted_acoustic_flux_accumulation'
} > "$build_directory/extracted.F90"
gfortran -O0 -ffp-contract=off -fno-range-check -ffree-line-length-none \
  "$build_directory/extracted.F90" "$parity/acoustic_flux_accumulation_driver.F90" \
  -o "$build_directory/oracle"
"$build_directory/oracle" > "$build_directory/actual.out"
if [[ "${1:-}" == "--accept" ]]; then
  cp "$build_directory/actual.out" "$expected"
else
  diff -u "$expected" "$build_directory/actual.out"
fi
