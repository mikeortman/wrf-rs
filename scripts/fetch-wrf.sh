#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

fetch_archive() {
    display_name=$1
    archive_url=$2
    expected_sha256=$3
    archive_path=$4
    destination=$5
    marker=$6

    if [ -f "$destination/$marker" ]; then
        echo "$display_name is already present at $destination"
        return
    fi

    if [ -f "$archive_path" ]; then
        actual_sha256=$(shasum -a 256 "$archive_path" | awk '{print $1}')
    else
        actual_sha256='missing'
    fi

    if [ "$actual_sha256" != "$expected_sha256" ]; then
        curl -L --fail --retry 3 "$archive_url" -o "$archive_path"
        actual_sha256=$(shasum -a 256 "$archive_path" | awk '{print $1}')
    fi

    if [ "$actual_sha256" != "$expected_sha256" ]; then
        echo "$display_name checksum mismatch: expected $expected_sha256, got $actual_sha256" >&2
        exit 1
    fi

    mkdir -p "$destination"
    tar -xzf "$archive_path" --strip-components=1 -C "$destination"
    echo "Fetched $display_name into $destination"
}

temporary_directory=${WRF_DOWNLOAD_CACHE:-/tmp}
wrf_destination="$repository_root/upstream/WRF"

fetch_archive \
    'WRF v4.7.1' \
    'https://github.com/wrf-model/WRF/archive/refs/tags/v4.7.1.tar.gz' \
    '7227916c7871cec36a0a1bf23619fe6d29664474679c8207b4c6f22b10cbab6b' \
    "$temporary_directory/WRF-v4.7.1.tar.gz" \
    "$wrf_destination" \
    'LICENSE.txt'

fetch_archive \
    'Noah-MP e5c0859' \
    'https://github.com/NCAR/noahmp/archive/e5c0859874407859936739e8be8741f9aed369ee.tar.gz' \
    '2470d879cbf1ba2cc0e5fd0710bfc55ace6165fdb2dac277342a93f0e045c8cd' \
    "$temporary_directory/noahmp-e5c0859.tar.gz" \
    "$wrf_destination/phys/noahmp" \
    'README.md'

fetch_archive \
    'MYNN-EDMF 90f36c2' \
    'https://github.com/NCAR/MYNN-EDMF/archive/90f36c25259ec1960b24325f5b29ac7c5adeac73.tar.gz' \
    'c68b6d08890e9231f4451927ec5b38e65d93dcf904fd1c034b257e5fe1b71d5d' \
    "$temporary_directory/MYNN-EDMF-90f36c2.tar.gz" \
    "$wrf_destination/phys/MYNN-EDMF" \
    'README.md'

fetch_archive \
    'HPC workflows dfc8e6d' \
    'https://github.com/islas/hpc-workflows/archive/dfc8e6d823b80497ea41bab94e1fdf3f4594ad18.tar.gz' \
    '2895473aa419b7720f905f818919a97e76c65408669fdd1d40e486e56dcc1c3b' \
    "$temporary_directory/hpc-workflows-dfc8e6d.tar.gz" \
    "$wrf_destination/.ci/hpc-workflows" \
    '.ci/Test.py'
