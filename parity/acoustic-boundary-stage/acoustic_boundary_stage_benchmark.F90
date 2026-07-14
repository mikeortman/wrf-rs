program acoustic_boundary_stage_benchmark
  use iso_fortran_env,only:int64,real64
  use module_configure,only:grid_config_rec_type
  use extracted_acoustic_boundary_stage,only:small_step_prep,calc_p_rho,calc_coef_w, &
    advance_uv,spec_bdyupdate,advance_mu_t,advance_w,sumflux,spec_bdyupdate_ph, &
    zero_grad_bdy,set_physical_bc3d,set_physical_bc2d
  implicit none
  integer,parameter::nx=128,ny=128,nz=40
  integer,parameter::ims=0,ime=nx+8,jms=0,jme=ny+8,kms=0,kme=nz+1
  integer,parameter::ids=4,ide=nx+4,jds=4,jde=ny+4,kds=1,kde=nz+1
  integer,parameter::spec_zone=2,small_step_count=3
  integer,parameter::samples=31,calls_per_sample=1,warmup_calls=3
  real,allocatable::volume(:,:,:,:),horizontal(:,:,:),vertical(:,:)
  type(grid_config_rec_type)::config
  integer(int64)::started,finished,rate
  integer::sample,iteration
  real(real64)::milliseconds,checksum

  allocate(volume(ims:ime,kms:kme,jms:jme,40))
  allocate(horizontal(ims:ime,jms:jme,21),vertical(kms:kme,14))
  config=grid_config_rec_type();config%specified=.true.

  do iteration=1,warmup_calls;call initialize;call invoke;enddo
  call system_clock(count_rate=rate)
  do sample=1,samples
    call initialize
    call system_clock(started)
    do iteration=1,calls_per_sample;call invoke;enddo
    call system_clock(finished)
    milliseconds=real(finished-started,real64)*1000._real64 &
      /real(rate,real64)/real(calls_per_sample,real64)
    write(*,'(A,I0,A,F12.6)')'sample_',sample,'_milliseconds_per_call ',milliseconds
  enddo
  checksum=sum(real(volume(:,:,:,2),real64))+sum(real(volume(:,:,:,4),real64)) &
    +sum(real(volume(:,:,:,6),real64))+sum(real(volume(:,:,:,8),real64)) &
    +sum(real(volume(:,:,:,10),real64))+sum(real(volume(:,:,:,19),real64)) &
    +sum(real(volume(:,:,:,20),real64))+sum(real(volume(:,:,:,26),real64)) &
    +sum(real(volume(:,:,:,27),real64))+sum(real(volume(:,:,:,28),real64)) &
    +sum(real(horizontal(:,:,2),real64))+sum(real(horizontal(:,:,11),real64)) &
    +sum(real(horizontal(:,:,12),real64))
  write(*,'(A,ES24.16)')'checksum ',checksum

