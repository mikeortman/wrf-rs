program kessler_precipitation_trajectory_driver
  ! Direct oracle composition for WRF v4.7.1 (f52c197):
  ! - module_mp_kessler.F is compiled without modification.
  ! - moist_physics_prep_em and moist_physics_finish_em are compiled directly
  !   from module_big_step_utilities_em.F by the oracle runner. Small stubs
  !   replace only generated WRF configuration and constant modules.
  ! - module_microphysics_driver.F:983-1004 and Registry.EM_COMMON:3019 pin
  !   the dispatch and QVAPOR/QCLOUD/QRAIN composition checked by Rust.
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_kessler_precipitation_trajectory, only: &
    moist_physics_prep_em, moist_physics_finish_em
  use module_mp_kessler, only: kessler
  implicit none

  integer, parameter :: nx = 6, nz = 5, nw = 6, ny = 5
  integer, parameter :: ims = -1, ime = 4
  integer, parameter :: jms = -2, jme = 2
  integer, parameter :: kms = 1, kme = 6
  integer, parameter :: its = 0, ite = 3
  integer, parameter :: jts = -1, jte = 1
  integer, parameter :: kts = 1, kte = 6
  integer, parameter :: ids = 0, ide = 4
  integer, parameter :: jds = -1, jde = 2
  integer, parameter :: kds = 1, kde = 6
  integer, parameter :: step_count = 3
  real, parameter :: dt = 60.0
  real, parameter :: t0 = 300.0
  real, parameter :: p0 = 100000.0
  real, parameter :: gravity = 9.81
  real, parameter :: r_d = 287.0
  real, parameter :: r_v = 461.6
  real, parameter :: cp = 7.0 * r_d / 2.0
  real, parameter :: rcp = r_d / cp
  real, parameter :: vapor_ratio = r_v / r_d
  real, parameter :: xlv = 2.5e6
  real, parameter :: ep2 = r_d / r_v
  real, parameter :: svp1 = 0.6112
  real, parameter :: svp2 = 17.67
  real, parameter :: svp3 = 29.65
  real, parameter :: svpt0 = 273.15
  real, parameter :: rhowater = 1000.0
  real, parameter :: maximum_theta_tendency = 0.5

  type :: trajectory_state
    real :: perturbation_theta(nx, nw, ny)
    real :: qv(nx, nw, ny)
    real :: qc(nx, nw, ny)
    real :: qr(nx, nw, ny)
    real :: perturbation_inverse_density(nx, nw, ny)
    real :: base_inverse_density(nx, nw, ny)
    real :: perturbation_pressure(nx, nw, ny)
    real :: base_pressure(nx, nw, ny)
    real :: perturbation_geopotential_w(nx, nw, ny)
    real :: base_geopotential_w(nx, nw, ny)
    real :: rainnc(nx, ny)
    real :: rainncv(nx, ny)
  end type trajectory_state

  type :: trajectory_diagnostics
    real :: full_theta(nx, nw, ny)
    real :: density(nx, nw, ny)
    real :: exner(nx, nw, ny)
    real :: height(nx, nw, ny)
    real :: dz8w(nx, nw, ny)
    real :: pressure_at_w(nx, nw, ny)
    real :: full_pressure(nx, nw, ny)
    real :: height_at_w(nx, nw, ny)
    real :: previous_theta(nx, nw, ny)
    real :: previous_qv(nx, nw, ny)
    real :: previous_qc(nx, nw, ny)
    real :: theta_tendency(nx, nw, ny)
    real :: qv_tendency(nx, nw, ny)
    real :: qc_tendency(nx, nw, ny)
    real :: dry_theta_perturbation(nx, nw, ny)
    real :: column_mass(nx, ny)
  end type trajectory_diagnostics

  type(trajectory_state) :: continuous, restarted, checkpoint
  type(trajectory_diagnostics) :: continuous_diagnostics, restarted_diagnostics
  integer :: step

  call initialize_state(continuous)
  do step = 1, step_count
    call advance_step('moist_heating_full.continuous', step, continuous, &
      continuous_diagnostics, .true., .true., maximum_theta_tendency, &
      its, ite, jts, jte)
  end do
  call emit_state('moist_heating_full.continuous.final', continuous)

  call initialize_state(restarted)
  call advance_step('moist_heating_full.restarted', 1, restarted, &
    restarted_diagnostics, .true., .true., maximum_theta_tendency, &
    its, ite, jts, jte)
  checkpoint = restarted
  call emit_state('moist_heating_full.restarted.checkpoint', checkpoint)

  ! Recreate the trajectory model state from the checkpoint. Scheme scratch is
  ! local to each Kessler call and deliberately is not checkpointed.
  call initialize_state(restarted)
  restarted = checkpoint
  do step = 2, step_count
    call advance_step('moist_heating_full.restarted', step, restarted, &
      restarted_diagnostics, .true., .true., maximum_theta_tendency, &
      its, ite, jts, jte)
  end do
  call emit_state('moist_heating_full.restarted.final', restarted)

  call initialize_state(continuous)
  call advance_step('dry_heating_clamped', 1, continuous, &
    continuous_diagnostics, .false., .true., 0.00001, its, ite, jts, jte)

  call initialize_state(continuous)
  call advance_step('moist_no_heating', 1, continuous, &
    continuous_diagnostics, .true., .false., maximum_theta_tendency, &
    its, ite, jts, jte)

  ! The Rust call also supplies a fully inactive tile before this partial tile.
  ! Domain clipping discards it without calling the scheme.
  call initialize_state(continuous)
  call advance_step('partial_with_inactive_tile', 1, continuous, &
    continuous_diagnostics, .true., .true., maximum_theta_tendency, &
    1, 2, 0, 1)

  call initialize_state(continuous)
  call seed_exceptional_values(continuous)
  call advance_step('exceptional', 1, continuous, continuous_diagnostics, &
    .true., .true., maximum_theta_tendency, its, ite, jts, jte)

