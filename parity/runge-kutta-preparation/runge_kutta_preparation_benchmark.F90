program runge_kutta_preparation_benchmark
  use iso_fortran_env, only: int64, real64
  use module_configure, only: grid_config_rec_type
  use extracted_big_step_column_mass, only: calc_mu_uv
  implicit none

  integer, parameter :: active_x = 256, active_y = 256, active_z = 40
  integer, parameter :: ims = 0, ime = active_x + 1
  integer, parameter :: jms = 0, jme = active_y + 1
  integer, parameter :: kms = 0, kme = active_z + 1
  integer, parameter :: ids = 1, ide = active_x + 1
  integer, parameter :: jds = 1, jde = active_y + 1
  integer, parameter :: kds = 1, kde = active_z + 1
  integer, parameter :: its = ids, ite = ide, jts = jds, jte = jde
  integer, parameter :: kts = kds, kte = kde, n_moist = 3
  integer, parameter :: sample_count = 31, calls_per_sample = 20
  integer, parameter :: warmup_call_count = 10
  type(grid_config_rec_type) :: config_flags
  real, allocatable :: u(:,:,:), v(:,:,:), w(:,:,:), ph(:,:,:), phb(:,:,:)
  real, allocatable :: al(:,:,:), alb(:,:,:), moist(:,:,:,:)
  real, allocatable :: ru(:,:,:), rv(:,:,:), rw(:,:,:), ww(:,:,:)
  real, allocatable :: cqu(:,:,:), cqv(:,:,:), cqw(:,:,:), alt(:,:,:), php(:,:,:)
  real, allocatable :: mu(:,:), mub(:,:), mut(:,:), muu(:,:), muv(:,:)
  real, allocatable :: msftx(:,:), msfty(:,:), msfux(:,:), msfuy(:,:)
  real, allocatable :: msfvx(:,:), msfvx_inv(:,:), msfvy(:,:)
  real, allocatable :: c1h(:), c2h(:), c1f(:), c2f(:), dnw(:)
  integer(int64) :: start_count, end_count, clock_rate
  real(real64) :: milliseconds_per_call, checksum
  integer :: call_index, sample

  allocate(u(ims:ime,kms:kme,jms:jme), v(ims:ime,kms:kme,jms:jme))
  allocate(w(ims:ime,kms:kme,jms:jme), ph(ims:ime,kms:kme,jms:jme))
  allocate(phb(ims:ime,kms:kme,jms:jme), al(ims:ime,kms:kme,jms:jme))
  allocate(alb(ims:ime,kms:kme,jms:jme), moist(ims:ime,kms:kme,jms:jme,n_moist))
  allocate(ru(ims:ime,kms:kme,jms:jme), rv(ims:ime,kms:kme,jms:jme))
  allocate(rw(ims:ime,kms:kme,jms:jme), ww(ims:ime,kms:kme,jms:jme))
  allocate(cqu(ims:ime,kms:kme,jms:jme), cqv(ims:ime,kms:kme,jms:jme))
  allocate(cqw(ims:ime,kms:kme,jms:jme), alt(ims:ime,kms:kme,jms:jme))
  allocate(php(ims:ime,kms:kme,jms:jme))
  allocate(mu(ims:ime,jms:jme), mub(ims:ime,jms:jme), mut(ims:ime,jms:jme))
  allocate(muu(ims:ime,jms:jme), muv(ims:ime,jms:jme))
  allocate(msftx(ims:ime,jms:jme), msfty(ims:ime,jms:jme))
  allocate(msfux(ims:ime,jms:jme), msfuy(ims:ime,jms:jme))
  allocate(msfvx(ims:ime,jms:jme), msfvx_inv(ims:ime,jms:jme), msfvy(ims:ime,jms:jme))
  allocate(c1h(kms:kme), c2h(kms:kme), c1f(kms:kme), c2f(kms:kme), dnw(kms:kme))

  config_flags%periodic_x = .false.
  config_flags%periodic_y = .false.
  mu = 10.0; mub = 90.0
  u = 1.0; v = -0.5; w = 0.25
  ph = 100.0; phb = 1000.0; al = 0.2; alb = 0.8
  moist(:,:,:,1) = -777.0; moist(:,:,:,2) = 0.001; moist(:,:,:,3) = 0.002
  msftx = 1.0; msfty = 1.1; msfux = 1.0; msfuy = 0.9
  msfvx = 1.0; msfvx_inv = 0.8; msfvy = 1.0
  c1h = 1.0; c2h = 0.1; c1f = 0.9; c2f = 0.2; dnw = -0.025
  mut = -9999.0; muu = -9999.0; muv = -9999.0
  ru = -9999.0; rv = -9999.0; rw = -9999.0; ww = -9999.0
  cqu = -9999.0; cqv = -9999.0; cqw = -9999.0; alt = -9999.0; php = -9999.0

  do call_index = 1, warmup_call_count
    call apply_preparation()
  end do

  call system_clock(count_rate=clock_rate)
  do sample = 1, sample_count
    call system_clock(start_count)
    do call_index = 1, calls_per_sample
      call apply_preparation()
    end do
    call system_clock(end_count)
    milliseconds_per_call = real(end_count-start_count,real64)*1000.0_real64 / &
                            real(clock_rate,real64)/real(calls_per_sample,real64)
    write (*, '(A,I0,A,F12.6)') 'sample_', sample, '_milliseconds_per_call ', &
                               milliseconds_per_call
  end do

  checksum = sum(real(mut,real64)) + sum(real(muu,real64)) + sum(real(muv,real64)) + &
             sum(real(ru,real64)) + sum(real(rv,real64)) + sum(real(rw,real64)) + &
             sum(real(ww,real64)) + sum(real(cqu,real64)) + sum(real(cqv,real64)) + &
             sum(real(cqw,real64)) + sum(real(alt,real64)) + sum(real(php,real64))
  write (*, '(A,I0)') 'mass_points_per_call ', active_x*active_y*active_z
  write (*, '(A,ES24.16)') 'checksum ', checksum

contains

  subroutine apply_preparation()
    call calculate_full(mut,mub,mu,ids,ide,jds,jde,1,2,ims,ime,jms,jme,1,1,its,ite,jts,jte,1,1)
    call calc_mu_uv(config_flags,mu,mub,muu,muv,ids,ide,jds,jde,kds,kde, &
                    ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kts,kte)
    call couple_momentum(muu,ru,u,msfuy,muv,rv,v,msfvx,msfvx_inv,mut,rw,w,msfty, &
                         c1h,c2h,c1f,c2f,ids,ide,jds,jde,kds,kde, &
                         ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kts,kte)
    call calc_ww_cp(u,v,mu,mub,c1h,c2h,ww,1.0,1.0,msftx,msfty,msfux,msfuy, &
                    msfvx,msfvx_inv,msfvy,dnw,ids,ide,jds,jde,kds,kde, &
                    ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kts,kte)
    call calc_cq(moist,cqu,cqv,cqw,n_moist,ids,ide,jds,jde,kds,kde, &
                 ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kts,kte)
    call calc_alt(alt,al,alb,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
                  its,ite,jts,jte,kts,kte)
    call calc_php(php,ph,phb,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
                  its,ite,jts,jte,kts,kte)
  end subroutine apply_preparation

end program runge_kutta_preparation_benchmark
