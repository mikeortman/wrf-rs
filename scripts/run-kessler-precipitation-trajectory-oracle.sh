#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
wrf_root="$repository_root/upstream/WRF"
upstream_module="$wrf_root/phys/module_mp_kessler.F"
big_step_module="$wrf_root/dyn_em/module_big_step_utilities_em.F"
driver="$repository_root/parity/kessler-precipitation-trajectory/kessler_precipitation_trajectory_driver.F90"
source_checksums="$repository_root/parity/kessler-precipitation-trajectory/wrf-v4.7.1.sha256"

for command in cargo gfortran shasum; do
    if ! command -v "$command" >/dev/null 2>&1; then
        echo "missing command required by Kessler precipitation trajectory oracle: $command" >&2
        exit 1
    fi
done
if [ ! -f "$upstream_module" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi
if ! grep -Fq 'commit = "f52c197ed39d12e087d02c50f412d90d418f6186"' \
    "$repository_root/UPSTREAM.toml"; then
    echo "UPSTREAM.toml does not pin the WRF v4.7.1 trajectory-oracle commit" >&2
    exit 1
fi
(
    cd "$wrf_root"
    shasum -a 256 -c "$source_checksums"
) >/dev/null

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-kessler-trajectory.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

{
    echo 'module module_configure'
    echo 'type grid_config_rec_type'
    echo '  integer :: use_theta_m = 1, no_mp_heating = 0'
    echo '  real :: mp_tend_lim = 10.0'
    echo 'end type grid_config_rec_type'
    echo 'end module module_configure'
    echo 'module module_model_constants'
    echo 'real, parameter :: r_d=287.0, r_v=461.6, cp=7.0*r_d/2.0'
    echo 'real, parameter :: rcp=r_d/cp, g=9.81'
    echo 'end module module_model_constants'
    echo 'module module_state_description'
    echo 'integer, parameter :: param_first_scalar=2, p_qv=2'
    echo 'end module module_state_description'
    echo 'module extracted_kessler_precipitation_trajectory'
    echo 'use module_configure, only: grid_config_rec_type'
    echo 'use module_model_constants, only: r_d, r_v, rcp, g'
    echo 'use module_state_description, only: param_first_scalar, p_qv'
    echo 'implicit none'
    echo 'contains'
    sed -n '/^[[:space:]]*SUBROUTINE moist_physics_prep_em[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE moist_physics_prep_em/p' "$big_step_module"
    sed -n '/^[[:space:]]*SUBROUTINE moist_physics_finish_em[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE moist_physics_finish_em/p' "$big_step_module"
    echo 'end module extracted_kessler_precipitation_trajectory'
} >"$build_directory/extracted.F90"

(
    cd "$build_directory"
    gfortran -O0 -ffp-contract=off -cpp -ffree-form -ffree-line-length-none \
        "$build_directory/extracted.F90" "$upstream_module" "$driver" \
        -o "$build_directory/kessler-precipitation-trajectory-fortran"
)
"$build_directory/kessler-precipitation-trajectory-fortran" \
    >"$build_directory/fortran.txt"

cargo run --quiet --release -p wrf-physics \
    --example kessler_precipitation_trajectory_oracle \
    >"$build_directory/rust.txt"
WRF_ORACLE_WORKERS=1 cargo run --quiet --release -p wrf-physics \
    --example kessler_precipitation_trajectory_oracle \
    >"$build_directory/rust-one-worker.txt"

if ! cmp -s "$build_directory/rust-one-worker.txt" "$build_directory/rust.txt"; then
    echo "Kessler precipitation trajectory differs between one and four workers" >&2
    exit 1
fi

# Exceptional values deliberately remain raw: the pinned compiler and Rust
# agree on NaN sign/payload and infinity bits, so no class normalization is
# permitted unless a future, documented toolchain policy requires it.
if ! cmp -s "$build_directory/fortran.txt" "$build_directory/rust.txt"; then
    diff -u "$build_directory/fortran.txt" "$build_directory/rust.txt" | sed -n '1,200p'
    echo "Rust Kessler precipitation trajectory differs from pinned WRF" >&2
    exit 1
fi

line_count=$(wc -l <"$build_directory/fortran.txt" | tr -d ' ')
echo "PASS Kessler precipitation trajectory oracle: $line_count exact raw-bit values, 0 class-normalized values; one and four workers match pinned WRF."
