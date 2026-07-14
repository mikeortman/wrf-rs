#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/dyn_em/module_big_step_utilities_em.F"
fixture_directory="$repository_root/parity/periodic-column-mass"
expected="$repository_root/crates/wrf-dynamics/test-data/periodic_column_mass.out.correct"

if ! command -v gfortran >/dev/null 2>&1; then
    echo "missing command required by periodic column-mass oracle: gfortran" >&2
    exit 1
fi
if [ ! -f "$upstream_module" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-periodic-column-mass.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM
extracted_module="$build_directory/extracted_big_step_column_mass.F90"

sed -n '1,$p' "$fixture_directory/extracted_module_header.F90" > "$extracted_module"
sed -n '/^SUBROUTINE calc_mu_uv /,/^END SUBROUTINE calc_mu_uv$/p' \
    "$upstream_module" >> "$extracted_module"
sed -n '/^SUBROUTINE calc_mu_uv_1 /,/^END SUBROUTINE calc_mu_uv_1$/p' \
    "$upstream_module" >> "$extracted_module"
sed -n '1,$p' "$fixture_directory/extracted_module_footer.F90" >> "$extracted_module"

gfortran -O0 -ffree-form -ffree-line-length-none \
    "$fixture_directory/module_configure_stub.F90" \
    "$extracted_module" \
    "$fixture_directory/periodic_column_mass_driver.F90" \
    -o "$build_directory/periodic_column_mass_driver"
"$build_directory/periodic_column_mass_driver" > "$build_directory/actual.out"

if [ "${UPDATE_GOLDEN:-0}" = "1" ]; then
    cp "$build_directory/actual.out" "$expected"
fi
if ! cmp -s "$build_directory/actual.out" "$expected"; then
    diff -u "$expected" "$build_directory/actual.out"
    exit 1
fi

echo "PASS WRF calc_mu_uv and calc_mu_uv_1 oracle"
