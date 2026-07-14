#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
build_directory="${TMPDIR:-/tmp}/wrf-rs-netcdf-oracle"
mkdir -p "${build_directory}"

cc $(nc-config --cflags) -O3 \
  "${repo_root}/parity/netcdf/minimal_wrf_restart.c" \
  $(nc-config --libs) \
  -o "${build_directory}/minimal_wrf_restart"

"${build_directory}/minimal_wrf_restart" \
  "${build_directory}/netcdf-c-restart.nc"
cargo run --quiet --release --manifest-path "${repo_root}/Cargo.toml" \
  -p wrf-io --example minimal_restart_oracle -- write \
  "${build_directory}/rust-restart.nc"
cmp "${build_directory}/netcdf-c-restart.nc" \
  "${build_directory}/rust-restart.nc"
cargo run --quiet --release --manifest-path "${repo_root}/Cargo.toml" \
  -p wrf-io --example minimal_restart_oracle -- compare \
  "${build_directory}/netcdf-c-restart.nc" \
  "${build_directory}/rust-restart.nc"

echo "NetCDF-C and Rust restart files are byte-identical and semantically equal."
