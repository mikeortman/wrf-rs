program momentum_coupling_driver
  use iso_fortran_env, only: int32
  implicit none

  call run_case('interior', 0, 1, 5, 6, 1, 3, .false.)
  call run_case('x_upper', -1, 3, 5, 6, 1, 3, .false.)
  call run_case('y_upper', 0, 1, 4, 7, 1, 3, .false.)
  call run_case('z_upper', 0, 1, 5, 6, 1, 4, .false.)
  call run_case('all_upper', -1, 3, 4, 7, 1, 4, .false.)
  call run_case('exceptional_finite', -1, 3, 4, 7, 1, 4, .true.)

contains

  subroutine run_case(case_name, its, ite, jts, jte, kts, kte, exceptional)
    character(len=*), intent(in) :: case_name
    integer, intent(in) :: its, ite, jts, jte, kts, kte
    logical, intent(in) :: exceptional
    integer, parameter :: ims = -2, ime = 4, jms = 3, jme = 8
    integer, parameter :: kms = 0, kme = 4
    integer, parameter :: ids = -1, ide = 3, jds = 4, jde = 7
    integer, parameter :: kds = 1, kde = 4
    real :: muu(ims:ime, jms:jme), muv(ims:ime, jms:jme)
    real :: mut(ims:ime, jms:jme)
    real :: msfu(ims:ime, jms:jme), msfv(ims:ime, jms:jme)
    real :: msfv_inv(ims:ime, jms:jme), msft(ims:ime, jms:jme)
    real :: u(ims:ime, kms:kme, jms:jme)
    real :: v(ims:ime, kms:kme, jms:jme)
    real :: w(ims:ime, kms:kme, jms:jme)
    real :: ru(ims:ime, kms:kme, jms:jme)
    real :: rv(ims:ime, kms:kme, jms:jme)
    real :: rw(ims:ime, kms:kme, jms:jme)
    real :: c1h(kms:kme), c2h(kms:kme), c1f(kms:kme), c2f(kms:kme)

    call initialize_fields(muu, muv, mut, msfu, msfv, msfv_inv, msft, &
                           u, v, w, c1h, c2h, c1f, c2f)
    if (exceptional) then
      u(ids, kds, jds) = huge(u)
      v(ids, kds, jds) = huge(v)
      w(ids, kds, jds) = huge(w)
      c1h(kds) = 2.0
      c1f(kds) = 2.0
      msfu(ids + 1, jds) = 0.0
      msfv_inv(ids + 1, jds) = 0.0
      msft(ids + 1, jds) = 0.0
    end if
    ru = -999.0
    rv = -999.0
    rw = -999.0

    call couple_momentum(muu, ru, u, msfu, muv, rv, v, msfv, msfv_inv, &
                         mut, rw, w, msft, c1h, c2h, c1f, c2f, &
                         ids, ide, jds, jde, kds, kde, &
                         ims, ime, jms, jme, kms, kme, &
                         its, ite, jts, jte, kts, kte)
    call write_output(case_name, 'west_east', ru)
    call write_output(case_name, 'south_north', rv)
    call write_output(case_name, 'vertical', rw)
  end subroutine run_case

  subroutine initialize_fields(muu, muv, mut, msfu, msfv, msfv_inv, msft, &
                               u, v, w, c1h, c2h, c1f, c2f)
    integer, parameter :: ims = -2, ime = 4, jms = 3, jme = 8
    integer, parameter :: kms = 0, kme = 4
    real, intent(out) :: muu(ims:ime, jms:jme), muv(ims:ime, jms:jme)
    real, intent(out) :: mut(ims:ime, jms:jme)
    real, intent(out) :: msfu(ims:ime, jms:jme), msfv(ims:ime, jms:jme)
    real, intent(out) :: msfv_inv(ims:ime, jms:jme), msft(ims:ime, jms:jme)
    real, intent(out) :: u(ims:ime, kms:kme, jms:jme)
    real, intent(out) :: v(ims:ime, kms:kme, jms:jme)
    real, intent(out) :: w(ims:ime, kms:kme, jms:jme)
    real, intent(out) :: c1h(kms:kme), c2h(kms:kme)
    real, intent(out) :: c1f(kms:kme), c2f(kms:kme)
    integer :: i, j, k

    do k = kms, kme
      c1h(k) = 0.7 + real(k) * 0.03
      c2h(k) = 1.5 - real(k) * 0.1
      c1f(k) = 0.6 + real(k) * 0.02
      c2f(k) = 2.0 + real(k) * 0.15
    end do
    do j = jms, jme
      do i = ims, ime
        muu(i, j) = 80.0 + real(i) * 0.5 + real(j) * 1.25
        muv(i, j) = 85.0 + real(i) * 0.75 - real(j) * 0.5
        mut(i, j) = 90.0 + real(i) * 0.25 + real(j) * 0.8
        msfu(i, j) = 1.0 + real(i - ims) * 0.01 + real(j - jms) * 0.005
        msfv(i, j) = 1.0
        msfv_inv(i, j) = 1.0 / &
          (1.1 + real(i - ims) * 0.008 + real(j - jms) * 0.004)
        msft(i, j) = 0.9 + real(i - ims) * 0.006 + real(j - jms) * 0.003
        do k = kms, kme
          u(i, k, j) = -3.0 + real(i) * 0.2 + real(k) * 0.3 + real(j) * 0.1
          v(i, k, j) = 2.0 - real(i) * 0.15 + real(k) * 0.25 - real(j) * 0.05
          w(i, k, j) = 0.5 + real(i) * 0.08 - real(k) * 0.12 + real(j) * 0.07
        end do
      end do
    end do
  end subroutine initialize_fields

  subroutine write_output(case_name, field_name, field)
    character(len=*), intent(in) :: case_name, field_name
    real, intent(in) :: field(-2:4, 0:4, 3:8)
    integer :: i, j, k

    do j = 3, 8
      do k = 0, 4
        do i = -2, 4
          write (*, '(A,1X,A,1X,I0,1X,I0,1X,I0,1X,Z8.8)') &
            case_name, field_name, i, k, j, transfer(field(i, k, j), 0_int32)
        end do
      end do
    end do
  end subroutine write_output
end program momentum_coupling_driver
