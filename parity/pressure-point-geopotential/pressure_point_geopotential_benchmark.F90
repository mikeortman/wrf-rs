program pressure_point_geopotential_benchmark
  use iso_fortran_env, only: int64, real64
  implicit none

  integer, parameter :: active_west_east_points = 256
  integer, parameter :: active_south_north_points = 256
  integer, parameter :: active_bottom_top_points = 40
  integer, parameter :: ims = 0, ime = active_west_east_points + 1
  integer, parameter :: jms = 0, jme = active_south_north_points + 1
  integer, parameter :: kms = 0, kme = active_bottom_top_points + 1
  integer, parameter :: ids = 1, ide = active_west_east_points + 1
  integer, parameter :: jds = 1, jde = active_south_north_points + 1
  integer, parameter :: kds = 1, kde = active_bottom_top_points + 1
  integer, parameter :: its = ids, ite = ide
  integer, parameter :: jts = jds, jte = jde
  integer, parameter :: kts = kds, kte = kde
  integer, parameter :: sample_count = 11
  integer, parameter :: calls_per_sample = 50
  integer, parameter :: warmup_call_count = 20
  real, allocatable :: php(:, :, :), ph(:, :, :), phb(:, :, :)
  integer(int64) :: start_count, end_count, clock_rate
  real(real64) :: milliseconds_per_call, checksum
  integer :: i, j, k, call_index, sample

  allocate(php(ims:ime, kms:kme, jms:jme))
  allocate(ph(ims:ime, kms:kme, jms:jme))
  allocate(phb(ims:ime, kms:kme, jms:jme))

  do j = jms, jme
    do k = kms, kme
      do i = ims, ime
        ph(i, k, j) = ((125.0 + real(i) * 0.003125) - &
                       real(k) * 0.00175) + real(j) * 0.000875
        phb(i, k, j) = ((875.0 - real(i) * 0.002375) + &
                        real(k) * 0.001125) - real(j) * 0.000625
      end do
    end do
  end do
  php = -999.0

  do call_index = 1, warmup_call_count
    call apply_geopotential()
  end do

  call system_clock(count_rate=clock_rate)
  do sample = 1, sample_count
    call system_clock(start_count)
    do call_index = 1, calls_per_sample
      call apply_geopotential()
    end do
    call system_clock(end_count)
    milliseconds_per_call = real(end_count - start_count, real64) * 1000.0_real64 / &
                            real(clock_rate, real64) / real(calls_per_sample, real64)
    write (*, '(A,I0,A,F12.6)') 'sample_', sample, '_milliseconds_per_call ', &
                               milliseconds_per_call
  end do

  checksum = sum(real(php(ids:ide-1, kds:kde-1, jds:jde-1), real64))
  write (*, '(A,I0)') 'pressure_point_outputs_per_call ', &
    active_west_east_points * active_south_north_points * active_bottom_top_points
  write (*, '(A,ES24.16)') 'checksum ', checksum

contains

  subroutine apply_geopotential()
    call calc_php(php, ph, phb, &
                  ids, ide, jds, jde, kds, kde, &
                  ims, ime, jms, jme, kms, kme, &
                  its, ite, jts, jte, kts, kte)
  end subroutine apply_geopotential

end program pressure_point_geopotential_benchmark
