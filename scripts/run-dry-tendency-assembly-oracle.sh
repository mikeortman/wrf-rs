#!/usr/bin/env bash
set -euo pipefail

repository_root=$(cd "$(dirname "$0")/.." && pwd)
driver="$repository_root/parity/dry-tendency-assembly/dry_tendency_assembly_driver.F90"
upstream="$repository_root/upstream/WRF/dyn_em/module_em.F"
expected="$repository_root/crates/wrf-dynamics/test-data/dry_tendency_assembly.out.correct"

command -v gfortran >/dev/null || { echo "missing command required by oracle: gfortran" >&2; exit 1; }
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-dry-tendency-oracle.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT

sed -n '/^SUBROUTINE rk_addtend_dry /,/^END SUBROUTINE rk_addtend_dry/p' "$upstream" > "$build_directory/rk_addtend_dry.F90"
gfortran -cpp -O0 -ffp-contract=off -fno-range-check "$driver" "$build_directory/rk_addtend_dry.F90" -o "$build_directory/oracle"
"$build_directory/oracle" > "$build_directory/actual.out"

if [[ "${1:-}" == "--accept" ]]; then
  cp "$build_directory/actual.out" "$expected"
else
  diff -u "$expected" "$build_directory/actual.out"
fi
