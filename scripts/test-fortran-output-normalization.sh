#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
normalizer="$repository_root/scripts/normalize-fortran-single-nans.awk"
input="$repository_root/parity/omega-diagnosis/nan-normalization.input"
expected="$repository_root/parity/omega-diagnosis/nan-normalization.expected"
build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-fortran-normalization.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

LC_ALL=C awk -f "$normalizer" "$input" > "$build_directory/actual"
diff -u "$expected" "$build_directory/actual"

sed 's/3F800000/40000000/' "$input" | \
    LC_ALL=C awk -f "$normalizer" > "$build_directory/changed-finite"
if cmp -s "$expected" "$build_directory/changed-finite"; then
    echo "normalizer incorrectly hid a finite-value mismatch" >&2
    exit 1
fi

echo "PASS Fortran single-precision NaN normalization"
