program omega_diagnosis_driver
  use iso_fortran_env, only: int32
  implicit none

  call run_case('interior', 0, 2, 5, 6, .false.)
  call run_case('x_upper', -1, 4, 5, 6, .false.)
  call run_case('y_upper', 0, 2, 4, 8, .false.)
  call run_case('all_boundaries', -1, 4, 4, 8, .false.)
  call run_case('exceptional_finite', -1, 4, 4, 8, .true.)

contains

  subroutine run_case(case_name, its, ite, jts, jte, exceptional)
    character(len=*), intent(in) :: case_name
    integer, intent(in) :: its, ite, jts, jte
    logical, intent(in) :: exceptional
    integer, parameter :: ims = -2, ime = 5, jms = 3, jme = 9
    integer, parameter :: kms = -1, kme = 5
    integer, parameter :: ids = -1, ide = 4, jds = 4, jde = 8
    integer, parameter :: kds = 1, kde = 4, kts = 1, kte = 4
    real :: u(ims:ime, kms:kme, jms:jme)
    real :: v(ims:ime, kms:kme, jms:jme)
    real :: mup(ims:ime, jms:jme), mub(ims:ime, jms:jme)
    real :: msftx(ims:ime, jms:jme), msfty(ims:ime, jms:jme)
    real :: msfux(ims:ime, jms:jme), msfuy(ims:ime, jms:jme)
    real :: msfvx(ims:ime, jms:jme), msfvx_inv(ims:ime, jms:jme)
    real :: msfvy(ims:ime, jms:jme)
    real :: c1h(kms:kme), c2h(kms:kme), dnw(kms:kme)
    real :: ww(ims:ime, kms:kme, jms:jme)
    real, parameter :: rdx = 0.125, rdy = 0.2

    call initialize_fields(u, v, mup, mub, msftx, msfty, msfux, msfuy, &
                           msfvx, msfvx_inv, msfvy, c1h, c2h, dnw)
    if (exceptional) then
      u(ids, kds, jds) = huge(u)
      u(ids + 1, kds, jds) = -huge(u)
      v(ids, kds, jds) = huge(v)
      msfuy(ids, jds) = 0.0
      msfvx_inv(ids, jds + 1) = 0.0
      c1h(kds) = 2.0
      dnw(kds) = -2.0
    end if
    ww = -999.0

    call calc_ww_cp(u, v, mup, mub, c1h, c2h, ww, &
                    rdx, rdy, msftx, msfty, msfux, msfuy, &
                    msfvx, msfvx_inv, msfvy, dnw, &
                    ids, ide, jds, jde, kds, kde, &
                    ims, ime, jms, jme, kms, kme, &
                    its, ite, jts, jte, kts, kte)
    call write_output(case_name, ww)
  end subroutine run_case

  subroutine initialize_fields(u, v, mup, mub, msftx, msfty, msfux, msfuy, &
                               msfvx, msfvx_inv, msfvy, c1h, c2h, dnw)
    integer, parameter :: ims = -2, ime = 5, jms = 3, jme = 9
    integer, parameter :: kms = -1, kme = 5
    real, intent(out) :: u(ims:ime, kms:kme, jms:jme)
    real, intent(out) :: v(ims:ime, kms:kme, jms:jme)
    real, intent(out) :: mup(ims:ime, jms:jme), mub(ims:ime, jms:jme)
    real, intent(out) :: msftx(ims:ime, jms:jme), msfty(ims:ime, jms:jme)
    real, intent(out) :: msfux(ims:ime, jms:jme), msfuy(ims:ime, jms:jme)
    real, intent(out) :: msfvx(ims:ime, jms:jme), msfvx_inv(ims:ime, jms:jme)
    real, intent(out) :: msfvy(ims:ime, jms:jme)
    real, intent(out) :: c1h(kms:kme), c2h(kms:kme), dnw(kms:kme)
    integer :: i, j, k

    do k = kms, kme
      c1h(k) = 0.65 + real(k) * 0.03
      c2h(k) = 1.4 - real(k) * 0.08
      dnw(k) = -0.2 - real(k) * 0.015
    end do
    do j = jms, jme
      do i = ims, ime
        mup(i, j) = -4.0 + real(i) * 0.45 - real(j) * 0.2
        mub(i, j) = 95.0 + real(i) * 0.3 + real(j) * 0.75
        msftx(i, j) = 0.9 + real(i - ims) * 0.007 + real(j - jms) * 0.004
        msfty(i, j) = -101.0 - real(i + j)
        msfux(i, j) = -202.0 - real(i - j)
        msfuy(i, j) = 1.1 + real(i - ims) * 0.009 + real(j - jms) * 0.003
        msfvx(i, j) = -303.0 + real(i * j)
        msfvx_inv(i, j) = 1.0 / &
          (1.05 + real(i - ims) * 0.006 + real(j - jms) * 0.005)
        msfvy(i, j) = -404.0 + real(i + 2 * j)
        do k = kms, kme
          u(i, k, j) = -2.5 + real(i) * 0.17 + real(k) * 0.29 + real(j) * 0.11
          v(i, k, j) = 1.75 - real(i) * 0.13 + real(k) * 0.21 - real(j) * 0.07
        end do
      end do
    end do
  end subroutine initialize_fields

  subroutine write_output(case_name, ww)
    character(len=*), intent(in) :: case_name
    real, intent(in) :: ww(-2:5, -1:5, 3:9)
    integer :: i, j, k

    do j = 3, 9
      do k = -1, 5
        do i = -2, 5
          write (*, '(A,1X,I0,1X,I0,1X,I0,1X,Z8.8)') &
            case_name, i, k, j, transfer(ww(i, k, j), 0_int32)
        end do
      end do
    end do
  end subroutine write_output
end program omega_diagnosis_driver
