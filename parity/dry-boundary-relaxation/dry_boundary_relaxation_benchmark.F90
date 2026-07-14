program dry_boundary_relaxation_benchmark
  use module_configure, only: grid_config_rec_type
  use extracted_dry_boundary_relaxation, only: relax_bdy_dry
  implicit none
  integer,parameter::ims=0,ime=257,jms=0,jme=257,kms=0,kme=41
  integer,parameter::ids=1,ide=257,jds=1,jde=257,kds=1,kde=41
  integer,parameter::boundary_width=8,calls_per_sample=10
  real,allocatable::ru(:,:,:),rv(:,:,:),ph(:,:,:),theta(:,:,:),w(:,:,:)
  real,allocatable::ru_tend(:,:,:),rv_tend(:,:,:),ph_tend(:,:,:),theta_tend(:,:,:),w_tend(:,:,:)
  real,allocatable::mu(:,:),mut(:,:),mu_tend(:,:)
  real,allocatable::west(:,:,:),east(:,:,:),south(:,:,:),north(:,:,:)
  real,allocatable::west_tend(:,:,:),east_tend(:,:,:),south_tend(:,:,:),north_tend(:,:,:)
  real,allocatable::mu_west(:,:,:),mu_east(:,:,:),mu_south(:,:,:),mu_north(:,:,:)
  real,allocatable::mu_west_tend(:,:,:),mu_east_tend(:,:,:),mu_south_tend(:,:,:),mu_north_tend(:,:,:)
  real::c1h(kms:kme),c2h(kms:kme),c1f(kms:kme),c2f(kms:kme)
  real::fcx(boundary_width),gcx(boundary_width)
  type(grid_config_rec_type)::config
  integer::sample,iteration,k
  integer(kind=8)::started,finished,rate

  allocate(ru(ims:ime,kms:kme,jms:jme),rv(ims:ime,kms:kme,jms:jme))
  allocate(ph(ims:ime,kms:kme,jms:jme),theta(ims:ime,kms:kme,jms:jme),w(ims:ime,kms:kme,jms:jme))
  allocate(ru_tend(ims:ime,kms:kme,jms:jme),rv_tend(ims:ime,kms:kme,jms:jme))
  allocate(ph_tend(ims:ime,kms:kme,jms:jme),theta_tend(ims:ime,kms:kme,jms:jme),w_tend(ims:ime,kms:kme,jms:jme))
  allocate(mu(ims:ime,jms:jme),mut(ims:ime,jms:jme),mu_tend(ims:ime,jms:jme))
  allocate(west(jms:jme,kds:kde,boundary_width),east(jms:jme,kds:kde,boundary_width))
  allocate(south(ims:ime,kds:kde,boundary_width),north(ims:ime,kds:kde,boundary_width))
  allocate(west_tend(jms:jme,kds:kde,boundary_width),east_tend(jms:jme,kds:kde,boundary_width))
  allocate(south_tend(ims:ime,kds:kde,boundary_width),north_tend(ims:ime,kds:kde,boundary_width))
  allocate(mu_west(jms:jme,1:1,boundary_width),mu_east(jms:jme,1:1,boundary_width))
  allocate(mu_south(ims:ime,1:1,boundary_width),mu_north(ims:ime,1:1,boundary_width))
  allocate(mu_west_tend(jms:jme,1:1,boundary_width),mu_east_tend(jms:jme,1:1,boundary_width))
  allocate(mu_south_tend(ims:ime,1:1,boundary_width),mu_north_tend(ims:ime,1:1,boundary_width))

  ru=1.2;rv=1.3;ph=1.4;theta=1.5;w=1.6;mu=1.7;mut=10.0
  ru_tend=.1;rv_tend=.2;ph_tend=.3;theta_tend=.4;w_tend=.5;mu_tend=.6
  west=2.1;east=2.2;south=2.3;north=2.4
  west_tend=.01;east_tend=.02;south_tend=.03;north_tend=.04
  mu_west=2.1;mu_east=2.2;mu_south=2.3;mu_north=2.4
  mu_west_tend=.01;mu_east_tend=.02;mu_south_tend=.03;mu_north_tend=.04
  do k=kms,kme
    c1h(k)=.60;c2h(k)=.40;c1f(k)=.55;c2f(k)=.45
  enddo
  fcx=(/0.0,0.7,0.6,0.5,0.4,0.3,0.2,0.0/)
  gcx=(/0.0,0.08,0.07,0.06,0.05,0.04,0.03,0.0/)
  config%periodic_x=.false.;config%nested=.true.
  call invoke
  call system_clock(count_rate=rate)
  do sample=1,11
    call system_clock(started)
    do iteration=1,calls_per_sample
      call invoke
    enddo
    call system_clock(finished)
    write(*,'(F12.6)')real(finished-started)/real(rate)*1000./real(calls_per_sample)
  enddo
  write(*,'(A,1X,ES16.8)')'checksum',sum(ru_tend)+sum(rv_tend)+sum(ph_tend)+sum(theta_tend)+sum(w_tend)+sum(mu_tend)
contains
  subroutine invoke
    call relax_bdy_dry(config,ru_tend,rv_tend,ph_tend,theta_tend,w_tend,mu_tend, &
      c1h,c2h,c1f,c2f,ru,rv,ph,theta,w,mu,mut, &
      west,east,south,north,west,east,south,north,west,east,south,north, &
      west,east,south,north,west,east,south,north,mu_west,mu_east,mu_south,mu_north, &
      west_tend,east_tend,south_tend,north_tend,west_tend,east_tend,south_tend,north_tend, &
      west_tend,east_tend,south_tend,north_tend,west_tend,east_tend,south_tend,north_tend, &
      west_tend,east_tend,south_tend,north_tend,mu_west_tend,mu_east_tend,mu_south_tend,mu_north_tend, &
      boundary_width,1,7,.25,fcx,gcx,ids,ide,jds,jde,kds,kde, &
      ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,1,256,1,256,1,41)
  end subroutine
end program dry_boundary_relaxation_benchmark
