#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
parity_directory="$repository_root/parity/randomized-arw"
corpus_directory="$repository_root/crates/wrf-dynamics/test-data/randomized-arw"
positive_module="$repository_root/upstream/WRF/dyn_em/module_positive_definite.F"
held_suarez_module="$repository_root/upstream/WRF/dyn_em/module_damping_em.F"
column_mass_module="$repository_root/upstream/WRF/dyn_em/module_big_step_utilities_em.F"
error_stub="$repository_root/parity/positive-definite/module_wrf_error_stub.F90"
mode=verify

if [ "${1:-}" = "--write-goldens" ]; then
    mode=write
    shift
fi
if [ "$#" -ne 0 ]; then
    echo "usage: $0 [--write-goldens]" >&2
    exit 2
fi
if ! command -v gfortran >/dev/null 2>&1; then
    echo "missing command required by randomized ARW oracles: gfortran" >&2
    exit 1
fi
if [ ! -f "$positive_module" ] || [ ! -f "$held_suarez_module" ] || [ ! -f "$column_mass_module" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-randomized-arw.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM
generated_corpus_directory="$build_directory/generated-corpora"

cargo run --quiet --manifest-path "$repository_root/Cargo.toml" \
    -p wrf-arw-corpus-generator -- "$generated_corpus_directory"
for corpus_name in positive_definite_sheet positive_definite_slab held_suarez column_mass_staggering; do
    if ! cmp -s "$generated_corpus_directory/${corpus_name}.in" "$corpus_directory/${corpus_name}.in"; then
        echo "committed randomized corpus is stale: ${corpus_name}.in" >&2
        diff -u "$corpus_directory/${corpus_name}.in" "$generated_corpus_directory/${corpus_name}.in" || true
        exit 1
    fi
done

gfortran -c -J "$build_directory" "$error_stub" \
    -o "$build_directory/module_wrf_error_stub.o"
gfortran -c -cpp -ffree-form -ffree-line-length-none \
    -I "$build_directory" -J "$build_directory" \
    "$positive_module" -o "$build_directory/module_positive_definite.o"
for routine in positive_definite_sheet positive_definite_slab; do
    gfortran -c -ffree-form -ffree-line-length-none \
        -I "$build_directory" -J "$build_directory" \
        "$parity_directory/${routine}_driver.F90" \
        -o "$build_directory/${routine}_driver.o"
    gfortran "$build_directory/module_wrf_error_stub.o" \
        "$build_directory/module_positive_definite.o" \
        "$build_directory/${routine}_driver.o" \
        -o "$build_directory/${routine}_driver"
done

gfortran -c -cpp -ffree-form -ffree-line-length-none \
    -I "$build_directory" -J "$build_directory" \
    "$held_suarez_module" -o "$build_directory/module_damping_em.o"
gfortran -c -ffree-form -ffree-line-length-none \
    -I "$build_directory" -J "$build_directory" \
    "$parity_directory/held_suarez_driver.F90" \
    -o "$build_directory/held_suarez_driver.o"
gfortran "$build_directory/module_wrf_error_stub.o" \
    "$build_directory/module_damping_em.o" \
    "$build_directory/held_suarez_driver.o" \
    -o "$build_directory/held_suarez_driver"

sed -n '/^SUBROUTINE calc_mu_staggered /,/^END SUBROUTINE calc_mu_staggered$/p' \
    "$column_mass_module" > "$build_directory/calc_mu_staggered.F90"
gfortran -ffree-form -ffree-line-length-none \
    "$build_directory/calc_mu_staggered.F90" \
    "$parity_directory/column_mass_staggering_driver.F90" \
    -o "$build_directory/column_mass_staggering_driver"

for corpus_name in positive_definite_sheet positive_definite_slab held_suarez column_mass_staggering; do
    "$build_directory/${corpus_name}_driver" "$generated_corpus_directory/${corpus_name}.in" \
        > "$build_directory/${corpus_name}.actual"
    expected_output="$corpus_directory/${corpus_name}.out.correct"
    if [ "$mode" = write ]; then
        cp "$build_directory/${corpus_name}.actual" "$expected_output"
        echo "WROTE $expected_output"
    else
        diff -u "$expected_output" "$build_directory/${corpus_name}.actual"
        echo "PASS randomized WRF ${corpus_name} corpus"
    fi
done
