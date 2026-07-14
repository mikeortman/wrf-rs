program positive_definite_benchmark
  use iso_fortran_env, only: int64, real64
  use module_positive_definite, only: positive_definite_sheet, positive_definite_slab
  implicit none

  integer, parameter :: west_east_points = 256
  integer, parameter :: line_count = 4096
  integer, parameter :: bottom_top_points = 64
  integer, parameter :: south_north_points = line_count / bottom_top_points
  integer, parameter :: sample_count = 31
  integer, parameter :: calls_per_sample = 32
  integer, parameter :: warmup_call_count = 100
  real, allocatable :: sheet_template(:, :), sheet_field(:, :), line_totals(:)
  real, allocatable :: slab_template(:, :, :), slab_field(:, :, :)
  integer(int64) :: start_count, end_count, elapsed_count, clock_rate
  real(real64) :: milliseconds_per_call, checksum
  integer :: copy_index, i, j, k, line_index, sample
  real :: offset

  call system_clock(count_rate=clock_rate)
  call benchmark_sheet()
  call benchmark_slab()

contains

  subroutine benchmark_sheet()
    allocate(sheet_template(west_east_points, line_count))
    allocate(sheet_field(west_east_points, line_count))
    allocate(line_totals(line_count))
    do j = 1, line_count
      line_index = j - 1
      offset = real(line_index) * 1.0e-6
      do i = 1, west_east_points
        sheet_template(i, j) = 0.01 + real(i - 1) * 0.001 + offset
      end do
      sheet_template(mod(line_index, west_east_points) + 1, j) = -0.001 - offset
      line_totals(j) = sum(sheet_template(:, j))
    end do

    checksum = 0.0_real64
    do copy_index = 1, warmup_call_count
      sheet_field = sheet_template
      call positive_definite_sheet( &
          sheet_field, line_totals, west_east_points, line_count)
      checksum = checksum + sum(real(sheet_field, real64))
    end do
    write (*, '(A,ES24.16)') 'sheet_warmup_checksum ', checksum

    checksum = 0.0_real64
    do sample = 1, sample_count
      elapsed_count = 0_int64
      do copy_index = 1, calls_per_sample
        sheet_field = sheet_template
        call system_clock(start_count)
        call positive_definite_sheet( &
            sheet_field, line_totals, west_east_points, line_count)
        call system_clock(end_count)
        elapsed_count = elapsed_count + end_count - start_count
        checksum = checksum + sum(real(sheet_field, real64))
      end do
      milliseconds_per_call = real(elapsed_count, real64) * 1000.0_real64 / &
                              real(clock_rate, real64) / real(calls_per_sample, real64)
      write (*, '(A,I0,A,F12.6)') 'sheet_sample_', sample, '_milliseconds_per_call ', &
                                 milliseconds_per_call
    end do
    write (*, '(A,ES24.16)') 'sheet_checksum ', checksum
    deallocate(sheet_template, sheet_field, line_totals)
  end subroutine benchmark_sheet

  subroutine benchmark_slab()
    allocate(slab_template(0:west_east_points-1, 0:bottom_top_points-1, &
                           0:south_north_points-1))
    allocate(slab_field(0:west_east_points-1, 0:bottom_top_points-1, &
                        0:south_north_points-1))
    do j = 0, south_north_points - 1
      do k = 0, bottom_top_points - 1
        line_index = j * bottom_top_points + k
        offset = real(line_index) * 1.0e-6
        do i = 0, west_east_points - 1
          slab_template(i, k, j) = 0.01 + real(i) * 0.001 + offset
        end do
        slab_template(mod(line_index, west_east_points), k, j) = -0.001 - offset
      end do
    end do

    checksum = 0.0_real64
    do copy_index = 1, warmup_call_count
      slab_field = slab_template
      call positive_definite_slab( &
          slab_field, &
          0, west_east_points, 0, south_north_points, 0, bottom_top_points, &
          0, west_east_points - 1, 0, south_north_points - 1, &
          0, bottom_top_points - 1, &
          0, west_east_points - 1, 0, south_north_points - 1, &
          0, bottom_top_points)
      checksum = checksum + sum(real(slab_field, real64))
    end do
    write (*, '(A,ES24.16)') 'slab_warmup_checksum ', checksum

    checksum = 0.0_real64
    do sample = 1, sample_count
      elapsed_count = 0_int64
      do copy_index = 1, calls_per_sample
        slab_field = slab_template
        call system_clock(start_count)
        call positive_definite_slab( &
            slab_field, &
            0, west_east_points, 0, south_north_points, 0, bottom_top_points, &
            0, west_east_points - 1, 0, south_north_points - 1, &
            0, bottom_top_points - 1, &
            0, west_east_points - 1, 0, south_north_points - 1, &
            0, bottom_top_points)
        call system_clock(end_count)
        elapsed_count = elapsed_count + end_count - start_count
        checksum = checksum + sum(real(slab_field, real64))
      end do
      milliseconds_per_call = real(elapsed_count, real64) * 1000.0_real64 / &
                              real(clock_rate, real64) / real(calls_per_sample, real64)
      write (*, '(A,I0,A,F12.6)') 'slab_sample_', sample, '_milliseconds_per_call ', &
                                 milliseconds_per_call
    end do
    write (*, '(A,ES24.16)') 'slab_checksum ', checksum
    deallocate(slab_template, slab_field)
  end subroutine benchmark_slab

end program positive_definite_benchmark
