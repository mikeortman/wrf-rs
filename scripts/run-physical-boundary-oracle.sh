#!/usr/bin/env bash
set -euo pipefail

repository_root=$(cd "$(dirname "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/share/module_bc.F"
parity_directory="$repository_root/parity/physical-boundary"
expected="$repository_root/crates/wrf-dynamics/test-data/physical_boundary.out.correct"

command -v gfortran >/dev/null || {
  echo "missing command required by physical-boundary oracle: gfortran" >&2
  exit 1
}

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-physical-boundary.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM
{
  echo 'module extracted_physical_boundary'
  echo 'use module_configure, only: grid_config_rec_type'
  echo 'implicit none'
  echo 'integer, parameter :: bdyzone=4'
  echo 'contains'
  sed -n '/^[[:space:]]*SUBROUTINE set_physical_bc2d(/,/^[[:space:]]*END SUBROUTINE set_physical_bc2d/p' "$upstream_module"
  sed -n '/^[[:space:]]*SUBROUTINE set_physical_bc3d(/,/^[[:space:]]*END SUBROUTINE set_physical_bc3d/p' "$upstream_module"
  echo 'end module extracted_physical_boundary'
} > "$build_directory/extracted.F90"

gfortran -O0 -ffp-contract=off -fno-range-check -cpp -ffree-form \
  -ffree-line-length-none "$parity_directory/module_configure_stub.F90" \
  "$build_directory/extracted.F90" "$parity_directory/physical_boundary_driver.F90" \
  -o "$build_directory/oracle"
"$build_directory/oracle" > "$build_directory/actual.out"

if [[ "${1:-}" == "--accept" ]]; then
  cp "$build_directory/actual.out" "$expected"
else
  diff -u "$expected" "$build_directory/actual.out"
fi
echo "PASS WRF set_physical_bc3d/set_physical_bc2d oracle"
