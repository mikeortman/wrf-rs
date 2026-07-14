program moisture_coefficient_benchmark
  use iso_fortran_env, only: int64, real64
  implicit none

  integer, parameter :: active_west_east_mass_points = 256
  integer, parameter :: active_south_north_mass_points = 256
  integer, parameter :: active_half_levels = 40
  integer, parameter :: n_moist = 7
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
  real, allocatable :: moist(:, :, :, :)
  real, allocatable :: cqu(:, :, :), cqv(:, :, :), cqw(:, :, :)
  integer(int64) :: start_count, end_count, clock_rate
  real(real64) :: milliseconds_per_call, checksum
  integer :: i, j, k, species, call_index, sample

  allocate(moist(ims:ime, kms:kme, jms:jme, n_moist))
  allocate(cqu(ims:ime, kms:kme, jms:jme))
  allocate(cqv(ims:ime, kms:kme, jms:jme))
  allocate(cqw(ims:ime, kms:kme, jms:jme))

  do species = 1, n_moist
    do j = jms, jme
      do k = kms, kme
        do i = ims, ime
          if (species == 1) then
            moist(i, k, j, species) = -777.0
          else
            moist(i, k, j, species) = real(species - 1) * 0.0005 + &
              real(i) * 0.000001 - real(k) * 0.0000005 + &
              real(j) * 0.00000025
          end if
        end do
      end do
    end do
  end do
  cqu = -901.0
  cqv = -902.0
  cqw = -903.0

  do call_index = 1, warmup_call_count
    call apply_coefficients()
  end do

  call system_clock(count_rate=clock_rate)
  do sample = 1, sample_count
    call system_clock(start_count)
    do call_index = 1, calls_per_sample
      call apply_coefficients()
    end do
    call system_clock(end_count)
    milliseconds_per_call = real(end_count - start_count, real64) * 1000.0_real64 / &
                            real(clock_rate, real64) / real(calls_per_sample, real64)
    write (*, '(A,I0,A,F12.6)') 'sample_', sample, '_milliseconds_per_call ', &
                               milliseconds_per_call
  end do

  checksum = sum(real(cqu(ids:ide, kds:kde-1, jds:jde-1), real64)) + &
             sum(real(cqv(ids:ide-1, kds:kde-1, jds:jde), real64)) + &
             sum(real(cqw(ids:ide-1, kds+1:kde-1, jds:jde-1), real64))
  write (*, '(A,I0)') 'coefficient_outputs_per_call ', &
    (active_west_east_mass_points + 1) * active_south_north_mass_points * active_half_levels + &
    active_west_east_mass_points * (active_south_north_mass_points + 1) * active_half_levels + &
    active_west_east_mass_points * active_south_north_mass_points * (active_half_levels - 1)
  write (*, '(A,ES24.16)') 'checksum ', checksum

contains

  subroutine apply_coefficients()
    call calc_cq(moist, cqu, cqv, cqw, n_moist, &
                 ids, ide, jds, jde, kds, kde, &
                 ims, ime, jms, jme, kms, kme, &
                 its, ite, jts, jte, kts, kte)
  end subroutine apply_coefficients

end program moisture_coefficient_benchmark
