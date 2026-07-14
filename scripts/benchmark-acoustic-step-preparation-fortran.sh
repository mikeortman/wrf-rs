#!/usr/bin/env bash
set -euo pipefail
repository_root=$(cd "$(dirname "$0")/.." && pwd)
upstream="$repository_root/upstream/WRF/dyn_em/module_small_step_em.F"
driver="$repository_root/parity/acoustic-step-preparation/acoustic_step_preparation_benchmark.F90"
command -v gfortran >/dev/null || { echo "missing command required by benchmark: gfortran" >&2; exit 1; }
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-acoustic-preparation-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT
{
  echo 'module module_configure'
  echo 'end module module_configure'
  echo 'module module_model_constants'
  echo 'real, parameter :: r_d=287., cp=7.*r_d/2., cpovcv=cp/(cp-r_d)'
  echo 'end module module_model_constants'
  echo 'module module_small_step_em'
  echo 'use module_configure'
  echo 'use module_model_constants'
  echo 'contains'
  sed -n '/^SUBROUTINE small_step_prep/,/^END SUBROUTINE small_step_prep/p' "$upstream"
  echo 'end module module_small_step_em'
} > "$build_directory/extracted.F90"
gfortran -O3 -flto -ffree-line-length-none "$build_directory/extracted.F90" "$driver" -o "$build_directory/benchmark"
"$build_directory/benchmark"
