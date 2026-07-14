program periodic_column_mass_benchmark
  use extracted_big_step_column_mass, only: calc_mu_uv
  use iso_fortran_env, only: int64, real64
  use module_configure, only: grid_config_rec_type
  implicit none

  integer, parameter :: active_west_east_mass_points = 1024
  integer, parameter :: active_south_north_mass_points = 1024
  integer, parameter :: ims = 0, ime = active_west_east_mass_points + 1
  integer, parameter :: jms = 0, jme = active_south_north_mass_points + 1
  integer, parameter :: ids = 1, ide = active_west_east_mass_points + 1
  integer, parameter :: jds = 1, jde = active_south_north_mass_points + 1
  integer, parameter :: kds = 1, kde = 2
  integer, parameter :: kms = 1, kme = 1
  integer, parameter :: its = ids, ite = ide
  integer, parameter :: jts = jds, jte = jde
  integer, parameter :: kts = 1, kte = 1
  integer, parameter :: sample_count = 31
  integer, parameter :: calls_per_sample = 500
  integer, parameter :: warmup_call_count = 100
  type(grid_config_rec_type) :: config_flags
  real, allocatable :: mu(:, :), mub(:, :), muu(:, :), muv(:, :)
  integer(int64) :: start_count, end_count, clock_rate
  real(real64) :: milliseconds_per_call, checksum
  integer :: i, j, call_index, sample

  config_flags%periodic_x = .true.
  config_flags%periodic_y = .true.
  allocate(mu(ims:ime, jms:jme), mub(ims:ime, jms:jme))
  allocate(muu(ims:ime, jms:jme), muv(ims:ime, jms:jme))
  do j = jms, jme
    do i = ims, ime
      mu(i, j) = real(i) * 0.25 + real(j) * 1.5 - 0.3
      mub(i, j) = 100.0 + real(i) * 0.5 - real(j) * 0.75
    end do
  end do
  muu = -999.0
  muv = -999.0

  do call_index = 1, warmup_call_count
    call apply_staggering()
  end do

  call system_clock(count_rate=clock_rate)
  do sample = 1, sample_count
    call system_clock(start_count)
    do call_index = 1, calls_per_sample
      call apply_staggering()
    end do
    call system_clock(end_count)
    milliseconds_per_call = real(end_count - start_count, real64) * 1000.0_real64 / &
                            real(clock_rate, real64) / real(calls_per_sample, real64)
    write (*, '(A,I0,A,F12.6)') 'sample_', sample, '_milliseconds_per_call ', &
                               milliseconds_per_call
  end do

  checksum = sum(real(muu(its:ite, jts:jde-1), real64)) + &
             sum(real(muv(its:ide-1, jts:jte), real64))
  write (*, '(A,I0)') 'momentum_mass_outputs_per_call ', &
                       (active_west_east_mass_points + 1) * &
                       active_south_north_mass_points + &
                       active_west_east_mass_points * &
                       (active_south_north_mass_points + 1)
  write (*, '(A,ES24.16)') 'checksum ', checksum

contains

  subroutine apply_staggering()
    call calc_mu_uv( &
        config_flags, mu, mub, muu, muv, &
        ids, ide, jds, jde, kds, kde, &
        ims, ime, jms, jme, kms, kme, &
        its, ite, jts, jte, kts, kte)
  end subroutine apply_staggering

end program periodic_column_mass_benchmark
