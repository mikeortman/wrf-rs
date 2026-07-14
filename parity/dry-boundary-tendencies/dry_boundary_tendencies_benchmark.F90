program dry_boundary_tendencies_benchmark
  use module_configure, only: grid_config_rec_type
  use extracted_dry_boundary_tendencies, only: spec_bdy_dry
  implicit none
  integer,parameter::ims=0,ime=257,jms=0,jme=257,kms=0,kme=41
  integer,parameter::ids=1,ide=257,jds=1,jde=257,kds=1,kde=41
  integer,parameter::boundary_width=8,calls_per_sample=100,volume_fields=5
  real,allocatable::output(:,:,:,:),mu_output(:,:)
  real,allocatable::state_w(:,:,:),state_e(:,:,:),state_s(:,:,:),state_n(:,:,:)
  real,allocatable::tend_w(:,:,:,:),tend_e(:,:,:,:),tend_s(:,:,:,:),tend_n(:,:,:,:)
  real,allocatable::mu_state_w(:,:,:),mu_state_e(:,:,:),mu_state_s(:,:,:),mu_state_n(:,:,:)
  real,allocatable::mu_tend_w(:,:,:),mu_tend_e(:,:,:),mu_tend_s(:,:,:),mu_tend_n(:,:,:)
  type(grid_config_rec_type)::config
  integer::sample,iteration
  integer(kind=8)::started,finished,rate
  allocate(output(ims:ime,kms:kme,jms:jme,volume_fields),mu_output(ims:ime,jms:jme))
  allocate(state_w(jms:jme,kds:kde,boundary_width),state_e(jms:jme,kds:kde,boundary_width))
  allocate(state_s(ims:ime,kds:kde,boundary_width),state_n(ims:ime,kds:kde,boundary_width))
  allocate(tend_w(jms:jme,kds:kde,boundary_width,volume_fields))
  allocate(tend_e(jms:jme,kds:kde,boundary_width,volume_fields))
  allocate(tend_s(ims:ime,kds:kde,boundary_width,volume_fields))
  allocate(tend_n(ims:ime,kds:kde,boundary_width,volume_fields))
  allocate(mu_state_w(jms:jme,1:1,boundary_width),mu_state_e(jms:jme,1:1,boundary_width))
  allocate(mu_state_s(ims:ime,1:1,boundary_width),mu_state_n(ims:ime,1:1,boundary_width))
  allocate(mu_tend_w(jms:jme,1:1,boundary_width),mu_tend_e(jms:jme,1:1,boundary_width))
  allocate(mu_tend_s(ims:ime,1:1,boundary_width),mu_tend_n(ims:ime,1:1,boundary_width))
  output=1.2;mu_output=1.2;state_w=0.;state_e=0.;state_s=0.;state_n=0.
  tend_w=.1;tend_e=.2;tend_s=.3;tend_n=.4
  mu_state_w=0.;mu_state_e=0.;mu_state_s=0.;mu_state_n=0.
  mu_tend_w=.1;mu_tend_e=.2;mu_tend_s=.3;mu_tend_n=.4
  config%periodic_x=.false.;config%nested=.true.
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
  write(*,'(A,1X,ES16.8)')'checksum',sum(output)+sum(mu_output)
contains
  subroutine invoke
    call spec_bdy_dry(config, &
      output(:,:,:,1),output(:,:,:,2),output(:,:,:,3),output(:,:,:,4), &
      output(:,:,:,5),mu_output, &
      state_w,state_e,state_s,state_n,state_w,state_e,state_s,state_n, &
      state_w,state_e,state_s,state_n,state_w,state_e,state_s,state_n, &
      state_w,state_e,state_s,state_n,mu_state_w,mu_state_e,mu_state_s,mu_state_n, &
      tend_w(:,:,:,1),tend_e(:,:,:,1),tend_s(:,:,:,1),tend_n(:,:,:,1), &
      tend_w(:,:,:,2),tend_e(:,:,:,2),tend_s(:,:,:,2),tend_n(:,:,:,2), &
      tend_w(:,:,:,3),tend_e(:,:,:,3),tend_s(:,:,:,3),tend_n(:,:,:,3), &
      tend_w(:,:,:,4),tend_e(:,:,:,4),tend_s(:,:,:,4),tend_n(:,:,:,4), &
      tend_w(:,:,:,5),tend_e(:,:,:,5),tend_s(:,:,:,5),tend_n(:,:,:,5), &
      mu_tend_w,mu_tend_e,mu_tend_s,mu_tend_n,boundary_width,5, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
  end subroutine
end program dry_boundary_tendencies_benchmark
