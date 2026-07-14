program momentum_coupling_benchmark
  use iso_fortran_env, only: int64, real64
  implicit none

  integer, parameter :: active_west_east_mass_points = 256
  integer, parameter :: active_south_north_mass_points = 256
  integer, parameter :: active_half_levels = 40
  integer, parameter :: ims = 0, ime = active_west_east_mass_points + 1
  integer, parameter :: jms = 0, jme = active_south_north_mass_points + 1
  integer, parameter :: kms = 0, kme = active_half_levels + 1
  integer, parameter :: ids = 1, ide = active_west_east_mass_points + 1
  integer, parameter :: jds = 1, jde = active_south_north_mass_points + 1
  integer, parameter :: kds = 1, kde = active_half_levels + 1
  integer, parameter :: its = ids, ite = ide
  integer, parameter :: jts = jds, jte = jde
  integer, parameter :: kts = kds, kte = kde
  integer, parameter :: sample_count = 11
  integer, parameter :: calls_per_sample = 40
  integer, parameter :: warmup_call_count = 20
  real, allocatable :: muu(:, :), muv(:, :), mut(:, :)
  real, allocatable :: msfu(:, :), msfv(:, :), msfv_inv(:, :), msft(:, :)
  real, allocatable :: u(:, :, :), v(:, :, :), w(:, :, :)
  real, allocatable :: ru(:, :, :), rv(:, :, :), rw(:, :, :)
  real, allocatable :: c1h(:), c2h(:), c1f(:), c2f(:)
  integer(int64) :: start_count, end_count, clock_rate
  real(real64) :: milliseconds_per_call, checksum
  integer :: i, j, k, call_index, sample

  allocate(muu(ims:ime, jms:jme), muv(ims:ime, jms:jme), mut(ims:ime, jms:jme))
  allocate(msfu(ims:ime, jms:jme), msfv(ims:ime, jms:jme))
  allocate(msfv_inv(ims:ime, jms:jme), msft(ims:ime, jms:jme))
  allocate(u(ims:ime, kms:kme, jms:jme), v(ims:ime, kms:kme, jms:jme))
  allocate(w(ims:ime, kms:kme, jms:jme), ru(ims:ime, kms:kme, jms:jme))
  allocate(rv(ims:ime, kms:kme, jms:jme), rw(ims:ime, kms:kme, jms:jme))
  allocate(c1h(kms:kme), c2h(kms:kme), c1f(kms:kme), c2f(kms:kme))

  do k = kms, kme
    c1h(k) = 0.7 + real(k) * 0.003
    c2h(k) = 1.5 - real(k) * 0.001
    c1f(k) = 0.6 + real(k) * 0.002
    c2f(k) = 2.0 + real(k) * 0.0015
  end do
  do j = jms, jme
    do i = ims, ime
      muu(i, j) = 80.0 + real(i) * 0.005 + real(j) * 0.0125
      muv(i, j) = 85.0 + real(i) * 0.0075 - real(j) * 0.005
      mut(i, j) = 90.0 + real(i) * 0.0025 + real(j) * 0.008
      msfu(i, j) = 1.0 + real(i) * 0.0001 + real(j) * 0.00005
      msfv(i, j) = 1.0
      msfv_inv(i, j) = 1.0 / (1.1 + real(i) * 0.00008 + real(j) * 0.00004)
      msft(i, j) = 0.9 + real(i) * 0.00006 + real(j) * 0.00003
      do k = kms, kme
        u(i, k, j) = -3.0 + real(i) * 0.002 + real(k) * 0.003 + real(j) * 0.001
        v(i, k, j) = 2.0 - real(i) * 0.0015 + real(k) * 0.0025 - real(j) * 0.0005
        w(i, k, j) = 0.5 + real(i) * 0.0008 - real(k) * 0.0012 + real(j) * 0.0007
      end do
    end do
  end do
  ru = -999.0
  rv = -999.0
  rw = -999.0

  do call_index = 1, warmup_call_count
    call apply_coupling()
  end do

  call system_clock(count_rate=clock_rate)
  do sample = 1, sample_count
    call system_clock(start_count)
    do call_index = 1, calls_per_sample
      call apply_coupling()
    end do
    call system_clock(end_count)
    milliseconds_per_call = real(end_count - start_count, real64) * 1000.0_real64 / &
                            real(clock_rate, real64) / real(calls_per_sample, real64)
    write (*, '(A,I0,A,F12.6)') 'sample_', sample, '_milliseconds_per_call ', &
                               milliseconds_per_call
  end do

  checksum = sum(real(ru(its:ite, kts:kde-1, jts:jde-1), real64)) + &
             sum(real(rv(its:ide-1, kts:kde-1, jts:jte), real64)) + &
             sum(real(rw(its:ide-1, kts:kte, jts:jde-1), real64))
  write (*, '(A,I0)') 'momentum_outputs_per_call ', &
    (active_west_east_mass_points + 1) * active_south_north_mass_points * &
    active_half_levels + active_west_east_mass_points * &
    (active_south_north_mass_points + 1) * active_half_levels + &
    active_west_east_mass_points * active_south_north_mass_points * &
    (active_half_levels + 1)
  write (*, '(A,ES24.16)') 'checksum ', checksum

contains

  subroutine apply_coupling()
    call couple_momentum(muu, ru, u, msfu, muv, rv, v, msfv, msfv_inv, &
                         mut, rw, w, msft, c1h, c2h, c1f, c2f, &
                         ids, ide, jds, jde, kds, kde, &
                         ims, ime, jms, jme, kms, kme, &
                         its, ite, jts, jte, kts, kte)
  end subroutine apply_coupling

end program momentum_coupling_benchmark
