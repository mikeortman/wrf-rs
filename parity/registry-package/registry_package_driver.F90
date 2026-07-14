program registry_package_driver
  use module_state_description
  implicit none

  integer, parameter :: max_domains = 1
  integer, parameter :: case_count = 9

  type :: stream_record
    integer :: stream(1)
  end type stream_record

  type :: model_configuration
    integer :: mp_physics(max_domains)
  end type model_configuration

  type(model_configuration) :: model_config_rec
  type(stream_record) :: moist_streams_table(max_domains, param_num_moist)
  integer :: idomain
  integer :: case_index
  integer :: moist_index_table(param_num_moist, max_domains)
  integer :: moist_num_table(max_domains)
  integer, parameter :: choices(case_count) = [ -9, -5, -4, -3, 0, 1, 2, 4, 5 ]
  logical :: moist_boundary_table(max_domains, param_num_moist)
  character(len=256) :: moist_dname_table(max_domains, param_num_moist)
  character(len=256) :: moist_desc_table(max_domains, param_num_moist)
  character(len=256) :: moist_units_table(max_domains, param_num_moist)

  idomain = 1
  do case_index = 1, case_count
    call reset_case(choices(case_index))
    include 'scalar_indices.inc'
    call print_case(choices(case_index))
  end do

contains

  subroutine reset_case(choice)
    integer, intent(in) :: choice

    model_config_rec%mp_physics = choice
    moist_index_table = 0
    moist_num_table = 1
    moist_streams_table = stream_record(stream=[0])
    moist_boundary_table = .false.
    moist_dname_table = ''
    moist_desc_table = ''
    moist_units_table = ''
  end subroutine reset_case

  subroutine print_case(choice)
    integer, intent(in) :: choice

    write (*, '(A,I0,A,I0,A,I0,A,L1,A,I0,A,I0,A,L1,A,I0,A,I0,A,L1,A,I0)') &
      'CASE|choice=', choice, &
      '|num=', moist_num_table(idomain), &
      '|qv=', p_qv, ':', f_qv, ':', dense_index(p_qv, f_qv), &
      '|qc=', p_qc, ':', f_qc, ':', dense_index(p_qc, f_qc), &
      '|qr=', p_qr, ':', f_qr, ':', dense_index(p_qr, f_qr)
  end subroutine print_case

  integer function dense_index(packed_index, is_active)
    integer, intent(in) :: packed_index
    logical, intent(in) :: is_active

    if (is_active) then
      dense_index = packed_index - param_first_scalar
    else
      dense_index = -1
    end if
  end function dense_index

end program registry_package_driver
