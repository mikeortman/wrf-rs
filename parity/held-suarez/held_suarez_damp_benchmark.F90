program held_suarez_damp_benchmark
  use iso_fortran_env, only: int64, real64
  use module_damping_em, only: held_suarez_damp
  implicit none

  integer, parameter :: active_west_east_points = 256
  integer, parameter :: active_bottom_top_points = 64
  integer, parameter :: active_south_north_points = 64
  integer, parameter :: sample_count = 31
  integer, parameter :: iteration_count = 500
  integer, parameter :: warmup_iteration_count = 100
  real, allocatable :: ru_tend(:, :, :), rv_tend(:, :, :)
  real, allocatable :: ru(:, :, :), rv(:, :, :), p(:, :, :), pb(:, :, :)
  integer(int64) :: start_count, end_count, clock_rate
  real(real64) :: milliseconds_per_call
  real(real64) :: checksum
  integer :: i, j, k, iteration, sample

  allocate(ru_tend(0:257, 0:64, 0:65), rv_tend(0:257, 0:64, 0:65))
  allocate(ru(0:257, 0:64, 0:65), rv(0:257, 0:64, 0:65))
  allocate(p(0:257, 0:64, 0:65), pb(0:257, 0:64, 0:65))

  do j = 0, 65
    do k = 0, 64
      do i = 0, 257
        p(i, k, j) = real(i) * 0.125 + real(j) * 0.25
        pb(i, k, j) = 100500.0 - real(k) * 500.0
        ru(i, k, j) = 10.0 + p(i, k, j) * 0.01
        rv(i, k, j) = -7.0 + p(i, k, j) * 0.02
        ru_tend(i, k, j) = 0.001
        rv_tend(i, k, j) = -0.002
      end do
    end do
  end do

  do iteration = 1, warmup_iteration_count
    call apply_damping()
  end do

  call system_clock(count_rate=clock_rate)
  do sample = 1, sample_count
    call system_clock(start_count)
    do iteration = 1, iteration_count
      call apply_damping()
    end do
    call system_clock(end_count)
    milliseconds_per_call = real(end_count - start_count, real64) * 1000.0_real64 / &
                            real(clock_rate, real64) / real(iteration_count, real64)
    write (*, '(A,I0,A,F12.6)') 'sample_', sample, '_milliseconds_per_call ', &
                               milliseconds_per_call
  end do

  checksum = sum(real(ru_tend, real64)) + sum(real(rv_tend, real64))
  write (*, '(A,I0)') 'momentum_updates_per_call ', &
                       2 * active_west_east_points * active_bottom_top_points * &
                       active_south_north_points
  write (*, '(A,ES24.16)') 'checksum ', checksum

contains

  subroutine apply_damping()
    call held_suarez_damp( &
        ru_tend, rv_tend, ru, rv, p, pb, &
        0, 257, 0, 66, 1, 65, &
        0, 257, 0, 65, 0, 64, &
        1, 256, 1, 64, 1, 64)
  end subroutine apply_damping

end program held_suarez_damp_benchmark
