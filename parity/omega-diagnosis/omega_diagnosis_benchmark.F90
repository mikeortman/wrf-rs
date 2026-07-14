program omega_diagnosis_benchmark
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
  integer, parameter :: calls_per_sample = 20
  integer, parameter :: warmup_call_count = 10
  real, allocatable :: u(:, :, :), v(:, :, :), ww(:, :, :)
  real, allocatable :: mup(:, :), mub(:, :)
  real, allocatable :: msftx(:, :), msfty(:, :), msfux(:, :), msfuy(:, :)
  real, allocatable :: msfvx(:, :), msfvx_inv(:, :), msfvy(:, :)
  real, allocatable :: c1h(:), c2h(:), dnw(:)
  integer(int64) :: start_count, end_count, clock_rate
  real(real64) :: milliseconds_per_call, checksum
  integer :: i, j, k, call_index, sample

  allocate(u(ims:ime, kms:kme, jms:jme), v(ims:ime, kms:kme, jms:jme))
  allocate(ww(ims:ime, kms:kme, jms:jme))
  allocate(mup(ims:ime, jms:jme), mub(ims:ime, jms:jme))
  allocate(msftx(ims:ime, jms:jme), msfty(ims:ime, jms:jme))
  allocate(msfux(ims:ime, jms:jme), msfuy(ims:ime, jms:jme))
  allocate(msfvx(ims:ime, jms:jme), msfvx_inv(ims:ime, jms:jme))
  allocate(msfvy(ims:ime, jms:jme))
  allocate(c1h(kms:kme), c2h(kms:kme), dnw(kms:kme))

  do k = kms, kme
    c1h(k) = 0.65 + real(k) * 0.003
    c2h(k) = 1.4 - real(k) * 0.001
    dnw(k) = -0.025 - real(k) * 0.00001
  end do
  do j = jms, jme
    do i = ims, ime
      mup(i, j) = -4.0 + real(i) * 0.002 - real(j) * 0.001
      mub(i, j) = 95.0 + real(i) * 0.003 + real(j) * 0.005
      msftx(i, j) = 0.9 + real(i) * 0.00006 + real(j) * 0.00003
      msfty(i, j) = -101.0
      msfux(i, j) = -202.0
      msfuy(i, j) = 1.0 + real(i) * 0.0001 + real(j) * 0.00005
      msfvx(i, j) = -303.0
      msfvx_inv(i, j) = 1.0 / (1.1 + real(i) * 0.00008 + real(j) * 0.00004)
      msfvy(i, j) = -404.0
      do k = kms, kme
        u(i, k, j) = -3.0 + real(i) * 0.002 + real(k) * 0.003 + real(j) * 0.001
        v(i, k, j) = 2.0 - real(i) * 0.0015 + real(k) * 0.0025 - real(j) * 0.0005
      end do
    end do
  end do
  ww = -999.0

  do call_index = 1, warmup_call_count
    call apply_diagnosis()
  end do

  call system_clock(count_rate=clock_rate)
  do sample = 1, sample_count
    call system_clock(start_count)
    do call_index = 1, calls_per_sample
      call apply_diagnosis()
    end do
    call system_clock(end_count)
    milliseconds_per_call = real(end_count - start_count, real64) * 1000.0_real64 / &
                            real(clock_rate, real64) / real(calls_per_sample, real64)
    write (*, '(A,I0,A,F12.6)') 'sample_', sample, '_milliseconds_per_call ', &
                               milliseconds_per_call
  end do

  checksum = sum(real(ww(ids:ide-1, kds:kde, jds:jde-1), real64))
  write (*, '(A,I0)') 'omega_outputs_per_call ', &
    active_west_east_mass_points * active_south_north_mass_points * &
    (active_half_levels + 1)
  write (*, '(A,ES24.16)') 'checksum ', checksum

contains

  subroutine apply_diagnosis()
    call calc_ww_cp(u, v, mup, mub, c1h, c2h, ww, &
                    0.125, 0.2, msftx, msfty, msfux, msfuy, &
                    msfvx, msfvx_inv, msfvy, dnw, &
                    ids, ide, jds, jde, kds, kde, &
                    ims, ime, jms, jme, kms, kme, &
                    its, ite, jts, jte, kts, kte)
  end subroutine apply_diagnosis

end program omega_diagnosis_benchmark
