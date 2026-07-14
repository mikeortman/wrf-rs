program inverse_density_driver
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
    real :: alt(ims:ime, kms:kme, jms:jme)
    real :: al(ims:ime, kms:kme, jms:jme)
    real :: alb(ims:ime, kms:kme, jms:jme)
    integer :: i, j, k

    do j = jms, jme
      do k = kms, kme
        do i = ims, ime
          al(i, k, j) = ((0.125 + real(i) * 0.031) - real(k) * 0.017) + &
                        real(j) * 0.009
          alb(i, k, j) = ((0.875 - real(i) * 0.023) + real(k) * 0.011) - &
                         real(j) * 0.007
        end do
      end do
    end do

    if (exceptional) then
      al(ids, kds, jds) = huge(al)
      alb(ids, kds, jds) = huge(alb)
      al(ids + 1, kds, jds) = huge(al)
      alb(ids + 1, kds, jds) = -huge(alb)
      al(ids + 2, kds, jds) = -0.0
      alb(ids + 2, kds, jds) = -0.0
      al(ids + 3, kds, jds) = huge(al)
      al(ids + 3, kds, jds) = al(ids + 3, kds, jds) * 2.0
      alb(ids + 3, kds, jds) = -huge(alb)
      alb(ids + 3, kds, jds) = alb(ids + 3, kds, jds) * 2.0
    end if

    alt = -999.0
    call calc_alt(alt, al, alb, &
                  ids, ide, jds, jde, kds, kde, &
                  ims, ime, jms, jme, kms, kme, &
                  its, ite, jts, jte, kts, kte)
    call write_output(case_name, alt)
  end subroutine run_case

  subroutine write_output(case_name, alt)
    character(len=*), intent(in) :: case_name
    real, intent(in) :: alt(-2:5, -1:5, 3:9)
    integer :: i, j, k

    do j = 3, 9
      do k = -1, 5
        do i = -2, 5
          if (isnan(alt(i, k, j))) then
            write (*, '(A,1X,I0,1X,I0,1X,I0,1X,A)') &
              case_name, i, k, j, 'NAN'
          else
            write (*, '(A,1X,I0,1X,I0,1X,I0,1X,Z8.8)') &
              case_name, i, k, j, transfer(alt(i, k, j), 0_int32)
          end if
        end do
      end do
    end do
  end subroutine write_output

end program inverse_density_driver
