#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/share/module_bc.F"
parity_directory="$repository_root/parity/flow-dependent-boundary"
command -v gfortran >/dev/null 2>&1 || {
    echo "missing command required by benchmark: gfortran" >&2
    exit 1
}
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-flow-dependent-boundary-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM
{
    echo 'module extracted_flow_dependent_boundary'
    echo 'use module_configure, only: grid_config_rec_type'
    echo 'implicit none'
    echo 'contains'
    sed -n '/^[[:space:]]*SUBROUTINE flow_dep_bdy[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE flow_dep_bdy/p' "$upstream_module"
    echo 'end module extracted_flow_dependent_boundary'
} > "$build_directory/extracted.F90"
gfortran -O3 -flto -ffp-contract=off -ffree-form -ffree-line-length-none \
    "$parity_directory/module_configure_stub.F90" "$build_directory/extracted.F90" \
    "$parity_directory/flow_dependent_boundary_benchmark.F90" \
    -o "$build_directory/benchmark"
"$build_directory/benchmark"
