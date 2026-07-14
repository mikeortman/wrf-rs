#!/usr/bin/env bash
set -euo pipefail
repository_root=$(cd "$(dirname "$0")/.." && pwd)
upstream="$repository_root/upstream/WRF/dyn_em/module_small_step_em.F"
parity="$repository_root/parity/acoustic-flux-accumulation"
command -v gfortran >/dev/null || { echo "missing command required by benchmark: gfortran" >&2; exit 1; }
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-acoustic-flux-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT
{
  echo 'module extracted_acoustic_flux_accumulation'
  echo 'implicit none'
  echo 'contains'
  sed -n '/^SUBROUTINE sumflux/,/^END SUBROUTINE sumflux/p' "$upstream"
  echo 'end module extracted_acoustic_flux_accumulation'
} > "$build_directory/extracted.F90"
gfortran -O3 -flto -ffp-contract=off -ffree-line-length-none \
  "$build_directory/extracted.F90" "$parity/acoustic_flux_accumulation_benchmark.F90" \
  -o "$build_directory/benchmark"
"$build_directory/benchmark"