contains

  subroutine initialize_state(state)
    type(trajectory_state), intent(out) :: state
    integer :: i, j, k, io, jo, ko
    real :: z_w

    do j = 1, ny
      jo = j - 1
      do k = 1, nw
        ko = k - 1
        do i = 1, nx
          io = i - 1
          state%perturbation_theta(i,k,j) = -20.0 + 0.7 * real(io) &
            + 0.3 * real(ko) - 0.4 * real(jo)
          state%qv(i,k,j) = 0.002 + 0.001 * real(mod(io + 2 * ko, 8))
          if (mod(io + ko, 3) == 0) then
            state%qc(i,k,j) = 0.002
          else
            state%qc(i,k,j) = 0.0002
          end if
          select case (mod(io + jo, 4))
          case (0)
            state%qr(i,k,j) = 0.0
          case (1)
            state%qr(i,k,j) = 0.0005
          case (2)
            state%qr(i,k,j) = 0.005
          case default
            state%qr(i,k,j) = 0.02
          end select
          state%perturbation_inverse_density(i,k,j) = &
            0.03 + 0.002 * real(io) + 0.001 * real(jo)
          state%base_inverse_density(i,k,j) = 0.84 + 0.07 * real(ko)
          state%perturbation_pressure(i,k,j) = &
            120.0 + 3.0 * real(io) - 2.0 * real(jo)
          state%base_pressure(i,k,j) = 92000.0 - 7000.0 * real(ko)
        end do
      end do
      do k = 1, nw
        do i = 1, nx
          io = i - 1
          z_w = 35.0 + 150.0 * real(k - 1) + 2.0 * real(io)
          state%perturbation_geopotential_w(i,k,j) = &
            gravity * (0.25 * real(jo))
          state%base_geopotential_w(i,k,j) = gravity * z_w
        end do
      end do
      do i = 1, nx
        io = i - 1
        state%rainnc(i,j) = 10.0 + 0.25 * real(io) + 0.5 * real(jo)
        state%rainncv(i,j) = -777.0
      end do
    end do
  end subroutine initialize_state

  subroutine seed_exceptional_values(state)
    type(trajectory_state), intent(inout) :: state
    ! Keep exceptional sentinels outside the active tile. WRF's active
    ! MIN/MAX propagation is compiler-dependent across GNU Fortran versions;
    ! inactive storage preservation remains an exact cross-toolchain contract.
    state%perturbation_theta(1,2,3) = quiet_nan()
    state%qv(6,1,4) = positive_infinity()
    state%qr(5,3,5) = quiet_nan()
    state%qc(1,1,1) = quiet_nan()
  end subroutine seed_exceptional_values

  subroutine advance_step(prefix, step_number, state, diagnostics, &
      uses_moist_theta, heating_enabled, maximum_tendency, &
      call_its, call_ite, call_jts, call_jte)
    character(len=*), intent(in) :: prefix
    integer, intent(in) :: step_number
    type(trajectory_state), intent(inout) :: state
    type(trajectory_diagnostics), intent(inout) :: diagnostics
    logical, intent(in) :: uses_moist_theta, heating_enabled
    real, intent(in) :: maximum_tendency
    integer, intent(in) :: call_its, call_ite, call_jts, call_jte
    character(len=64) :: stage_prefix
    real :: fzm(nw), fzp(nw)
    type(grid_config_rec_type) :: config

    diagnostics%full_theta = 0.0
    diagnostics%density = 0.0
    diagnostics%exner = 0.0
    diagnostics%height = 0.0
    diagnostics%dz8w = 0.0
    diagnostics%pressure_at_w = 0.0
    diagnostics%full_pressure = 0.0
    diagnostics%height_at_w = 0.0
    diagnostics%previous_theta = 0.0
    diagnostics%previous_qv = 0.0
    diagnostics%previous_qc = 0.0
    diagnostics%theta_tendency = 0.0
    diagnostics%qv_tendency = 0.0
    diagnostics%qc_tendency = 0.0
    diagnostics%dry_theta_perturbation = 0.0
    diagnostics%column_mass = 1.0
    fzm = 0.5
    fzp = 0.5
    config = grid_config_rec_type()
    if (uses_moist_theta) then
      config%use_theta_m = 1
    else
      config%use_theta_m = 0
    end if
    if (heating_enabled) then
      config%no_mp_heating = 0
    else
      config%no_mp_heating = 1
    end if
    config%mp_tend_lim = maximum_tendency

    call moist_physics_prep_em( &
      state%perturbation_theta, diagnostics%previous_theta, t0, &
      diagnostics%density, state%perturbation_inverse_density, &
      state%base_inverse_density, state%perturbation_pressure, &
      diagnostics%pressure_at_w, p0, state%base_pressure, &
      state%perturbation_geopotential_w, state%base_geopotential_w, &
      diagnostics%full_theta, diagnostics%exner, diagnostics%full_pressure, &
      diagnostics%height, diagnostics%height_at_w, diagnostics%dz8w, &
      dt, diagnostics%theta_tendency, state%qv, diagnostics%qv_tendency, &
      state%qc, diagnostics%qc_tendency, config, fzm, fzp, &
      ids, ide, jds, jde, kds, kde, ims, ime, jms, jme, kms, kme, &
      call_its, call_ite, call_jts, call_jte, kts, kte)
    write(stage_prefix, '(A,".step",I0,".prepared")') trim(prefix), step_number
    call emit_prepared(trim(stage_prefix), state, diagnostics)

    call kessler( &
      t=diagnostics%full_theta, qv=state%qv, qc=state%qc, qr=state%qr, &
      rho=diagnostics%density, pii=diagnostics%exner, dt_in=dt, &
      z=diagnostics%height, xlv=xlv, cp=cp, ep2=ep2, svp1=svp1, &
      svp2=svp2, svp3=svp3, svpt0=svpt0, rhowater=rhowater, &
      dz8w=diagnostics%dz8w, rainnc=state%rainnc, rainncv=state%rainncv, &
      ids=ids, ide=ide, jds=jds, jde=jde, kds=kds, kde=kde, &
      ims=ims, ime=ime, jms=jms, jme=jme, kms=kms, kme=kme, &
      its=call_its, ite=call_ite, jts=call_jts, jte=call_jte, &
      kts=kts, kte=nz)
    write(stage_prefix, '(A,".step",I0,".microphysics")') trim(prefix), step_number
    call emit_microphysics(trim(stage_prefix), state, diagnostics)

    call moist_physics_finish_em( &
      state%perturbation_theta, diagnostics%previous_theta, t0, &
      diagnostics%column_mass, diagnostics%full_theta, &
      diagnostics%theta_tendency, dt, state%qv, diagnostics%qv_tendency, &
      state%qc, diagnostics%qc_tendency, diagnostics%dry_theta_perturbation, &
      config, ids, ide, jds, jde, kds, kde, ims, ime, jms, jme, kms, kme, &
      call_its, call_ite, call_jts, call_jte, kts, kte)
    write(stage_prefix, '(A,".step",I0,".finished")') trim(prefix), step_number
    call emit_finished(trim(stage_prefix), state, diagnostics)
  end subroutine advance_step

  subroutine emit_prepared(prefix, state, diagnostics)
    character(len=*), intent(in) :: prefix
    type(trajectory_state), intent(in) :: state
    type(trajectory_diagnostics), intent(in) :: diagnostics
    call emit_volume(prefix, 'perturbation_theta', state%perturbation_theta)
    call emit_volume(prefix, 'qv', state%qv)
    call emit_volume(prefix, 'qc', state%qc)
    call emit_volume(prefix, 'qr', state%qr)
    call emit_volume(prefix, 'full_theta', diagnostics%full_theta)
    call emit_volume(prefix, 'density', diagnostics%density)
    call emit_volume(prefix, 'exner', diagnostics%exner)
    call emit_volume(prefix, 'height', diagnostics%height)
    call emit_volume(prefix, 'dz8w', diagnostics%dz8w)
    call emit_horizontal(prefix, 'rainnc', state%rainnc)
    call emit_horizontal(prefix, 'rainncv', state%rainncv)
  end subroutine emit_prepared

  subroutine emit_microphysics(prefix, state, diagnostics)
    character(len=*), intent(in) :: prefix
    type(trajectory_state), intent(in) :: state
    type(trajectory_diagnostics), intent(in) :: diagnostics
    call emit_volume(prefix, 'full_theta', diagnostics%full_theta)
    call emit_volume(prefix, 'qv', state%qv)
    call emit_volume(prefix, 'qc', state%qc)
    call emit_volume(prefix, 'qr', state%qr)
    call emit_horizontal(prefix, 'rainnc', state%rainnc)
    call emit_horizontal(prefix, 'rainncv', state%rainncv)
  end subroutine emit_microphysics

  subroutine emit_finished(prefix, state, diagnostics)
    character(len=*), intent(in) :: prefix
    type(trajectory_state), intent(in) :: state
    type(trajectory_diagnostics), intent(in) :: diagnostics
    call emit_volume(prefix, 'perturbation_theta', state%perturbation_theta)
    call emit_volume(prefix, 'qv', state%qv)
    call emit_volume(prefix, 'qc', state%qc)
    call emit_volume(prefix, 'qr', state%qr)
    call emit_volume(prefix, 'theta_tendency', diagnostics%theta_tendency)
    call emit_volume(prefix, 'qv_tendency', diagnostics%qv_tendency)
    call emit_volume(prefix, 'qc_tendency', diagnostics%qc_tendency)
    call emit_volume(prefix, 'dry_theta_perturbation', diagnostics%dry_theta_perturbation)
    call emit_horizontal(prefix, 'rainnc', state%rainnc)
    call emit_horizontal(prefix, 'rainncv', state%rainncv)
  end subroutine emit_finished

  subroutine emit_state(prefix, state)
    character(len=*), intent(in) :: prefix
    type(trajectory_state), intent(in) :: state
    call emit_volume(prefix, 'perturbation_theta', state%perturbation_theta)
    call emit_volume(prefix, 'qv', state%qv)
    call emit_volume(prefix, 'qc', state%qc)
    call emit_volume(prefix, 'qr', state%qr)
    call emit_horizontal(prefix, 'rainnc', state%rainnc)
    call emit_horizontal(prefix, 'rainncv', state%rainncv)
  end subroutine emit_state

  subroutine emit_volume(prefix, name, field)
    character(len=*), intent(in) :: prefix, name
    real, intent(in) :: field(nx, nw, ny)
    integer :: field_index, i, j, k
    field_index = 0
    do j = 1, ny
      do k = 1, nz
        do i = 1, nx
          write(*, '(A,".",A,1X,I0,1X,Z8.8)') trim(prefix), trim(name), &
            field_index, transfer(field(i,k,j), 0_int32)
          field_index = field_index + 1
        end do
      end do
    end do
  end subroutine emit_volume

  subroutine emit_horizontal(prefix, name, field)
    character(len=*), intent(in) :: prefix, name
    real, intent(in) :: field(nx, ny)
    integer :: field_index, i, j
    field_index = 0
    do j = 1, ny
      do i = 1, nx
        write(*, '(A,".",A,1X,I0,1X,Z8.8)') trim(prefix), trim(name), &
          field_index, transfer(field(i,j), 0_int32)
        field_index = field_index + 1
      end do
    end do
  end subroutine emit_horizontal

  real function quiet_nan()
    quiet_nan = transfer(int(z'7FC00000', int32), 1.0)
  end function quiet_nan

  real function positive_infinity()
    positive_infinity = transfer(int(z'7F800000', int32), 1.0)
  end function positive_infinity

end program kessler_precipitation_trajectory_driver
