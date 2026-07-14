program microphysics_driver_kessler
  use iso_fortran_env, only: int32
  use module_mp_kessler, only: kessler
  implicit none

  call run_case('disabled', 0, .true., .false., 1, 1, 1, &
                [0], [6], [-1], [4], 1, 2, 3, .false.)
  call run_case('two_tile_specified', 1, .true., .false., 1, 2, 2, &
                [0, 0], [6, 6], [-1, 1], [0, 4], 1, 2, 3, .false.)
  call run_case('channel_switch', 1, .true., .true., 1, 1, 1, &
                [0], [6], [-1], [4], 1, 2, 3, .false.)
  call run_case('partial_and_inactive', 1, .true., .false., 2, 1, 2, &
                [0, 0], [6, 3], [-1, 0], [-1, 3], 1, 2, 3, .false.)
  call run_case('open_boundaries', 1, .false., .false., 1, 1, 1, &
                [0], [6], [-1], [4], 1, 2, 3, .false.)
  call run_case('reordered_species', 1, .true., .false., 1, 1, 1, &
                [0], [6], [-1], [4], 2, 1, 3, .false.)
  call run_case('exceptional', 1, .true., .false., 1, 1, 1, &
                [0], [6], [-1], [4], 1, 2, 3, .true.)

contains

  subroutine run_case(name, mp_physics, specified, channel_switch, spec_zone, ncalls, num_tiles, &
                      i_start, i_end, j_start, j_end, p_qv, p_qc, p_qr, exceptional)
    character(len=*), intent(in) :: name
    logical, intent(in) :: specified, channel_switch, exceptional
    integer, intent(in) :: mp_physics, spec_zone, ncalls, num_tiles, p_qv, p_qc, p_qr
    integer, intent(in) :: i_start(num_tiles), i_end(num_tiles)
    integer, intent(in) :: j_start(num_tiles), j_end(num_tiles)
    integer, parameter :: ims = -1, ime = 6
    integer, parameter :: jms = -2, jme = 4
    integer, parameter :: kms = 1, kme = 5
    integer, parameter :: ids = 0, ide = 6
    integer, parameter :: jds = -1, jde = 4
    integer, parameter :: kds = 1, kde = 6
    integer, parameter :: num_species = 3
    real, parameter :: dt = 60.0
    real, parameter :: xlv = 2.5e6
    real, parameter :: cp = 7.0 * 287.0 / 2.0
    real, parameter :: ep2 = 287.0 / 461.6
    real, parameter :: svp1 = 0.6112
    real, parameter :: svp2 = 17.67
    real, parameter :: svp3 = 29.65
    real, parameter :: svpt0 = 273.15
    real, parameter :: rhowater = 1000.0
    real :: th(ims:ime, kms:kme, jms:jme)
    real :: moist(ims:ime, kms:kme, jms:jme, num_species)
    real :: rho(ims:ime, kms:kme, jms:jme)
    real :: pii(ims:ime, kms:kme, jms:jme)
    real :: z(ims:ime, kms:kme, jms:jme)
    real :: dz8w(ims:ime, kms:kme, jms:jme)
    real :: rainnc(ims:ime, jms:jme)
    real :: rainncv(ims:ime, jms:jme)
    logical :: channel
    integer :: sz, call_index, ij, its, ite, jts, jte, kts, kte
    integer :: i, j, k, io, jo, ko, n

    do j = jms, jme
      jo = j - jms
      do k = kms, kme
        ko = k - kms
        do i = ims, ime
          io = i - ims
          th(i,k,j) = 278.0 + 0.7 * real(io) + 0.3 * real(ko) - 0.4 * real(jo)
          moist(i,k,j,p_qv) = 0.002 + 0.001 * real(mod(io + 2 * ko, 8))
          if (mod(io + ko, 3) == 0) then
            moist(i,k,j,p_qc) = 0.002
          else
            moist(i,k,j,p_qc) = 0.0002
          end if
          select case (mod(io + jo, 4))
          case (0)
            moist(i,k,j,p_qr) = 0.0
          case (1)
            moist(i,k,j,p_qr) = 0.0005
          case (2)
            moist(i,k,j,p_qr) = 0.005
          case default
            moist(i,k,j,p_qr) = 0.02
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
    if (exceptional) then
      th(2 + ims, 1 + kms, 2 + jms) = quiet_nan()
      moist(3 + ims, 0 + kms, 3 + jms, p_qv) = positive_infinity()
      moist(4 + ims, 2 + kms, 2 + jms, p_qr) = quiet_nan()
      moist(0 + ims, 0 + kms, 0 + jms, p_qc) = quiet_nan()
    end if

    ! Replicate module_microphysics_driver.F's preamble before dispatch.
    if (mp_physics /= 0) then
      channel = channel_switch
      if (specified) then
        sz = spec_zone
      else
        sz = 0
      end if
      kts = kds
      kte = min(kde - 1, kme)

      do call_index = 1, ncalls
        do ij = 1, num_tiles
          if (channel) then
            its = max(i_start(ij), ids)
            ite = min(min(i_end(ij), ide - 1), ide - 1)
          else
            its = max(i_start(ij), ids + sz)
            ite = min(min(i_end(ij), ide - 1), ide - 1 - sz)
          end if
          jts = max(j_start(ij), jds + sz)
          jte = min(min(j_end(ij), jde - 1), jde - 1 - sz)
          call kessler(T=th &
                      ,QV=moist(:,:,:,p_qv) &
                      ,QC=moist(:,:,:,p_qc) &
                      ,QR=moist(:,:,:,p_qr) &
                      ,RHO=rho, PII=pii, DT_IN=dt, Z=z, XLV=xlv, CP=cp &
                      ,EP2=ep2, SVP1=svp1, SVP2=svp2 &
                      ,SVP3=svp3, SVPT0=svpt0, RHOWATER=rhowater &
                      ,DZ8W=dz8w &
                      ,RAINNC=rainnc, RAINNCV=rainncv &
                      ,IDS=ids, IDE=ide, JDS=jds, JDE=jde, KDS=kds, KDE=kde &
                      ,IMS=ims, IME=ime, JMS=jms, JME=jme, KMS=kms, KME=kme &
                      ,ITS=its, ITE=ite, JTS=jts, JTE=jte, KTS=kts, KTE=kte)
        end do
      end do
    end if

    call print_three_dimensional_field(name, 'theta', th)
    do n = 1, num_species
      call print_three_dimensional_field(name, 'moist_'//achar(48 + n), moist(:,:,:,n))
    end do
    call print_two_dimensional_field(name, 'rainnc', rainnc)
    call print_two_dimensional_field(name, 'rainncv', rainncv)
  end subroutine run_case

  subroutine print_three_dimensional_field(case_name, label, values)
    character(len=*), intent(in) :: case_name, label
    real, intent(in) :: values(:,:,:)
    integer :: field_index, field_i, field_j, field_k

    field_index = 0
    do field_j = 1, size(values, 3)
      do field_k = 1, size(values, 2)
        do field_i = 1, size(values, 1)
          write(*,'(A,".",A,1X,I0,1X,Z8.8)') trim(case_name), trim(label), &
               field_index, transfer(values(field_i,field_k,field_j), 0)
          field_index = field_index + 1
        end do
      end do
    end do
  end subroutine print_three_dimensional_field

  subroutine print_two_dimensional_field(case_name, label, values)
    character(len=*), intent(in) :: case_name, label
    real, intent(in) :: values(:,:)
    integer :: field_index, field_i, field_j

    field_index = 0
    do field_j = 1, size(values, 2)
      do field_i = 1, size(values, 1)
        write(*,'(A,".",A,1X,I0,1X,Z8.8)') trim(case_name), trim(label), &
             field_index, transfer(values(field_i,field_j), 0)
        field_index = field_index + 1
      end do
    end do
  end subroutine print_two_dimensional_field

  real function quiet_nan()
    quiet_nan = transfer(int(z'7FC00000', int32), 1.0)
  end function quiet_nan

  real function positive_infinity()
    positive_infinity = transfer(int(z'7F800000', int32), 1.0)
  end function positive_infinity

end program microphysics_driver_kessler
