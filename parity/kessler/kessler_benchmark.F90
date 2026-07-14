program kessler_benchmark
  use iso_fortran_env, only: int64, real64
  use module_mp_kessler, only: kessler
  implicit none

  integer, parameter :: west_east_points = 128
  integer, parameter :: south_north_points = 128
  integer, parameter :: bottom_top_points = 40
  integer, parameter :: ims = 1, ime = west_east_points
  integer, parameter :: jms = 1, jme = south_north_points
  integer, parameter :: kms = 1, kme = bottom_top_points
  integer, parameter :: its = ims, ite = ime
  integer, parameter :: jts = jms, jte = jme
  integer, parameter :: kts = kms, kte = kme
  integer, parameter :: ids = ims, ide = ime + 1
  integer, parameter :: jds = jms, jde = jme + 1
  integer, parameter :: kds = kms, kde = kme + 1
  integer, parameter :: sample_count = 11
  integer, parameter :: calls_per_sample = 5
  integer, parameter :: warmup_call_count = 3
  real, parameter :: dt = 60.0
  real, parameter :: xlv = 2.5e6
  real, parameter :: cp = 7.0 * 287.0 / 2.0
  real, parameter :: ep2 = 287.0 / 461.6
  real, parameter :: svp1 = 0.6112
  real, parameter :: svp2 = 17.67
  real, parameter :: svp3 = 29.65
  real, parameter :: svpt0 = 273.15
  real, parameter :: rhowater = 1000.0
  real, allocatable :: t(:,:,:), qv(:,:,:), qc(:,:,:), qr(:,:,:)
  real, allocatable :: initial_t(:,:,:), initial_qv(:,:,:)
  real, allocatable :: initial_qc(:,:,:), initial_qr(:,:,:)
  real, allocatable :: rho(:,:,:), pii(:,:,:), z(:,:,:), dz8w(:,:,:)
  real, allocatable :: rainnc(:,:), rainncv(:,:)
  real, allocatable :: initial_rainnc(:,:), initial_rainncv(:,:)
  integer(int64) :: start_count, end_count, clock_rate, elapsed_count
  real(real64) :: milliseconds_per_call, checksum
  integer :: i, j, k, call_index, sample

  allocate(t(ims:ime,kms:kme,jms:jme), qv(ims:ime,kms:kme,jms:jme))
  allocate(qc(ims:ime,kms:kme,jms:jme), qr(ims:ime,kms:kme,jms:jme))
  allocate(initial_t(ims:ime,kms:kme,jms:jme), initial_qv(ims:ime,kms:kme,jms:jme))
  allocate(initial_qc(ims:ime,kms:kme,jms:jme), initial_qr(ims:ime,kms:kme,jms:jme))
  allocate(rho(ims:ime,kms:kme,jms:jme), pii(ims:ime,kms:kme,jms:jme))
  allocate(z(ims:ime,kms:kme,jms:jme), dz8w(ims:ime,kms:kme,jms:jme))
  allocate(rainnc(ims:ime,jms:jme), rainncv(ims:ime,jms:jme))
  allocate(initial_rainnc(ims:ime,jms:jme), initial_rainncv(ims:ime,jms:jme))

  do j = jms, jme
    do k = kms, kme
      do i = ims, ime
        initial_t(i,k,j) = 278.0 + 0.007 * real(i-1) + 0.03 * real(k-1) - &
                           0.004 * real(j-1)
        initial_qv(i,k,j) = 0.002 + 0.001 * real(mod((i-1) + 2 * (k-1), 8))
        if (mod((i-1) + (k-1), 3) == 0) then
          initial_qc(i,k,j) = 0.002
        else
          initial_qc(i,k,j) = 0.0002
        end if
        select case (mod((i-1) + (j-1), 4))
        case (0)
          initial_qr(i,k,j) = 0.0
        case (1)
          initial_qr(i,k,j) = 0.0005
        case (2)
          initial_qr(i,k,j) = 0.005
        case default
          initial_qr(i,k,j) = 0.02
        end select
        rho(i,k,j) = 1.15 - 0.008 * real(k-1)
        pii(i,k,j) = 0.99 - 0.0015 * real(k-1)
        z(i,k,j) = 50.0 + 150.0 * real(k-1)
        dz8w(i,k,j) = 150.0
      end do
    end do
  end do
  initial_rainnc = 10.0
  initial_rainncv = 0.0

  do call_index = 1, warmup_call_count
    call reset_mutable_fields()
    call apply_kessler()
  end do

  call system_clock(count_rate=clock_rate)
  do sample = 1, sample_count
    elapsed_count = 0_int64
    do call_index = 1, calls_per_sample
      call reset_mutable_fields()
      call system_clock(start_count)
      call apply_kessler()
      call system_clock(end_count)
      elapsed_count = elapsed_count + end_count - start_count
    end do
    milliseconds_per_call = real(elapsed_count, real64) * 1000.0_real64 / &
                            real(clock_rate, real64) / real(calls_per_sample, real64)
    write (*, '(A,I0,A,F12.6)') 'sample_', sample, '_milliseconds_per_call ', &
                               milliseconds_per_call
  end do

  checksum = sum(real(t, real64)) + sum(real(qv, real64)) + sum(real(qc, real64)) + &
             sum(real(qr, real64)) + sum(real(rainnc, real64)) + &
             sum(real(rainncv, real64))
  write (*, '(A,I0)') 'grid_points_per_call ', &
                       west_east_points * south_north_points * bottom_top_points
  write (*, '(A,ES24.16)') 'checksum ', checksum

contains

  subroutine reset_mutable_fields()
    t = initial_t
    qv = initial_qv
    qc = initial_qc
    qr = initial_qr
    rainnc = initial_rainnc
    rainncv = initial_rainncv
  end subroutine reset_mutable_fields

  subroutine apply_kessler()
    call kessler(t, qv, qc, qr, rho, pii, dt, z, xlv, cp, &
                 ep2, svp1, svp2, svp3, svpt0, rhowater, dz8w, &
                 rainnc, rainncv, ids, ide, jds, jde, kds, kde, &
                 ims, ime, jms, jme, kms, kme, &
                 its, ite, jts, jte, kts, kte)
  end subroutine apply_kessler

end program kessler_benchmark
