program randomized_positive_definite_slab_driver
  use ieee_arithmetic, only: ieee_is_nan
  use iso_fortran_env, only: int32
  use module_positive_definite, only: positive_definite_slab
  implicit none

  character(len=1024) :: corpus_path
  integer :: case_count, case_index, corpus_unit, seed
  integer :: ids, ide, jds, jde, kds, kde
  integer :: ims, ime, jms, jme, kms, kme
  integer :: its, ite, jts, jte, kts, kte
  integer :: i, j, k, output_index
  integer(int32) :: input_bits
  real, allocatable :: field(:, :, :)

  call get_command_argument(1, corpus_path)
  if (len_trim(corpus_path) == 0) error stop 'missing slab corpus path'
  open(newunit=corpus_unit, file=trim(corpus_path), status='old', action='read')
  read(corpus_unit, *) case_count

  do case_index = 1, case_count
    read(corpus_unit, *) seed, ids, ide, jds, jde, kds, kde, &
        ims, ime, jms, jme, kms, kme, its, ite, jts, jte, kts, kte
    allocate(field(ims:ime, kms:kme, jms:jme))
    do j = jms, jme
      do k = kms, kme
        do i = ims, ime
          read(corpus_unit, *) input_bits
          field(i, k, j) = transfer(input_bits, field(i, k, j))
        end do
      end do
    end do

    call positive_definite_slab(field, ids, ide, jds, jde, kds, kde, &
        ims, ime, jms, jme, kms, kme, its, ite, jts, jte, kts, kte)
    output_index = 0
    do j = jms, jme
      do k = kms, kme
        do i = ims, ime
          call print_value(seed, 'slab', output_index, field(i, k, j))
          output_index = output_index + 1
        end do
      end do
    end do
    deallocate(field)
  end do

  close(corpus_unit)

contains

  subroutine print_value(case_seed, field_name, value_index, value)
    integer, intent(in) :: case_seed, value_index
    character(len=*), intent(in) :: field_name
    real, intent(in) :: value
    integer(int32) :: value_bits

    if (ieee_is_nan(value)) then
      write(*, '(I0,1X,A,1X,I0,1X,A)') case_seed, field_name, value_index, 'NAN'
    else
      value_bits = transfer(value, value_bits)
      write(*, '(I0,1X,A,1X,I0,1X,Z8.8)') case_seed, field_name, value_index, value_bits
    end if
  end subroutine print_value

end program randomized_positive_definite_slab_driver
