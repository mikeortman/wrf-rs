#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
wrf_tools="$repository_root/upstream/WRF/tools"
wrf_root="$repository_root/upstream/WRF"
wrf_registry="$wrf_root/Registry/Registry.EM_COMMON"
fixture="$repository_root/parity/registry-package/Registry.package"
fortran_driver="$repository_root/parity/registry-package/registry_package_driver.F90"
projector="$repository_root/parity/registry-package/project_wrf_registry.py"
source_checksums="$repository_root/parity/registry-package/wrf-v4.7.1.sha256"

for command in make python3 gfortran cargo shasum; do
    if ! command -v "$command" >/dev/null 2>&1; then
        echo "missing command required by Registry package oracle: $command" >&2
        exit 1
    fi
done
if [ ! -f "$wrf_tools/reg_parse.c" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi
if ! grep -Fq 'commit = "f52c197ed39d12e087d02c50f412d90d418f6186"' \
    "$repository_root/UPSTREAM.toml"; then
    echo "UPSTREAM.toml does not pin the WRF v4.7.1 package-oracle commit" >&2
    exit 1
fi
(
    cd "$wrf_root"
    shasum -a 256 -c "$source_checksums"
) >/dev/null

for source in "$wrf_registry" "$fixture"; do
    grep -Eq '^state[[:space:]]+real[[:space:]]+-[[:space:]]+ikjftb[[:space:]]+moist' "$source"
    grep -Eq '^state[[:space:]]+real[[:space:]]+qv[[:space:]]+ikjftb[[:space:]]+moist' "$source"
    grep -Eq '^state[[:space:]]+real[[:space:]]+qc[[:space:]]+ikjftb[[:space:]]+moist' "$source"
    grep -Eq '^state[[:space:]]+real[[:space:]]+qr[[:space:]]+ikjftb[[:space:]]+moist' "$source"
    grep -Eq '^rconfig[[:space:]]+integer[[:space:]]+mp_physics' "$source"
    grep -Eq '^package[[:space:]]+passiveqv[[:space:]]+mp_physics==0[[:space:]]+-[[:space:]]+moist:qv' "$source"
    grep -Eq '^package[[:space:]]+kesslerscheme[[:space:]]+mp_physics==1[[:space:]]+-[[:space:]]+moist:qv,qc,qr' "$source"
done

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-registry-package.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM
mkdir -p "$build_directory/wrf/Registry" "$build_directory/wrf/inc" \
    "$build_directory/wrf/frame" "$build_directory/modules"

make -C "$wrf_tools" registry >/dev/null
cp "$fixture" "$build_directory/wrf/Registry/Registry.package"
(
    cd "$build_directory/wrf"
    "$wrf_tools/registry" Registry/Registry.package >/dev/null 2>/dev/null
)

python3 "$projector" \
    "$build_directory/wrf/frame/module_state_description.F" \
    "$build_directory/wrf/inc/scalar_indices.inc" \
    >"$build_directory/wrf.txt"

awk '$1 == "package" { print $2 }' "$fixture" >"$build_directory/fixture-packages.txt"
sed -n 's/^PACKAGE|name=\([^|]*\)|.*$/\1/p' \
    "$build_directory/wrf.txt" >"$build_directory/generated-packages.txt"
if ! cmp -s "$build_directory/fixture-packages.txt" "$build_directory/generated-packages.txt"; then
    echo "WRF generated package names do not preserve the complete fixture order" >&2
    exit 1
fi

gfortran -O0 -ffree-form -ffree-line-length-none \
    -J "$build_directory/modules" \
    -I "$build_directory/modules" \
    -I "$build_directory/wrf/inc" \
    "$build_directory/wrf/frame/module_state_description.F" \
    "$fortran_driver" \
    -o "$build_directory/registry-package-fortran"
"$build_directory/registry-package-fortran" >>"$build_directory/wrf.txt"

package_count=$(grep -c '^PACKAGE|' "$build_directory/wrf.txt")
case_count=$(grep -c '^CASE|' "$build_directory/wrf.txt")
if [ "$package_count" -ne 9 ] || [ "$case_count" -ne 9 ]; then
    echo "WRF package oracle expected 9 packages and 9 runtime cases" >&2
    exit 1
fi

cargo run --quiet --release -p wrf-physics --example registry_package_oracle -- \
    "$fixture" >"$build_directory/rust.txt"

if ! cmp -s "$build_directory/wrf.txt" "$build_directory/rust.txt"; then
    diff -u "$build_directory/wrf.txt" "$build_directory/rust.txt" | sed -n '1,200p'
    echo "Rust Registry package projection differs from pinned WRF" >&2
    exit 1
fi

line_count=$(wc -l <"$build_directory/wrf.txt" | tr -d ' ')
echo "PASS Registry package oracle: $line_count static and runtime rows match pinned WRF exactly."
