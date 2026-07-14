program dry_tendency_boundary_stage_benchmark
  use iso_fortran_env,only:int64,real64
  use module_configure,only:grid_config_rec_type
  use extracted_dry_boundary_tendencies,only:spec_bdy_dry
  implicit none
  integer,parameter::nx=256,ny=256,nz=40
  integer,parameter::ims=0,ime=nx+1,jms=0,jme=ny+1,kms=0,kme=nz+1
  integer,parameter::ids=1,ide=nx+1,jds=1,jde=ny+1,kds=1,kde=nz+1
  integer,parameter::boundary_width=3,samples=11,calls_per_sample=20,warmup_calls=10
  real,allocatable::rk(:,:,:,:),forward(:,:,:,:),saved(:,:,:,:),heat(:,:,:)
  real,allocatable::mu(:,:),muf(:,:),mut(:,:),msfty(:,:),msfuy(:,:),msfvx(:,:),msfvxi(:,:)
  real,allocatable::dummy_map(:,:),c1(:),c2(:)
  real,allocatable::state_w(:,:,:),state_e(:,:,:),state_s(:,:,:),state_n(:,:,:)
  real,allocatable::b_w(:,:,:,:),b_e(:,:,:,:),b_s(:,:,:,:),b_n(:,:,:,:)
  real,allocatable::mu_state_w(:,:,:),mu_state_e(:,:,:),mu_state_s(:,:,:),mu_state_n(:,:,:)
  real,allocatable::mu_w(:,:,:),mu_e(:,:,:),mu_s(:,:,:),mu_n(:,:,:)
  type(grid_config_rec_type)::config
  integer(int64)::started,finished,rate
  integer::sample,iteration
  real(real64)::milliseconds,checksum

  allocate(rk(ims:ime,kms:kme,jms:jme,5),forward(ims:ime,kms:kme,jms:jme,5))
  allocate(saved(ims:ime,kms:kme,jms:jme,5),heat(ims:ime,kms:kme,jms:jme))
  allocate(mu(ims:ime,jms:jme),muf(ims:ime,jms:jme),mut(ims:ime,jms:jme))
  allocate(msfty(ims:ime,jms:jme),msfuy(ims:ime,jms:jme),msfvx(ims:ime,jms:jme))
  allocate(msfvxi(ims:ime,jms:jme),dummy_map(ims:ime,jms:jme),c1(kms:kme),c2(kms:kme))
  allocate(state_w(jms:jme,kds:kde,boundary_width),state_e(jms:jme,kds:kde,boundary_width))
  allocate(state_s(ims:ime,kds:kde,boundary_width),state_n(ims:ime,kds:kde,boundary_width))
  allocate(b_w(jms:jme,kds:kde,boundary_width,5),b_e(jms:jme,kds:kde,boundary_width,5))
  allocate(b_s(ims:ime,kds:kde,boundary_width,5),b_n(ims:ime,kds:kde,boundary_width,5))
  allocate(mu_state_w(jms:jme,1:1,boundary_width),mu_state_e(jms:jme,1:1,boundary_width))
  allocate(mu_state_s(ims:ime,1:1,boundary_width),mu_state_n(ims:ime,1:1,boundary_width))
  allocate(mu_w(jms:jme,1:1,boundary_width),mu_e(jms:jme,1:1,boundary_width))
  allocate(mu_s(ims:ime,1:1,boundary_width),mu_n(ims:ime,1:1,boundary_width))
  rk(:,:,:,1)=1.;rk(:,:,:,2)=2.;rk(:,:,:,3)=-1.;rk(:,:,:,4)=3.;rk(:,:,:,5)=-2.
  forward(:,:,:,1)=.3;forward(:,:,:,2)=-.4;forward(:,:,:,3)=.5
  forward(:,:,:,4)=-.6;forward(:,:,:,5)=.7
  saved(:,:,:,1)=.09;saved(:,:,:,2)=-.08;saved(:,:,:,3)=.07
  saved(:,:,:,4)=-.06;saved(:,:,:,5)=.05;heat=.001
  mu=.6;muf=-.2;mut=50.;msfty=1.12;msfuy=1.03;msfvx=.97;msfvxi=1./.97
  dummy_map=1.;c1=.2;c2=.4
  state_w=0.;state_e=0.;state_s=0.;state_n=0.
  b_w=.1;b_e=.2;b_s=.3;b_n=.4
  mu_state_w=0.;mu_state_e=0.;mu_state_s=0.;mu_state_n=0.
  mu_w=.1;mu_e=.2;mu_s=.3;mu_n=.4
  config%periodic_x=.false.;config%nested=.true.

  do iteration=1,warmup_calls;call invoke;enddo
  call system_clock(count_rate=rate)
  do sample=1,samples
    call system_clock(started)
    do iteration=1,calls_per_sample;call invoke;enddo
    call system_clock(finished)
    milliseconds=real(finished-started,real64)*1000._real64/real(rate,real64)/real(calls_per_sample,real64)
    write(*,'(A,I0,A,F12.6)')'sample_',sample,'_milliseconds_per_call ',milliseconds
  enddo
  checksum=sum(real(rk(ids:ide,kds:kde-1,jds:jde-1,:),real64))+sum(real(mu(ids:ide-1,jds:jde-1),real64))
  write(*,'(A,ES24.16)')'checksum ',checksum

contains
  subroutine invoke
    call rk_addtend_dry(rk(:,:,:,1),rk(:,:,:,2),rk(:,:,:,3),rk(:,:,:,4),rk(:,:,:,5), &
      forward(:,:,:,1),forward(:,:,:,2),forward(:,:,:,3),forward(:,:,:,4),forward(:,:,:,5), &
      saved(:,:,:,1),saved(:,:,:,2),saved(:,:,:,3),saved(:,:,:,4),saved(:,:,:,5), &
      mu,muf,1,c1,c2,heat,mut,dummy_map,msfty,dummy_map,msfuy,msfvx,msfvxi,dummy_map, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde, &
      ids,ide,jds,jde,kds,kde)
    call spec_bdy_dry(config,rk(:,:,:,1),rk(:,:,:,2),rk(:,:,:,4),rk(:,:,:,5),rk(:,:,:,3),mu, &
      state_w,state_e,state_s,state_n,state_w,state_e,state_s,state_n, &
      state_w,state_e,state_s,state_n,state_w,state_e,state_s,state_n, &
      state_w,state_e,state_s,state_n,mu_state_w,mu_state_e,mu_state_s,mu_state_n, &
      b_w(:,:,:,1),b_e(:,:,:,1),b_s(:,:,:,1),b_n(:,:,:,1), &
      b_w(:,:,:,2),b_e(:,:,:,2),b_s(:,:,:,2),b_n(:,:,:,2), &
      b_w(:,:,:,3),b_e(:,:,:,3),b_s(:,:,:,3),b_n(:,:,:,3), &
      b_w(:,:,:,4),b_e(:,:,:,4),b_s(:,:,:,4),b_n(:,:,:,4), &
      b_w(:,:,:,5),b_e(:,:,:,5),b_s(:,:,:,5),b_n(:,:,:,5), &
      mu_w,mu_e,mu_s,mu_n,boundary_width,2,ids,ide,jds,jde,kds,kde, &
      ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
  end subroutine
end program
