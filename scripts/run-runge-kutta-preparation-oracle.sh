#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/dyn_em/module_big_step_utilities_em.F"
fixture_directory="$repository_root/parity/runge-kutta-preparation"
column_mass_fixture="$repository_root/parity/periodic-column-mass"
expected="$repository_root/crates/wrf-dynamics/test-data/runge_kutta_preparation.out.correct"

if ! command -v gfortran >/dev/null 2>&1; then
    echo "missing command required by Runge-Kutta preparation oracle: gfortran" >&2
    exit 1
fi
if [ ! -f "$upstream_module" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-rk-preparation.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM
extracted_column_mass="$build_directory/extracted_column_mass.F90"

sed -n '1,$p' "$column_mass_fixture/extracted_module_header.F90" > "$extracted_column_mass"
sed -n '/^SUBROUTINE calc_mu_uv /,/^END SUBROUTINE calc_mu_uv$/p' \
    "$upstream_module" >> "$extracted_column_mass"
sed -n '1,$p' "$column_mass_fixture/extracted_module_footer.F90" >> "$extracted_column_mass"

for routine in calculate_full couple_momentum calc_ww_cp calc_cq calc_alt calc_php; do
    sed -n "/^SUBROUTINE $routine /,/^END SUBROUTINE $routine$/p" \
        "$upstream_module" > "$build_directory/$routine.F90"
done

gfortran -O0 -cpp -DPARAM_FIRST_SCALAR=2 -ffree-form -ffree-line-length-none \
    "$column_mass_fixture/module_configure_stub.F90" \
    "$extracted_column_mass" \
    "$build_directory/calculate_full.F90" \
    "$build_directory/couple_momentum.F90" \
    "$build_directory/calc_ww_cp.F90" \
    "$build_directory/calc_cq.F90" \
    "$build_directory/calc_alt.F90" \
    "$build_directory/calc_php.F90" \
    "$fixture_directory/runge_kutta_preparation_driver.F90" \
    -o "$build_directory/runge_kutta_preparation_driver"
"$build_directory/runge_kutta_preparation_driver" > "$build_directory/actual.out"

if [ "${UPDATE_GOLDEN:-0}" = "1" ]; then
    cp "$build_directory/actual.out" "$expected"
fi
if ! cmp -s "$build_directory/actual.out" "$expected"; then
    diff -u "$expected" "$build_directory/actual.out"
    exit 1
fi

echo "PASS WRF Runge-Kutta preparation coupled oracle"
