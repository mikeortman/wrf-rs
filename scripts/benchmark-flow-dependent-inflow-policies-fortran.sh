#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/share/module_bc.F"
parity_directory="$repository_root/parity/flow-dependent-inflow-policies"
command -v gfortran >/dev/null 2>&1 || {
    echo "missing command required by benchmark: gfortran" >&2
    exit 1
}
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-flow-dependent-inflow-policies-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM
{
    echo 'module extracted_flow_dependent_inflow_policies'
    echo 'use module_configure, only: grid_config_rec_type'
    echo 'implicit none'
    echo 'contains'
    sed -n '/^[[:space:]]*SUBROUTINE flow_dep_bdy_qnn[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE flow_dep_bdy_qnn/p' "$upstream_module"
    sed -n '/^[[:space:]]*SUBROUTINE flow_dep_bdy_fixed_inflow[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE flow_dep_bdy_fixed_inflow/p' "$upstream_module"
    echo 'end module extracted_flow_dependent_inflow_policies'
} > "$build_directory/extracted.F90"
gfortran -O3 -flto -ffp-contract=off -ffree-form -ffree-line-length-none \
    "$parity_directory/module_configure_stub.F90" "$build_directory/extracted.F90" \
    "$parity_directory/flow_dependent_inflow_policies_benchmark.F90" \
    -o "$build_directory/benchmark"
"$build_directory/benchmark"