contains
  subroutine initialize
    volume=.2;horizontal=0.;vertical=1.
    volume(:,:,:,7)=300.;volume(:,:,:,8)=300.
    volume(:,:,:,9)=10.;volume(:,:,:,10)=10.
    volume(:,:,:,29)=80000.;volume(:,:,:,30)=1.
    volume(:,:,:,31)=10.;volume(:,:,:,32)=1000.
    volume(:,:,:,33:37)=.01;volume(:,:,:,38:40)=1.
    horizontal(:,:,1)=1.;horizontal(:,:,2)=1.
    horizontal(:,:,4:7)=10.;horizontal(:,:,8)=.01
    horizontal(:,:,14:20)=1.;horizontal(:,:,21)=0.
    vertical(:,1)=.60;vertical(:,2)=.40;vertical(:,3)=.55;vertical(:,4)=.45
    vertical(:,5:8)=0.;vertical(:,9)=1.;vertical(:,10)=.20
    vertical(:,11)=1.;vertical(:,12)=1.;vertical(:,13)=.60;vertical(:,14)=.40
  end subroutine initialize

  subroutine invoke
    integer::small_step
    call small_step_prep(volume(:,:,:,1),volume(:,:,:,2),volume(:,:,:,3),volume(:,:,:,4), &
      volume(:,:,:,5),volume(:,:,:,6),volume(:,:,:,7),volume(:,:,:,8),volume(:,:,:,9),volume(:,:,:,10), &
      horizontal(:,:,4),horizontal(:,:,1),horizontal(:,:,2),horizontal(:,:,5),horizontal(:,:,9), &
      horizontal(:,:,6),horizontal(:,:,10),horizontal(:,:,7),horizontal(:,:,11),horizontal(:,:,12), &
      vertical(:,1),vertical(:,2),vertical(:,3),vertical(:,4),vertical(:,5),vertical(:,6),vertical(:,7),vertical(:,8), &
      volume(:,:,:,11),volume(:,:,:,12),volume(:,:,:,13),volume(:,:,:,14),volume(:,:,:,15),horizontal(:,:,3), &
      volume(:,:,:,18),volume(:,:,:,16),volume(:,:,:,17),volume(:,:,:,29),volume(:,:,:,20),volume(:,:,:,30), &
      horizontal(:,:,14),horizontal(:,:,15),horizontal(:,:,16),horizontal(:,:,17),horizontal(:,:,18), &
      horizontal(:,:,19),horizontal(:,:,20),.1,.1,1,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde)
    call calc_p_rho(volume(:,:,:,19),volume(:,:,:,20),volume(:,:,:,10),volume(:,:,:,30),volume(:,:,:,8), &
      volume(:,:,:,14),volume(:,:,:,17),volume(:,:,:,21),horizontal(:,:,2),horizontal(:,:,11), &
      vertical(:,1),vertical(:,2),vertical(:,3),vertical(:,4),vertical(:,5),vertical(:,6),vertical(:,7),vertical(:,8), &
      vertical(:,9),300.,vertical(:,11),vertical(:,10),0.,.true.,0,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde)
    call calc_coef_w(volume(:,:,:,22),volume(:,:,:,23),volume(:,:,:,24),horizontal(:,:,7), &
      vertical(:,1),vertical(:,2),vertical(:,3),vertical(:,4),vertical(:,5),vertical(:,6),vertical(:,7),vertical(:,8), &
      volume(:,:,:,40),vertical(:,12),vertical(:,11),volume(:,:,:,17),.01,9.81,.1,.false., &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde)
    call set_physical_bc3d(volume(:,:,:,33),'u',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
    call set_physical_bc3d(volume(:,:,:,34),'v',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
    call set_physical_bc3d(volume(:,:,:,10),'w',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
    call set_physical_bc3d(volume(:,:,:,19),'p',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
    call set_physical_bc3d(volume(:,:,:,20),'p',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
    call set_physical_bc3d(volume(:,:,:,7),'p',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
    call set_physical_bc3d(volume(:,:,:,14),'t',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
    call set_physical_bc2d(horizontal(:,:,1),'t',config,ids,ide,jds,jde,ims,ime,jms,jme,ids,ide,jds,jde,ids,ide,jds,jde)
    call set_physical_bc2d(horizontal(:,:,2),'t',config,ids,ide,jds,jde,ims,ime,jms,jme,ids,ide,jds,jde,ids,ide,jds,jde)
    call set_physical_bc2d(horizontal(:,:,12),'t',config,ids,ide,jds,jde,ims,ime,jms,jme,ids,ide,jds,jde,ids,ide,jds,jde)
    do small_step=1,small_step_count
      call advance_uv(volume(:,:,:,2),volume(:,:,:,33),volume(:,:,:,4),volume(:,:,:,34),volume(:,:,:,20), &
        volume(:,:,:,29),volume(:,:,:,10),volume(:,:,:,31),volume(:,:,:,30),volume(:,:,:,19),horizontal(:,:,2), &
        horizontal(:,:,5),volume(:,:,:,38),horizontal(:,:,6),volume(:,:,:,39),horizontal(:,:,12), &
        vertical(:,1),vertical(:,2),vertical(:,3),vertical(:,4),vertical(:,5),vertical(:,6),vertical(:,7),vertical(:,8), &
        horizontal(:,:,14),horizontal(:,:,15),horizontal(:,:,16),horizontal(:,:,17),horizontal(:,:,18), &
        .1,.1,.01,.5,.3,.2,vertical(:,13),vertical(:,14),0.,vertical(:,11),config,spec_zone,.true.,.false., &
        ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde)
      call spec_bdyupdate(volume(:,:,:,2),volume(:,:,:,33),.01,'u',config,spec_zone,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
      call spec_bdyupdate(volume(:,:,:,4),volume(:,:,:,34),.01,'v',config,spec_zone,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
      call advance_mu_t(volume(:,:,:,18),volume(:,:,:,16),volume(:,:,:,2),volume(:,:,:,11),volume(:,:,:,4),volume(:,:,:,12), &
        horizontal(:,:,2),horizontal(:,:,7),horizontal(:,:,13),horizontal(:,:,11),horizontal(:,:,5),horizontal(:,:,6), &
        horizontal(:,:,12),vertical(:,1),vertical(:,2),vertical(:,3),vertical(:,4),vertical(:,5),vertical(:,6),vertical(:,7),vertical(:,8), &
        volume(:,:,:,26),volume(:,:,:,27),volume(:,:,:,28),volume(:,:,:,8),volume(:,:,:,14),volume(:,:,:,25), &
        volume(:,:,:,36),horizontal(:,:,8),.1,.1,.01,.1,vertical(:,10),vertical(:,13),vertical(:,14),vertical(:,11), &
        horizontal(:,:,14),horizontal(:,:,15),horizontal(:,:,16),horizontal(:,:,17),horizontal(:,:,18),horizontal(:,:,19),horizontal(:,:,20), &
        small_step,config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde)
      call spec_bdyupdate(volume(:,:,:,8),volume(:,:,:,36),.01,'t',config,spec_zone,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
      call spec_bdyupdate(horizontal(:,:,2),horizontal(:,:,8),.01,'m',config,spec_zone,ids,ide,jds,jde,1,1,ims,ime,jms,jme,1,1,ids,ide,jds,jde,1,1,ids,ide,jds,jde,1,1)
      call spec_bdyupdate(horizontal(:,:,11),horizontal(:,:,8),.01,'m',config,spec_zone,ids,ide,jds,jde,1,1,ims,ime,jms,jme,1,1,ids,ide,jds,jde,1,1,ids,ide,jds,jde,1,1)
      call advance_w(volume(:,:,:,6),volume(:,:,:,35),volume(:,:,:,18),volume(:,:,:,13),volume(:,:,:,2),volume(:,:,:,4), &
        horizontal(:,:,2),horizontal(:,:,7),horizontal(:,:,13),horizontal(:,:,11),vertical(:,1),vertical(:,2),vertical(:,3),vertical(:,4), &
        vertical(:,5),vertical(:,6),vertical(:,7),vertical(:,8),volume(:,:,:,25),volume(:,:,:,8),volume(:,:,:,14), &
        volume(:,:,:,10),volume(:,:,:,15),volume(:,:,:,32),volume(:,:,:,37),horizontal(:,:,21),volume(:,:,:,17), &
        volume(:,:,:,40),volume(:,:,:,30),volume(:,:,:,30),volume(:,:,:,22),volume(:,:,:,23),volume(:,:,:,24), &
        .1,.1,.01,300.,.1,vertical(:,10),vertical(:,13),vertical(:,14),vertical(:,11),vertical(:,12),.5,.3,.2, &
        horizontal(:,:,19),horizontal(:,:,20),config,.false.,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde)
      call sumflux(volume(:,:,:,2),volume(:,:,:,4),volume(:,:,:,18),volume(:,:,:,11),volume(:,:,:,12),volume(:,:,:,16), &
        horizontal(:,:,5),horizontal(:,:,6),vertical(:,1),vertical(:,2),vertical(:,3),vertical(:,4),vertical(:,5),vertical(:,6),vertical(:,7),vertical(:,8), &
        volume(:,:,:,26),volume(:,:,:,27),volume(:,:,:,28),.1,horizontal(:,:,14),horizontal(:,:,15),horizontal(:,:,16),horizontal(:,:,17),horizontal(:,:,18), &
        small_step,small_step_count,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde)
      call spec_bdyupdate_ph(volume(:,:,:,15),volume(:,:,:,10),volume(:,:,:,37),horizontal(:,:,8),horizontal(:,:,11), &
        vertical(:,3),vertical(:,4),.01,'h',config,spec_zone,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
        ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
      call zero_grad_bdy(volume(:,:,:,6),'w',config,spec_zone,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
        ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
      call calc_p_rho(volume(:,:,:,19),volume(:,:,:,20),volume(:,:,:,10),volume(:,:,:,30),volume(:,:,:,8), &
        volume(:,:,:,14),volume(:,:,:,17),volume(:,:,:,21),horizontal(:,:,2),horizontal(:,:,11),vertical(:,1),vertical(:,2), &
        vertical(:,3),vertical(:,4),vertical(:,5),vertical(:,6),vertical(:,7),vertical(:,8),vertical(:,9),300.,vertical(:,11), &
        vertical(:,10),0.,.true.,small_step,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde)
      call set_physical_bc3d(volume(:,:,:,10),'w',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
      call set_physical_bc3d(volume(:,:,:,19),'p',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
      call set_physical_bc3d(volume(:,:,:,20),'p',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
      call set_physical_bc2d(horizontal(:,:,11),'t',config,ids,ide,jds,jde,ims,ime,jms,jme,ids,ide,jds,jde,ids,ide,jds,jde)
      call set_physical_bc2d(horizontal(:,:,2),'t',config,ids,ide,jds,jde,ims,ime,jms,jme,ids,ide,jds,jde,ids,ide,jds,jde)
      call set_physical_bc2d(horizontal(:,:,12),'t',config,ids,ide,jds,jde,ims,ime,jms,jme,ids,ide,jds,jde,ids,ide,jds,jde)
    enddo
  end subroutine invoke
end program acoustic_boundary_stage_benchmark
