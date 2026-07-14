#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream="$repository_root/upstream/WRF/dyn_em/module_em.F"
driver="$repository_root/parity/dry-tendency-assembly/dry_tendency_assembly_benchmark.F90"
command -v gfortran >/dev/null 2>&1 || { echo "missing command required by benchmark: gfortran" >&2; exit 1; }
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-dry-tendency-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM
sed -n '/^SUBROUTINE rk_addtend_dry /,/^END SUBROUTINE rk_addtend_dry/p' "$upstream" > "$build_directory/rk_addtend_dry.F90"
gfortran -O3 -flto -cpp -ffree-form -ffree-line-length-none \
  "$build_directory/rk_addtend_dry.F90" "$driver" -o "$build_directory/benchmark"
echo "compiler $(gfortran --version | sed -n '1p')"
echo "flags -O3 -flto (no fast-math or explicit native-CPU flag)"
"$build_directory/benchmark"
