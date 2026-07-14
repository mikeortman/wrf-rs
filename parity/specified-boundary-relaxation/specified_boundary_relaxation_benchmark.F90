program specified_boundary_relaxation_benchmark
  use module_configure, only: grid_config_rec_type
  use extracted_specified_boundary_relaxation, only: relax_bdytend
  implicit none
  integer,parameter::ims=0,ime=257,jms=0,jme=257,kms=0,kme=41
  integer,parameter::ids=1,ide=257,jds=1,jde=257,kds=1,kde=41
  integer,parameter::boundary_width=8,calls_per_sample=100
  real,allocatable::field(:,:,:),field_tend(:,:,:)
  real,allocatable::west(:,:,:),east(:,:,:),south(:,:,:),north(:,:,:)
  real,allocatable::west_tend(:,:,:),east_tend(:,:,:),south_tend(:,:,:),north_tend(:,:,:)
  real::fcx(boundary_width),gcx(boundary_width)
  type(grid_config_rec_type)::config
  integer::sample,iteration
  integer(kind=8)::started,finished,rate

  allocate(field(ims:ime,kms:kme,jms:jme),field_tend(ims:ime,kms:kme,jms:jme))
  allocate(west(jms:jme,kds:kde,boundary_width),east(jms:jme,kds:kde,boundary_width))
  allocate(south(ims:ime,kds:kde,boundary_width),north(ims:ime,kds:kde,boundary_width))
  allocate(west_tend(jms:jme,kds:kde,boundary_width),east_tend(jms:jme,kds:kde,boundary_width))
  allocate(south_tend(ims:ime,kds:kde,boundary_width),north_tend(ims:ime,kds:kde,boundary_width))
  field=1.2;field_tend=0.1
  west=2.1;east=2.2;south=2.3;north=2.4
  west_tend=.01;east_tend=.02;south_tend=.03;north_tend=.04
  fcx=(/0.0,0.7,0.6,0.5,0.4,0.3,0.2,0.0/)
  gcx=(/0.0,0.08,0.07,0.06,0.05,0.04,0.03,0.0/)
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
  write(*,'(A,1X,ES16.8)')'checksum',sum(field_tend)
contains
  subroutine invoke
    call relax_bdytend(field,field_tend,west,east,south,north, &
      west_tend,east_tend,south_tend,north_tend,'t',config, &
      boundary_width,1,7,0.25,fcx,gcx, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
  end subroutine
end program specified_boundary_relaxation_benchmark
