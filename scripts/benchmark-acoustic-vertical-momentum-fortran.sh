#!/usr/bin/env bash
set -euo pipefail
repository_root=$(cd "$(dirname "$0")/.." && pwd)
upstream="$repository_root/upstream/WRF/dyn_em/module_small_step_em.F"
parity="$repository_root/parity/acoustic-vertical-momentum"
command -v gfortran >/dev/null || { echo "missing command required by benchmark: gfortran" >&2; exit 1; }
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-acoustic-vertical-momentum-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT
{
  echo 'module extracted_acoustic_vertical_momentum'
  echo 'use module_configure, only: grid_config_rec_type'
  echo 'implicit none'
  echo 'real, parameter :: g = 9.81'
  echo 'contains'
  sed -n '/^SUBROUTINE advance_w(/,/^END SUBROUTINE advance_w/p' "$upstream"
  echo 'end module extracted_acoustic_vertical_momentum'
} > "$build_directory/extracted.F90"
gfortran -O3 -flto -ffp-contract=off -ffree-line-length-none \
  "$parity/module_configure_stub.F90" "$build_directory/extracted.F90" \
  "$parity/acoustic_vertical_momentum_benchmark.F90" -o "$build_directory/benchmark"
"$build_directory/benchmark"
