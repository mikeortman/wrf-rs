program specified_boundary_geopotential_benchmark
  use module_configure, only: grid_config_rec_type
  use extracted_specified_boundary_geopotential, only: spec_bdyupdate_ph
  implicit none
  integer,parameter::ims=0,ime=257,jms=0,jme=257,kms=0,kme=41
  integer,parameter::ids=1,ide=257,jds=1,jde=257,kds=1,kde=41
  integer,parameter::calls_per_sample=100
  real,allocatable::field(:,:,:),field_tend(:,:,:),ph_save(:,:,:)
  real,allocatable::mu_tend(:,:),muts(:,:),c1(:),c2(:)
  type(grid_config_rec_type)::config
  integer::sample,iteration
  integer(kind=8)::started,finished,rate
  allocate(field(ims:ime,kms:kme,jms:jme),field_tend(ims:ime,kms:kme,jms:jme))
  allocate(ph_save(ims:ime,kms:kme,jms:jme),mu_tend(ims:ime,jms:jme),muts(ims:ime,jms:jme))
  allocate(c1(kms:kme),c2(kms:kme))
  field=-200.;field_tend=1.5;ph_save=100.;mu_tend=.3;muts=10.;c1=.4;c2=2.
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
    call spec_bdyupdate_ph(ph_save,field,field_tend,mu_tend,muts,c1,c2,.25, &
      'h',config,5,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
  end subroutine
end program specified_boundary_geopotential_benchmark
