program column_mass_staggering_driver
  use iso_fortran_env, only: int32
  implicit none

  call run_case('interior', 0, 5, 0, 4, 1, 4, 1, 3)
  call run_case('lower',    1, 5, 1, 4, 1, 4, 1, 3)
  call run_case('upper',    0, 4, 0, 3, 1, 4, 1, 3)
  call run_case('both',     1, 4, 1, 3, 1, 4, 1, 3)

contains

  subroutine run_case(case_name, ids, ide, jds, jde, its, ite, jts, jte)
    character(len=*), intent(in) :: case_name
    integer, intent(in) :: ids, ide, jds, jde
    integer, intent(in) :: its, ite, jts, jte
    integer, parameter :: ims = 0, ime = 5
    integer, parameter :: jms = 0, jme = 4
    integer, parameter :: kds = 1, kde = 2
    integer, parameter :: kms = 1, kme = 1
    integer, parameter :: kts = 1, kte = 1
    real :: mu(ims:ime, jms:jme), mub(ims:ime, jms:jme)
    real :: muu(ims:ime, jms:jme), muv(ims:ime, jms:jme)
    integer :: i, j

    do j = jms, jme
      do i = ims, ime
        mu(i, j) = real(i) * 0.25 + real(j) * 1.5 - 0.3
        mub(i, j) = 100.0 + real(i) * 0.5 - real(j) * 0.75
      end do
    end do
    muu = -999.0
    muv = -999.0

    call calc_mu_staggered( &
        mu, mub, muu, muv, &
        ids, ide, jds, jde, kds, kde, &
        ims, ime, jms, jme, kms, kme, &
        its, ite, jts, jte, kts, kte)

    do j = jms, jme
      do i = ims, ime
        write (*, '(A,1X,A,1X,I0,1X,I0,1X,Z8.8)') case_name, 'west_east', i, j, &
                                                       transfer(muu(i, j), 0_int32)
      end do
    end do
    do j = jms, jme
      do i = ims, ime
        write (*, '(A,1X,A,1X,I0,1X,I0,1X,Z8.8)') case_name, 'south_north', i, j, &
                                                       transfer(muv(i, j), 0_int32)
      end do
    end do
  end subroutine run_case
end program column_mass_staggering_driver
