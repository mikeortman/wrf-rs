#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
upstream_module="$repository_root/upstream/WRF/phys/module_mp_kessler.F"
driver="$repository_root/parity/kessler/kessler_benchmark.F90"

if ! command -v gfortran >/dev/null 2>&1; then
    echo "missing command required by Kessler benchmark: gfortran" >&2
    exit 1
fi
if [ ! -f "$upstream_module" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-kessler-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

gfortran -O3 -flto -ffree-form -ffree-line-length-none \
    "$upstream_module" "$driver" -o "$build_directory/kessler-benchmark"

echo "compiler $(gfortran --version | sed -n '1p')"
echo "flags -O3 -flto (no fast-math or explicit native-CPU flag)"
"$build_directory/kessler-benchmark"
