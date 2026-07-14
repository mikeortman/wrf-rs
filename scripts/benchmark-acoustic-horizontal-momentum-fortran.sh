#!/usr/bin/env bash
set -euo pipefail
repository_root=$(cd "$(dirname "$0")/.." && pwd)
upstream="$repository_root/upstream/WRF/dyn_em/module_small_step_em.F"
parity="$repository_root/parity/acoustic-horizontal-momentum"
command -v gfortran >/dev/null || { echo "missing command required by benchmark: gfortran" >&2; exit 1; }
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-acoustic-horizontal-momentum-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT
{
  echo 'module extracted_acoustic_horizontal_momentum'
  echo 'use module_configure, only: grid_config_rec_type'
  echo 'implicit none'
  echo 'contains'
  sed -n '/^SUBROUTINE advance_uv/,/^END SUBROUTINE advance_uv/p' "$upstream"
  echo 'end module extracted_acoustic_horizontal_momentum'
} > "$build_directory/extracted.F90"
gfortran -O3 -flto -ffree-line-length-none \
  "$parity/module_configure_stub.F90" "$build_directory/extracted.F90" \
  "$parity/acoustic_horizontal_momentum_benchmark.F90" -o "$build_directory/benchmark"
"$build_directory/benchmark"
