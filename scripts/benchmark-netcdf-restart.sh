#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
build_directory="${TMPDIR:-/tmp}/wrf-rs-netcdf-benchmark"
repetitions="${1:-1000}"
mkdir -p "${build_directory}"

cc $(nc-config --cflags) -O3 \
  "${repo_root}/parity/netcdf/minimal_wrf_restart.c" \
  $(nc-config --libs) \
  -o "${build_directory}/minimal_wrf_restart"
cargo build --quiet --release --manifest-path "${repo_root}/Cargo.toml" \
  -p wrf-io --examples

echo "NetCDF-C: ${repetitions} complete minimal restart writes"
/usr/bin/time -p "${build_directory}/minimal_wrf_restart" \
  "${build_directory}/netcdf-c-restart.nc" "${repetitions}"

echo "Rust: ${repetitions} complete minimal restart writes"
/usr/bin/time -p "${repo_root}/target/release/examples/minimal_restart_oracle" \
  write-repeat "${build_directory}/rust-restart.nc" "${repetitions}"

field_repetitions="${FIELD_REPETITIONS:-25}"
cc $(nc-config --cflags) -O3 \
  "${repo_root}/parity/netcdf/netcdf_field_benchmark.c" \
  $(nc-config --libs) \
  -o "${build_directory}/netcdf_field_benchmark"

echo "NetCDF-C: ${field_repetitions} writes of one 16 MiB field"
/usr/bin/time -p "${build_directory}/netcdf_field_benchmark" \
  "${build_directory}/netcdf-c-field.nc" "${field_repetitions}"

echo "Rust: ${field_repetitions} writes of one 16 MiB field"
/usr/bin/time -p "${repo_root}/target/release/examples/netcdf_field_benchmark" \
  "${build_directory}/rust-field.nc" "${field_repetitions}"
