#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
build_directory="${TMPDIR:-/tmp}/wrf-rs-netcdf-oracle"
mkdir -p "${build_directory}"

wrf_netcdf_source="${repo_root}/upstream/WRF/external/io_netcdf/wrf_io.F90"
fortran_routines="${build_directory}/wrf_schema_order_routines.F90"
{
  sed -n '/^subroutine GetDim(/,/^end subroutine GetDim/p' "${wrf_netcdf_source}"
  sed -n '/^subroutine ExtOrder(/,/^end subroutine ExtOrder/p' "${wrf_netcdf_source}"
  sed -n '/^subroutine ExtOrderStr(/,/^end subroutine ExtOrderStr/p' "${wrf_netcdf_source}"
  sed -n '/^subroutine LowerCase(/,/^end subroutine LowerCase/p' "${wrf_netcdf_source}"
  sed -n '/^subroutine reorder (/,/^end subroutine reorder/p' "${wrf_netcdf_source}"
} > "${fortran_routines}"

gfortran -cpp -O2 \
  -J "${build_directory}" \
  -I "${repo_root}/upstream/WRF/external/ioapi_share" \
  "${repo_root}/parity/netcdf/wrf_data_stub.F90" \
  "${fortran_routines}" \
  "${repo_root}/parity/netcdf/wrf_schema_order_driver.F90" \
  -o "${build_directory}/wrf_schema_order_driver"

LC_ALL=C "${build_directory}/wrf_schema_order_driver" \
  > "${build_directory}/fortran-schema-summary.txt"
cargo run --quiet --release --manifest-path "${repo_root}/Cargo.toml" \
  -p wrf-io --example minimal_restart_oracle -- schema-summary \
  > "${build_directory}/rust-schema-summary.txt"
diff -u "${build_directory}/fortran-schema-summary.txt" \
  "${build_directory}/rust-schema-summary.txt"

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

echo "Pinned Fortran ordering and Registry-selected NetCDF-C/Rust restart parity passed."
