#!/usr/bin/env bash
set -euo pipefail
repository_root=$(cd "$(dirname "$0")/.." && pwd)
upstream="$repository_root/upstream/WRF/dyn_em/module_small_step_em.F"
driver="$repository_root/parity/vertical-acoustic-coefficients/vertical_acoustic_coefficients_benchmark.F90"
command -v gfortran >/dev/null || { echo "missing command required by benchmark: gfortran" >&2; exit 1; }
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-vertical-acoustic-coefficients-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT
{
  echo 'module extracted_vertical_acoustic_coefficients'
  echo 'contains'
  sed -n '/^SUBROUTINE calc_coef_w/,/^END SUBROUTINE calc_coef_w/p' "$upstream"
  echo 'end module extracted_vertical_acoustic_coefficients'
} > "$build_directory/extracted.F90"
gfortran -O3 -flto -ffree-line-length-none \
  "$build_directory/extracted.F90" "$driver" -o "$build_directory/benchmark"
"$build_directory/benchmark"
