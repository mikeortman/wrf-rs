program kessler_driver
  use module_mp_kessler, only: kessler
  implicit none

  integer, parameter :: ims = -1, ime = 4
  integer, parameter :: jms = -2, jme = 2
  integer, parameter :: kms = 1, kme = 5
  integer, parameter :: its = 0, ite = 3
  integer, parameter :: jts = -1, jte = 1
  integer, parameter :: kts = 1, kte = 5
  integer, parameter :: ids = 0, ide = 4
  integer, parameter :: jds = -1, jde = 2
  integer, parameter :: kds = 1, kde = 6
  real, parameter :: dt = 60.0
  real, parameter :: xlv = 2.5e6
  real, parameter :: cp = 7.0 * 287.0 / 2.0
  real, parameter :: ep2 = 287.0 / 461.6
  real, parameter :: svp1 = 0.6112
  real, parameter :: svp2 = 17.67
  real, parameter :: svp3 = 29.65
  real, parameter :: svpt0 = 273.15
  real, parameter :: rhowater = 1000.0
  real :: t(ims:ime, kms:kme, jms:jme)
  real :: qv(ims:ime, kms:kme, jms:jme)
  real :: qc(ims:ime, kms:kme, jms:jme)
  real :: qr(ims:ime, kms:kme, jms:jme)
  real :: rho(ims:ime, kms:kme, jms:jme)
  real :: pii(ims:ime, kms:kme, jms:jme)
  real :: z(ims:ime, kms:kme, jms:jme)
  real :: dz8w(ims:ime, kms:kme, jms:jme)
  real :: rainnc(ims:ime, jms:jme)
  real :: rainncv(ims:ime, jms:jme)
  integer :: i, j, k, io, jo, ko

  do j = jms, jme
    jo = j - jms
    do k = kms, kme
      ko = k - kms
      do i = ims, ime
        io = i - ims
        t(i,k,j) = 278.0 + 0.7 * real(io) + 0.3 * real(ko) - 0.4 * real(jo)
        qv(i,k,j) = 0.002 + 0.001 * real(mod(io + 2 * ko, 8))
        if (mod(io + ko, 3) == 0) then
          qc(i,k,j) = 0.002
        else
          qc(i,k,j) = 0.0002
        end if
        select case (mod(io + jo, 4))
        case (0)
          qr(i,k,j) = 0.0
        case (1)
          qr(i,k,j) = 0.0005
        case (2)
          qr(i,k,j) = 0.005
        case default
          qr(i,k,j) = 0.02
        end select
        rho(i,k,j) = 1.15 - 0.08 * real(ko) + 0.01 * real(io)
        pii(i,k,j) = 0.99 - 0.015 * real(ko) + 0.002 * real(jo)
        z(i,k,j) = 50.0 + 150.0 * real(ko) + 2.0 * real(io)
        dz8w(i,k,j) = 150.0 + 0.5 * real(io)
      end do
    end do
  end do

  do j = jms, jme
    jo = j - jms
    do i = ims, ime
      io = i - ims
      rainnc(i,j) = 10.0 + 0.25 * real(io) + 0.5 * real(jo)
      rainncv(i,j) = -777.0
    end do
  end do

  call kessler(t, qv, qc, qr, rho, pii, dt, z, xlv, cp, &
               ep2, svp1, svp2, svp3, svpt0, rhowater, dz8w, &
               rainnc, rainncv, ids, ide, jds, jde, kds, kde, &
               ims, ime, jms, jme, kms, kme, &
               its, ite, jts, jte, kts, kte)

  call print_three_dimensional_field('potential_temperature', t)
  call print_three_dimensional_field('water_vapor', qv)
  call print_three_dimensional_field('cloud_water', qc)
  call print_three_dimensional_field('rain_water', qr)
  call print_two_dimensional_field('accumulated_precipitation', rainnc)
  call print_two_dimensional_field('step_precipitation', rainncv)

contains

  subroutine print_three_dimensional_field(label, values)
    character(len=*), intent(in) :: label
    real, intent(in) :: values(ims:ime, kms:kme, jms:jme)
    integer :: field_index, field_i, field_j, field_k

    field_index = 0
    do field_j = jms, jme
      do field_k = kms, kme
        do field_i = ims, ime
          write(*,'(A,1X,I0,1X,Z8.8)') trim(label), field_index, &
               transfer(values(field_i,field_k,field_j), 0)
          field_index = field_index + 1
        end do
      end do
    end do
  end subroutine print_three_dimensional_field

  subroutine print_two_dimensional_field(label, values)
    character(len=*), intent(in) :: label
    real, intent(in) :: values(ims:ime, jms:jme)
    integer :: field_index, field_i, field_j

    field_index = 0
    do field_j = jms, jme
      do field_i = ims, ime
        write(*,'(A,1X,I0,1X,Z8.8)') trim(label), field_index, &
             transfer(values(field_i,field_j), 0)
        field_index = field_index + 1
      end do
    end do
  end subroutine print_two_dimensional_field

end program kessler_driver
