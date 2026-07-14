program runge_kutta_preparation_driver
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_big_step_column_mass, only: calc_mu_uv
  implicit none

  integer, parameter :: ims = -1, ime = 4, jms = -1, jme = 4
  integer, parameter :: kms = 0, kme = 4
  integer, parameter :: ids = 0, ide = 4, jds = 0, jde = 4
  integer, parameter :: kds = 1, kde = 4
  integer, parameter :: its = 1, ite = 3, jts = 1, jte = 3
  integer, parameter :: kts = 1, kte = 4
  integer, parameter :: n_moist = 3
  real, parameter :: sentinel = -9999.0

  type(grid_config_rec_type) :: config_flags
  real :: u(ims:ime, kms:kme, jms:jme)
  real :: v(ims:ime, kms:kme, jms:jme)
  real :: w(ims:ime, kms:kme, jms:jme)
  real :: ph(ims:ime, kms:kme, jms:jme)
  real :: phb(ims:ime, kms:kme, jms:jme)
  real :: al(ims:ime, kms:kme, jms:jme)
  real :: alb(ims:ime, kms:kme, jms:jme)
  real :: moist(ims:ime, kms:kme, jms:jme, n_moist)
  real :: ru(ims:ime, kms:kme, jms:jme)
  real :: rv(ims:ime, kms:kme, jms:jme)
  real :: rw(ims:ime, kms:kme, jms:jme)
  real :: ww(ims:ime, kms:kme, jms:jme)
  real :: cqu(ims:ime, kms:kme, jms:jme)
  real :: cqv(ims:ime, kms:kme, jms:jme)
  real :: cqw(ims:ime, kms:kme, jms:jme)
  real :: alt(ims:ime, kms:kme, jms:jme)
  real :: php(ims:ime, kms:kme, jms:jme)
  real :: mu(ims:ime, jms:jme), mub(ims:ime, jms:jme)
  real :: mut(ims:ime, jms:jme), muu(ims:ime, jms:jme), muv(ims:ime, jms:jme)
  real :: msftx(ims:ime, jms:jme), msfty(ims:ime, jms:jme)
  real :: msfux(ims:ime, jms:jme), msfuy(ims:ime, jms:jme)
  real :: msfvx(ims:ime, jms:jme), msfvx_inv(ims:ime, jms:jme)
  real :: msfvy(ims:ime, jms:jme)
  real :: c1h(kms:kme), c2h(kms:kme), c1f(kms:kme), c2f(kms:kme)
  real :: dnw(kms:kme)
  integer :: i, j, k, species, ii, jj, kk

  config_flags%periodic_x = .false.
  config_flags%periodic_y = .false.

  do j = jms, jme
    jj = j - jms
    do i = ims, ime
      ii = i - ims
      mu(i, j) = (10.0 + real(ii) * 0.25) + real(jj) * 0.5
      mub(i, j) = (90.0 - real(ii) * 0.125) + real(jj) * 0.375
      msftx(i, j) = (1.0 + real(ii) * 0.002) + real(jj) * 0.003
      msfty(i, j) = (1.1 - real(ii) * 0.001) + real(jj) * 0.002
      msfux(i, j) = 1.0
      msfuy(i, j) = (0.9 + real(ii) * 0.0015) - real(jj) * 0.0005
      msfvx(i, j) = 1.0
      msfvx_inv(i, j) = (0.8 - real(ii) * 0.001) + real(jj) * 0.001
      msfvy(i, j) = 1.0
    end do
  end do

  do j = jms, jme
    jj = j - jms
    do k = kms, kme
      kk = k - kms
      do i = ims, ime
        ii = i - ims
        u(i, k, j) = ((1.0 + real(ii) * 0.01) + real(kk) * 0.02) - real(jj) * 0.03
        v(i, k, j) = ((-0.5 + real(ii) * 0.015) - real(kk) * 0.01) + real(jj) * 0.025
        w(i, k, j) = ((0.25 - real(ii) * 0.005) + real(kk) * 0.03) + real(jj) * 0.004
        al(i, k, j) = ((0.2 + real(ii) * 0.001) - real(kk) * 0.002) + real(jj) * 0.0005
        alb(i, k, j) = ((0.8 - real(ii) * 0.0005) + real(kk) * 0.001) - real(jj) * 0.00025
        ph(i, k, j) = ((100.0 + real(ii) * 2.0) + real(kk) * 11.0) - real(jj) * 3.0
        phb(i, k, j) = ((1000.0 - real(ii) * 1.5) + real(kk) * 17.0) + real(jj) * 2.5
        moist(i, k, j, 1) = -777.0
        moist(i, k, j, 2) = ((0.001 + real(ii) * 0.00001) + real(kk) * 0.00002) + real(jj) * 0.00003
        moist(i, k, j, 3) = ((0.002 - real(ii) * 0.000005) + real(kk) * 0.00001) - real(jj) * 0.000004
      end do
    end do
  end do

  do k = kms, kme
    kk = k - kms
    c1h(k) = 1.0 + real(kk) * 0.01
    c2h(k) = 0.1 - real(kk) * 0.005
    c1f(k) = 0.9 + real(kk) * 0.015
    c2f(k) = 0.2 + real(kk) * 0.004
    dnw(k) = -0.25 + real(kk) * 0.002
  end do

  mut = sentinel
  muu = sentinel
  muv = sentinel
  ru = sentinel
  rv = sentinel
  rw = sentinel
  ww = sentinel
  cqu = sentinel
  cqv = sentinel
  cqw = sentinel
  alt = sentinel
  php = sentinel

  call calculate_full(mut, mub, mu, &
                      ids, ide, jds, jde, 1, 2, &
                      ims, ime, jms, jme, 1, 1, &
                      its, ite, jts, jte, 1, 1)
  call calc_mu_uv(config_flags, mu, mub, muu, muv, &
                  ids, ide, jds, jde, kds, kde, &
                  ims, ime, jms, jme, kms, kme, &
                  its, ite, jts, jte, kts, kte)
  call couple_momentum(muu, ru, u, msfuy, &
                       muv, rv, v, msfvx, msfvx_inv, &
                       mut, rw, w, msfty, &
                       c1h, c2h, c1f, c2f, &
                       ids, ide, jds, jde, kds, kde, &
                       ims, ime, jms, jme, kms, kme, &
                       its, ite, jts, jte, kts, kte)
  call calc_ww_cp(u, v, mu, mub, c1h, c2h, ww, &
                  1.0, 1.0, msftx, msfty, &
                  msfux, msfuy, msfvx, msfvx_inv, msfvy, dnw, &
                  ids, ide, jds, jde, kds, kde, &
                  ims, ime, jms, jme, kms, kme, &
                  its, ite, jts, jte, kts, kte)
  call calc_cq(moist, cqu, cqv, cqw, n_moist, &
               ids, ide, jds, jde, kds, kde, &
               ims, ime, jms, jme, kms, kme, &
               its, ite, jts, jte, kts, kte)
  call calc_alt(alt, al, alb, &
                ids, ide, jds, jde, kds, kde, &
                ims, ime, jms, jme, kms, kme, &
                its, ite, jts, jte, kts, kte)
  call calc_php(php, ph, phb, &
                ids, ide, jds, jde, kds, kde, &
                ims, ime, jms, jme, kms, kme, &
                its, ite, jts, jte, kts, kte)

  call write_horizontal('mut', mut)
  call write_horizontal('muu', muu)
  call write_horizontal('muv', muv)
  call write_volume('ru', ru)
  call write_volume('rv', rv)
  call write_volume('rw', rw)
  call write_volume('ww', ww)
  call write_volume('cqu', cqu)
  call write_volume('cqv', cqv)
  call write_volume('cqw', cqw)
  call write_volume('alt', alt)
  call write_volume('php', php)

contains

  subroutine write_horizontal(field_name, field)
    character(len=*), intent(in) :: field_name
    real, intent(in) :: field(ims:ime, jms:jme)
    integer :: i, j, output_index

    output_index = 0
    do j = jms, jme
      do i = ims, ime
        write (*, '(I0,1X,A,1X,I0,1X,Z8.8)') &
          0, field_name, output_index, transfer(field(i, j), 0_int32)
        output_index = output_index + 1
      end do
    end do
  end subroutine write_horizontal

  subroutine write_volume(field_name, field)
    character(len=*), intent(in) :: field_name
    real, intent(in) :: field(ims:ime, kms:kme, jms:jme)
    integer :: i, j, k, output_index

    output_index = 0
    do j = jms, jme
      do k = kms, kme
        do i = ims, ime
          write (*, '(I0,1X,A,1X,I0,1X,Z8.8)') &
            0, field_name, output_index, transfer(field(i, k, j), 0_int32)
          output_index = output_index + 1
        end do
      end do
    end do
  end subroutine write_volume

end program runge_kutta_preparation_driver
