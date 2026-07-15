#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
wrf_root="$repository_root/upstream/WRF"
parity_directory="$repository_root/parity/registry-backed-arw-trajectory"
driver="$parity_directory/registry_backed_arw_trajectory_benchmark.F90"
checksums="$parity_directory/wrf-v4.7.1.sha256"

for command in gfortran shasum; do
    if ! command -v "$command" >/dev/null 2>&1; then
        echo "missing command required by Registry-backed ARW trajectory benchmark: $command" >&2
        exit 1
    fi
done
if [ ! -f "$wrf_root/dyn_em/module_em.F" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi
if ! grep -Fq 'commit = "f52c197ed39d12e087d02c50f412d90d418f6186"' \
    "$repository_root/UPSTREAM.toml"; then
    echo "UPSTREAM.toml does not pin the WRF v4.7.1 trajectory benchmark commit" >&2
    exit 1
fi
(
    cd "$wrf_root"
    shasum -a 256 -c "$checksums"
) >/dev/null

build_directory=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-registry-arw-benchmark.XXXXXX")
trap 'rm -rf "$build_directory"' EXIT HUP INT TERM

cat >"$build_directory/module_configure.F90" <<'EOF'
module module_configure
  implicit none
  type :: grid_config_rec_type
    logical :: periodic_x=.false.,periodic_y=.false.
    logical :: specified=.false.,nested=.false.,polar=.false.
    logical :: open_xs=.false.,open_xe=.false.,symmetric_xs=.false.,symmetric_xe=.false.
    logical :: open_ys=.false.,open_ye=.false.,symmetric_ys=.false.,symmetric_ye=.false.
    integer :: phi_adv_z=2,damp_opt=0,use_theta_m=1,no_mp_heating=0
    real :: dampcoef=0.0,zdamp=1.0,mp_tend_lim=10.0
  end type grid_config_rec_type
end module module_configure
EOF

cat >"$build_directory/model_stubs.F90" <<'EOF'
module module_model_constants
  real, parameter :: r_d=287.0, r_v=461.6, cp=7.0*r_d/2.0
  real, parameter :: rcp=r_d/cp, g=9.81, cpovcv=cp/(cp-r_d)
end module module_model_constants
module module_state_description
  integer, parameter :: param_first_scalar=2, p_qv=2
end module module_state_description
EOF

big_step="$wrf_root/dyn_em/module_big_step_utilities_em.F"
module_em="$wrf_root/dyn_em/module_em.F"
small_step="$wrf_root/dyn_em/module_small_step_em.F"

{
    echo 'module extracted_big_step_column_mass'
    echo 'use module_configure, only: grid_config_rec_type'
    echo 'implicit none'
    echo 'contains'
    sed -n '/^SUBROUTINE calc_mu_uv /,/^END SUBROUTINE calc_mu_uv$/p' "$big_step"
    sed -n '/^SUBROUTINE calc_mu_uv_1 /,/^END SUBROUTINE calc_mu_uv_1$/p' "$big_step"
    echo 'end module extracted_big_step_column_mass'
} >"$build_directory/extracted_column_mass.F90"

for routine in calculate_full couple_momentum calc_ww_cp calc_cq calc_alt calc_php; do
    sed -n "/^SUBROUTINE $routine /,/^END SUBROUTINE $routine$/p" \
        "$big_step" >"$build_directory/$routine.F90"
done
sed -n '/^SUBROUTINE rk_addtend_dry /,/^END SUBROUTINE rk_addtend_dry/p' \
    "$module_em" >"$build_directory/rk_addtend_dry.F90"

{
    echo 'module extracted_acoustic_trajectory'
    echo 'use module_configure, only: grid_config_rec_type'
    echo 'use module_model_constants'
    echo 'implicit none'
    echo 'contains'
    for routine in small_step_prep small_step_finish calc_p_rho calc_coef_w advance_uv advance_mu_t advance_w sumflux; do
        sed -n "/^SUBROUTINE $routine/,/^END SUBROUTINE $routine/p" "$small_step"
    done
    echo 'end module extracted_acoustic_trajectory'
} >"$build_directory/extracted_acoustic.F90"

{
    echo 'module extracted_kessler_trajectory'
    echo 'use module_configure, only: grid_config_rec_type'
    echo 'use module_model_constants, only: r_d, r_v, rcp, g'
    echo 'use module_state_description, only: param_first_scalar, p_qv'
    echo 'implicit none'
    echo 'contains'
    sed -n '/^[[:space:]]*SUBROUTINE moist_physics_prep_em[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE moist_physics_prep_em/p' "$big_step"
    sed -n '/^[[:space:]]*SUBROUTINE moist_physics_finish_em[[:space:]]*(/,/^[[:space:]]*END SUBROUTINE moist_physics_finish_em/p' "$big_step"
    echo 'end module extracted_kessler_trajectory'
} >"$build_directory/extracted_kessler.F90"

(
    cd "$build_directory"
    gfortran -O3 -flto -cpp -ffree-form -ffree-line-length-none \
        -ffp-contract=off -fno-range-check -DPARAM_FIRST_SCALAR=2 \
        -J"$build_directory" \
        "$build_directory/module_configure.F90" \
        "$build_directory/model_stubs.F90" \
        "$build_directory/extracted_column_mass.F90" \
        "$build_directory/calculate_full.F90" \
        "$build_directory/couple_momentum.F90" \
        "$build_directory/calc_ww_cp.F90" \
        "$build_directory/calc_cq.F90" \
        "$build_directory/calc_alt.F90" \
        "$build_directory/calc_php.F90" \
        "$build_directory/rk_addtend_dry.F90" \
        "$build_directory/extracted_acoustic.F90" \
        "$build_directory/extracted_kessler.F90" \
        "$wrf_root/phys/module_mp_kessler.F" \
        "$driver" -o "$build_directory/benchmark"
)

"$build_directory/benchmark"
