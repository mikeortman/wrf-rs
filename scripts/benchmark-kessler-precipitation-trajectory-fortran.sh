#!/usr/bin/env bash
set -euo pipefail

repository_root=$(cd "$(dirname "$0")/.." && pwd)
kessler_module="$repository_root/upstream/WRF/phys/module_mp_kessler.F"
benchmark="$repository_root/parity/kessler-precipitation-trajectory/kessler_precipitation_trajectory_benchmark.F90"
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-kessler-trajectory-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

command -v gfortran >/dev/null || {
  echo "missing command required by Kessler precipitation trajectory benchmark: gfortran" >&2
  exit 1
}
if [[ ! -f "$kessler_module" ]]; then
  echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
  exit 1
fi

(
  cd "$build_directory"
  gfortran -O3 -flto -ffp-contract=off -cpp -ffree-form -ffree-line-length-none \
    "$kessler_module" "$benchmark" \
    -o "$build_directory/benchmark"
)
"$build_directory/benchmark"
