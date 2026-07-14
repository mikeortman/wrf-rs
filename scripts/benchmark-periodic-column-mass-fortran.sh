#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/dyn_em/module_big_step_utilities_em.F"
fixture_directory="$repository_root/parity/periodic-column-mass"
driver="$fixture_directory/periodic_column_mass_benchmark.F90"

if ! command -v gfortran >/dev/null 2>&1; then
    echo "missing command required by periodic column-mass benchmark: gfortran" >&2
    exit 1
fi
if [ ! -f "$upstream_module" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-periodic-column-mass-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM
extracted_module="$build_directory/extracted_big_step_column_mass.F90"

sed -n '1,$p' "$fixture_directory/extracted_module_header.F90" > "$extracted_module"
sed -n '/^SUBROUTINE calc_mu_uv /,/^END SUBROUTINE calc_mu_uv$/p' \
    "$upstream_module" >> "$extracted_module"
sed -n '1,$p' "$fixture_directory/extracted_module_footer.F90" >> "$extracted_module"

gfortran -O3 -flto -ffree-form -ffree-line-length-none \
    "$fixture_directory/module_configure_stub.F90" \
    "$extracted_module" \
    "$driver" \
    -o "$build_directory/periodic_column_mass_benchmark"

echo "compiler $(gfortran --version | sed -n '1p')"
echo "flags -O3 -flto (no fast-math or explicit native-CPU flag)"
"$build_directory/periodic_column_mass_benchmark"
