program randomized_held_suarez_driver
  use ieee_arithmetic, only: ieee_is_nan
  use iso_fortran_env, only: int32
  use module_damping_em, only: held_suarez_damp
  implicit none

  character(len=1024) :: corpus_path
  integer :: case_count, case_index, corpus_unit, seed
  integer :: ids, ide, jds, jde, kds, kde
  integer :: ims, ime, jms, jme, kms, kme
  integer :: its, ite, jts, jte, kts, kte
  real, allocatable :: ru_tend(:, :, :), rv_tend(:, :, :)
  real, allocatable :: ru(:, :, :), rv(:, :, :), p(:, :, :), pb(:, :, :)

  call get_command_argument(1, corpus_path)
  if (len_trim(corpus_path) == 0) error stop 'missing Held-Suarez corpus path'
  open(newunit=corpus_unit, file=trim(corpus_path), status='old', action='read')
  read(corpus_unit, *) case_count

  do case_index = 1, case_count
    read(corpus_unit, *) seed, ids, ide, jds, jde, kds, kde, &
        ims, ime, jms, jme, kms, kme, its, ite, jts, jte, kts, kte
    allocate(ru_tend(ims:ime, kms:kme, jms:jme))
    allocate(rv_tend(ims:ime, kms:kme, jms:jme))
    allocate(ru(ims:ime, kms:kme, jms:jme))
    allocate(rv(ims:ime, kms:kme, jms:jme))
    allocate(p(ims:ime, kms:kme, jms:jme))
    allocate(pb(ims:ime, kms:kme, jms:jme))
    call read_field(corpus_unit, ru_tend, ims, ime, kms, kme, jms, jme)
    call read_field(corpus_unit, rv_tend, ims, ime, kms, kme, jms, jme)
    call read_field(corpus_unit, ru, ims, ime, kms, kme, jms, jme)
    call read_field(corpus_unit, rv, ims, ime, kms, kme, jms, jme)
    call read_field(corpus_unit, p, ims, ime, kms, kme, jms, jme)
    call read_field(corpus_unit, pb, ims, ime, kms, kme, jms, jme)

    call held_suarez_damp(ru_tend, rv_tend, ru, rv, p, pb, &
        ids, ide, jds, jde, kds, kde, ims, ime, jms, jme, kms, kme, &
        its, ite, jts, jte, kts, kte)
    call print_field(seed, 'west_east_tendency', ru_tend, ims, ime, kms, kme, jms, jme)
    call print_field(seed, 'south_north_tendency', rv_tend, ims, ime, kms, kme, jms, jme)
    deallocate(ru_tend, rv_tend, ru, rv, p, pb)
  end do

  close(corpus_unit)

contains

  subroutine read_field(unit_number, field, ims, ime, kms, kme, jms, jme)
    integer, intent(in) :: unit_number, ims, ime, kms, kme, jms, jme
    real, intent(out) :: field(ims:ime, kms:kme, jms:jme)
    integer :: i, j, k
    integer(int32) :: input_bits

    do j = jms, jme
      do k = kms, kme
        do i = ims, ime
          read(unit_number, *) input_bits
          field(i, k, j) = transfer(input_bits, field(i, k, j))
        end do
      end do
    end do
  end subroutine read_field

  subroutine print_field(case_seed, field_name, field, ims, ime, kms, kme, jms, jme)
    integer, intent(in) :: case_seed, ims, ime, kms, kme, jms, jme
    character(len=*), intent(in) :: field_name
    real, intent(in) :: field(ims:ime, kms:kme, jms:jme)
    integer :: i, j, k, output_index

    output_index = 0
    do j = jms, jme
      do k = kms, kme
        do i = ims, ime
          call print_value(case_seed, field_name, output_index, field(i, k, j))
          output_index = output_index + 1
        end do
      end do
    end do
  end subroutine print_field

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

end program randomized_held_suarez_driver
