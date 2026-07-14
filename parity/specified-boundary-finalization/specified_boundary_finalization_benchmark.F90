program specified_boundary_finalization_benchmark
  use module_configure, only: grid_config_rec_type
  use extracted_specified_boundary_finalization, only: spec_bdy_final
  implicit none
  integer,parameter::ims=0,ime=257,jms=0,jme=257,kms=0,kme=41
  integer,parameter::ids=1,ide=257,jds=1,jde=257,kds=1,kde=41
  integer,parameter::boundary_width=8,calls_per_sample=100
  real,allocatable::field(:,:,:),mu(:,:),msf(:,:),c1(:),c2(:)
  real,allocatable::west(:,:,:),east(:,:,:),south(:,:,:),north(:,:,:)
  real,allocatable::west_tendency(:,:,:),east_tendency(:,:,:)
  real,allocatable::south_tendency(:,:,:),north_tendency(:,:,:)
  type(grid_config_rec_type)::config
  integer::sample,iteration
  integer(kind=8)::started,finished,rate

  allocate(field(ims:ime,kms:kme,jms:jme),mu(ims:ime,jms:jme),msf(ims:ime,jms:jme))
  allocate(c1(kms:kme),c2(kms:kme))
  allocate(west(jms:jme,kds:kde,boundary_width),east(jms:jme,kds:kde,boundary_width))
  allocate(south(ims:ime,kds:kde,boundary_width),north(ims:ime,kds:kde,boundary_width))
  allocate(west_tendency(jms:jme,kds:kde,boundary_width))
  allocate(east_tendency(jms:jme,kds:kde,boundary_width))
  allocate(south_tendency(ims:ime,kds:kde,boundary_width))
  allocate(north_tendency(ims:ime,kds:kde,boundary_width))
  field=-30.;mu=5.;msf=.9;c1=.4;c2=1.3
  west=10.;east=-8.;south=4.;north=-2.
  west_tendency=.03;east_tendency=-.02;south_tendency=.01;north_tendency=-.015
  config%periodic_x=.false.
  call invoke
  call system_clock(count_rate=rate)
  do sample=1,31
    call system_clock(started)
    do iteration=1,calls_per_sample
      call invoke
    enddo
    call system_clock(finished)
    write(*,'(F12.6)')real(finished-started)/real(rate)*1000./real(calls_per_sample)
  enddo
  write(*,'(A,1X,ES16.8)')'checksum',sum(field)
contains
  subroutine invoke
    call spec_bdy_final(field,mu,c1,c2,msf,west,east,south,north, &
      west_tendency,east_tendency,south_tendency,north_tendency, &
      'w',config,boundary_width,5,.25, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,ids,ide-1,jds,jde-1,kds,kde)
  end subroutine
end program specified_boundary_finalization_benchmark
