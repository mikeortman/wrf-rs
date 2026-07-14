program pressure_point_geopotential_driver
  use iso_fortran_env, only: int32
  implicit none

  call run_case('interior', 0, 2, 5, 6, 1, 2, .false.)
  call run_case('x_upper', -1, 4, 5, 6, 1, 2, .false.)
  call run_case('y_upper', 0, 2, 4, 8, 1, 2, .false.)
  call run_case('z_upper', 0, 2, 5, 6, 1, 4, .false.)
  call run_case('all_upper', -1, 4, 4, 8, 1, 4, .false.)
  call run_case('exceptional', -1, 4, 4, 8, 1, 4, .true.)

contains

  subroutine run_case(case_name, its, ite, jts, jte, kts, kte, exceptional)
    character(len=*), intent(in) :: case_name
    integer, intent(in) :: its, ite, jts, jte, kts, kte
    logical, intent(in) :: exceptional
    integer, parameter :: ims = -2, ime = 5, jms = 3, jme = 9
    integer, parameter :: kms = -1, kme = 5
    integer, parameter :: ids = -1, ide = 4, jds = 4, jde = 8
    integer, parameter :: kds = 1, kde = 4
    real :: php(ims:ime, kms:kme, jms:jme)
    real :: ph(ims:ime, kms:kme, jms:jme)
    real :: phb(ims:ime, kms:kme, jms:jme)
    integer :: i, j, k

    do j = jms, jme
      do k = kms, kme
        do i = ims, ime
          ph(i, k, j) = ((125.0 + real(i) * 3.125) - real(k) * 1.75) + &
                        real(j) * 0.875
          phb(i, k, j) = ((875.0 - real(i) * 2.375) + real(k) * 1.125) - &
                         real(j) * 0.625
        end do
      end do
    end do

    if (exceptional) then
      phb(ids, kds, jds) = huge(phb)
      phb(ids, kds + 1, jds) = huge(phb)
      ph(ids, kds, jds) = -huge(ph)
      ph(ids, kds + 1, jds) = -huge(ph)

      phb(ids + 1, kds, jds) = -0.0
      phb(ids + 1, kds + 1, jds) = -0.0
      ph(ids + 1, kds, jds) = -0.0
      ph(ids + 1, kds + 1, jds) = -0.0

      phb(ids + 2, kds, jds) = huge(phb)
      phb(ids + 2, kds, jds) = phb(ids + 2, kds, jds) * 2.0
      phb(ids + 2, kds + 1, jds) = -huge(phb)
      phb(ids + 2, kds + 1, jds) = phb(ids + 2, kds + 1, jds) * 2.0
    end if

    php = -999.0
    call calc_php(php, ph, phb, &
                  ids, ide, jds, jde, kds, kde, &
                  ims, ime, jms, jme, kms, kme, &
                  its, ite, jts, jte, kts, kte)
    call write_output(case_name, php)
  end subroutine run_case

  subroutine write_output(case_name, php)
    character(len=*), intent(in) :: case_name
    real, intent(in) :: php(-2:5, -1:5, 3:9)
    integer :: i, j, k

    do j = 3, 9
      do k = -1, 5
        do i = -2, 5
          if (isnan(php(i, k, j))) then
            write (*, '(A,1X,I0,1X,I0,1X,I0,1X,A)') &
              case_name, i, k, j, 'NAN'
          else
            write (*, '(A,1X,I0,1X,I0,1X,I0,1X,Z8.8)') &
              case_name, i, k, j, transfer(php(i, k, j), 0_int32)
          end if
        end do
      end do
    end do
  end subroutine write_output

end program pressure_point_geopotential_driver
