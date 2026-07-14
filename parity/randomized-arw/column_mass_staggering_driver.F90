program randomized_column_mass_staggering_driver
  use ieee_arithmetic, only: ieee_is_nan
  use iso_fortran_env, only: int32
  implicit none

  character(len=1024) :: corpus_path
  integer :: case_count, case_index, corpus_unit, seed
  integer :: ids, ide, jds, jde, kds, kde
  integer :: ims, ime, jms, jme, kms, kme
  integer :: its, ite, jts, jte, kts, kte
  real, allocatable :: mu(:, :), mub(:, :), muu(:, :), muv(:, :)

  call get_command_argument(1, corpus_path)
  if (len_trim(corpus_path) == 0) error stop 'missing column-mass corpus path'
  open(newunit=corpus_unit, file=trim(corpus_path), status='old', action='read')
  read(corpus_unit, *) case_count

  do case_index = 1, case_count
    read(corpus_unit, *) seed, ids, ide, jds, jde, kds, kde, &
        ims, ime, jms, jme, kms, kme, its, ite, jts, jte, kts, kte
    allocate(mu(ims:ime, jms:jme), mub(ims:ime, jms:jme))
    allocate(muu(ims:ime, jms:jme), muv(ims:ime, jms:jme))
    call read_field(corpus_unit, mu, ims, ime, jms, jme)
    call read_field(corpus_unit, mub, ims, ime, jms, jme)
    muu = -999.0
    muv = -999.0

    call calc_mu_staggered(mu, mub, muu, muv, ids, ide, jds, jde, kds, kde, &
        ims, ime, jms, jme, kms, kme, its, ite, jts, jte, kts, kte)
    call print_field(seed, 'west_east_mass', muu, ims, ime, jms, jme)
    call print_field(seed, 'south_north_mass', muv, ims, ime, jms, jme)
    deallocate(mu, mub, muu, muv)
  end do

  close(corpus_unit)

contains

  subroutine read_field(unit_number, field, ims, ime, jms, jme)
    integer, intent(in) :: unit_number, ims, ime, jms, jme
    real, intent(out) :: field(ims:ime, jms:jme)
    integer :: i, j
    integer(int32) :: input_bits

    do j = jms, jme
      do i = ims, ime
        read(unit_number, *) input_bits
        field(i, j) = transfer(input_bits, field(i, j))
      end do
    end do
  end subroutine read_field

  subroutine print_field(case_seed, field_name, field, ims, ime, jms, jme)
    integer, intent(in) :: case_seed, ims, ime, jms, jme
    character(len=*), intent(in) :: field_name
    real, intent(in) :: field(ims:ime, jms:jme)
    integer :: i, j, output_index

    output_index = 0
    do j = jms, jme
      do i = ims, ime
        call print_value(case_seed, field_name, output_index, field(i, j))
        output_index = output_index + 1
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

end program randomized_column_mass_staggering_driver
