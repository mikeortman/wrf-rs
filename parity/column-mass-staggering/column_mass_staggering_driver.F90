program column_mass_staggering_driver
  use iso_fortran_env, only: int32
  implicit none

  integer, parameter :: ims = 0, ime = 5
  integer, parameter :: jms = 0, jme = 4
  integer, parameter :: ids = 0, ide = 5
  integer, parameter :: jds = 0, jde = 4
  integer, parameter :: kds = 1, kde = 2
  integer, parameter :: kms = 1, kme = 1
  integer, parameter :: its = 1, ite = 4
  integer, parameter :: jts = 1, jte = 3
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
      write (*, '(A,1X,I0,1X,I0,1X,Z8.8)') 'west_east', i, j, &
                                             transfer(muu(i, j), 0_int32)
    end do
  end do
  do j = jms, jme
    do i = ims, ime
      write (*, '(A,1X,I0,1X,I0,1X,Z8.8)') 'south_north', i, j, &
                                             transfer(muv(i, j), 0_int32)
    end do
  end do
end program column_mass_staggering_driver
