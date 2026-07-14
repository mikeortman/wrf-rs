program moisture_coefficient_driver
  use iso_fortran_env, only: int32
  implicit none

  call run_case('interior', 0, 2, 5, 6, 1, 3, 4, .false.)
  call run_case('x_upper', -1, 4, 5, 6, 1, 3, 4, .false.)
  call run_case('y_upper', 0, 2, 4, 8, 1, 3, 4, .false.)
  call run_case('all_upper', -1, 4, 4, 8, 1, 4, 4, .false.)
  call run_case('no_active_species', -1, 4, 4, 8, 1, 4, 1, .false.)
  call run_case('one_active_species', -1, 4, 4, 8, 1, 4, 2, .false.)
  call run_case('exceptional', -1, 4, 4, 8, 1, 4, 4, .true.)

contains

  subroutine run_case(case_name, its, ite, jts, jte, kts, kte, n_moist, exceptional)
    character(len=*), intent(in) :: case_name
    integer, intent(in) :: its, ite, jts, jte, kts, kte, n_moist
    logical, intent(in) :: exceptional
    integer, parameter :: ims = -2, ime = 5, jms = 3, jme = 9
    integer, parameter :: kms = -1, kme = 5
    integer, parameter :: ids = -1, ide = 4, jds = 4, jde = 8
    integer, parameter :: kds = 1, kde = 4
    real, allocatable :: moist(:, :, :, :)
    real :: cqu(ims:ime, kms:kme, jms:jme)
    real :: cqv(ims:ime, kms:kme, jms:jme)
    real :: cqw(ims:ime, kms:kme, jms:jme)
    integer :: i, j, k, species

    allocate(moist(ims:ime, kms:kme, jms:jme, n_moist))
    do species = 1, n_moist
      do j = jms, jme
        do k = kms, kme
          do i = ims, ime
            if (species == 1) then
              moist(i, k, j, species) = 700.0 + real(i) - real(k) + real(j)
            else
              moist(i, k, j, species) = real(species) * 0.001 + &
                real(i) * 0.0001 - real(k) * 0.0002 + real(j) * 0.00005
            end if
          end do
        end do
      end do
    end do

    if (exceptional) then
      moist(ids, kds, jds, 2) = huge(moist)
      moist(ids - 1, kds, jds, 2) = huge(moist)
      moist(ids, kds, jds - 1, 3) = -huge(moist)
      moist(ids, kds + 1, jds, 4) = huge(moist)
      moist(ids, kds, jds + 1, 4) = -huge(moist)
      moist(ids + 1, kds, jds, 2) = -0.0
    end if

    cqu = -901.0
    cqv = -902.0
    cqw = -903.0
    call calc_cq(moist, cqu, cqv, cqw, n_moist, &
                 ids, ide, jds, jde, kds, kde, &
                 ims, ime, jms, jme, kms, kme, &
                 its, ite, jts, jte, kts, kte)

    call write_output(case_name, 'cqu', cqu)
    call write_output(case_name, 'cqv', cqv)
    call write_output(case_name, 'cqw', cqw)
    deallocate(moist)
  end subroutine run_case

  subroutine write_output(case_name, field_name, values)
    character(len=*), intent(in) :: case_name, field_name
    real, intent(in) :: values(-2:5, -1:5, 3:9)
    integer :: i, j, k

    do j = 3, 9
      do k = -1, 5
        do i = -2, 5
          write (*, '(A,1X,A,1X,I0,1X,I0,1X,I0,1X,Z8.8)') &
            case_name, field_name, i, k, j, transfer(values(i, k, j), 0_int32)
        end do
      end do
    end do
  end subroutine write_output

end program moisture_coefficient_driver
