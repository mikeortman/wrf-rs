program positive_definite_sheet_driver
  use iso_fortran_env, only: int32
  use module_positive_definite, only: positive_definite_sheet
  implicit none

  call unchanged_case()
  call negative_total_case()
  call redistribute_case()
  call degenerate_case()
  call multiple_lines_case()
  call below_epsilon_case()
  call signed_zero_case()
  call zero_total_case()
  call negative_zero_total_case()

contains

  subroutine print_case(name, field)
    character(len=*), intent(in) :: name
    real, intent(in) :: field(:, :)
    integer(int32) :: bits(size(field))

    bits = transfer(field, bits)
    write (*, '(A,*(1X,Z8.8))') name, bits
  end subroutine print_case

  subroutine unchanged_case()
    real :: field(4, 1), totals(1)
    field(:, 1) = [1.0, 2.0, 0.0, 3.0]
    totals = [6.0]
    call positive_definite_sheet(field, totals, 4, 1)
    call print_case('unchanged', field)
  end subroutine unchanged_case

  subroutine negative_total_case()
    real :: field(4, 1), totals(1)
    field(:, 1) = [-1.0, 2.0, 3.0, 4.0]
    totals = [-1.0]
    call positive_definite_sheet(field, totals, 4, 1)
    call print_case('negative_total', field)
  end subroutine negative_total_case

  subroutine redistribute_case()
    real :: field(4, 1), totals(1)
    field(:, 1) = [-1.0, 1.0, 2.0, 4.0]
    totals = [10.0]
    call positive_definite_sheet(field, totals, 4, 1)
    call print_case('redistribute', field)
  end subroutine redistribute_case

  subroutine degenerate_case()
    real :: field(4, 1), totals(1)
    field(:, 1) = [-1.0, -1.0, -1.0, -1.0]
    totals = [4.0]
    call positive_definite_sheet(field, totals, 4, 1)
    call print_case('degenerate', field)
  end subroutine degenerate_case

  subroutine multiple_lines_case()
    real :: field(3, 4), totals(4)
    field(:, 1) = [1.0, 2.0, 3.0]
    field(:, 2) = [-2.0, 1.0, 4.0]
    field(:, 3) = [-1.0, 0.5, 0.25]
    field(:, 4) = [-3.0, -3.0, -3.0]
    totals = [6.0, 7.0, -1.0, 9.0]
    call positive_definite_sheet(field, totals, 3, 4)
    call print_case('multiple_lines', field)
  end subroutine multiple_lines_case

  subroutine below_epsilon_case()
    real :: field(2, 1), totals(1)
    field(:, 1) = [-1.0e-20, 0.0]
    totals = [1.0]
    call positive_definite_sheet(field, totals, 2, 1)
    call print_case('below_epsilon', field)
  end subroutine below_epsilon_case

  subroutine signed_zero_case()
    real :: field(3, 1), totals(1)
    field(:, 1) = [1.0, -0.0, 2.0]
    totals = [99.0]
    call positive_definite_sheet(field, totals, 3, 1)
    call print_case('signed_zero', field)
  end subroutine signed_zero_case

  subroutine zero_total_case()
    real :: field(2, 1), totals(1)
    field(:, 1) = [-1.0, 2.0]
    totals = [0.0]
    call positive_definite_sheet(field, totals, 2, 1)
    call print_case('zero_total', field)
  end subroutine zero_total_case

  subroutine negative_zero_total_case()
    real :: field(2, 1), totals(1)
    field(:, 1) = [-1.0, 2.0]
    totals = [-0.0]
    call positive_definite_sheet(field, totals, 2, 1)
    call print_case('negative_zero_total', field)
  end subroutine negative_zero_total_case

end program positive_definite_sheet_driver
