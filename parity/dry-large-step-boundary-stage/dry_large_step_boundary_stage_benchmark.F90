program dry_large_step_boundary_stage_benchmark
  use iso_fortran_env,only:int64,real64
  use module_configure,only:grid_config_rec_type
  use extracted_dry_boundary_relaxation,only:relax_bdy_dry
  use extracted_dry_boundary_tendencies,only:spec_bdy_dry
  implicit none
  integer,parameter::nx=256,ny=256,nz=40
  integer,parameter::ims=0,ime=nx+1,jms=0,jme=ny+1,kms=0,kme=nz+1
  integer,parameter::ids=1,ide=nx+1,jds=1,jde=ny+1,kds=1,kde=nz+1
  integer,parameter::its=1,ite=nx+1,jts=1,jte=ny+1,kts=1,kte=nz+1
  integer,parameter::boundary_width=5,spec_zone=1,relax_zone=4
  integer,parameter::samples=31,calls_per_sample=5,warmup_calls=5
  real,allocatable,dimension(:,:,:)::ru_tend,rv_tend,rw_tend,ph_tend,t_tend
  real,allocatable,dimension(:,:,:)::ru_tendf,rv_tendf,rw_tendf,ph_tendf,t_tendf
  real,allocatable,dimension(:,:,:)::u_save,v_save,w_save,ph_save,t_save
  real,allocatable,dimension(:,:,:)::h_diabatic,ru,rv,ph_2,t_2,w_2
  real,allocatable,dimension(:,:)::mu_tend,mu_tendf,mu_2,mut
  real,allocatable,dimension(:,:)::msftx,msfty,msfux,msfuy,msfvx,msfvx_inv,msfvy
  real,allocatable::c1h(:),c2h(:),c1f(:),c2f(:)
  real,allocatable,dimension(:,:,:,:)::bv_w,bv_e,bv_s,bv_n
  real,allocatable,dimension(:,:,:,:)::bt_w,bt_e,bt_s,bt_n
  real,allocatable,dimension(:,:,:)::mu_bv_w,mu_bv_e,mu_bv_s,mu_bv_n
  real,allocatable,dimension(:,:,:)::mu_bt_w,mu_bt_e,mu_bt_s,mu_bt_n
  real::fcx(boundary_width),gcx(boundary_width)
  type(grid_config_rec_type)::config
  integer(int64)::started,finished,rate
  integer::sample,iteration,field
  real(real64)::milliseconds,checksum

  allocate(ru_tend(ims:ime,kms:kme,jms:jme),rv_tend(ims:ime,kms:kme,jms:jme))
  allocate(rw_tend(ims:ime,kms:kme,jms:jme),ph_tend(ims:ime,kms:kme,jms:jme))
  allocate(t_tend(ims:ime,kms:kme,jms:jme))
  allocate(ru_tendf(ims:ime,kms:kme,jms:jme),rv_tendf(ims:ime,kms:kme,jms:jme))
  allocate(rw_tendf(ims:ime,kms:kme,jms:jme),ph_tendf(ims:ime,kms:kme,jms:jme))
  allocate(t_tendf(ims:ime,kms:kme,jms:jme))
  allocate(u_save(ims:ime,kms:kme,jms:jme),v_save(ims:ime,kms:kme,jms:jme))
  allocate(w_save(ims:ime,kms:kme,jms:jme),ph_save(ims:ime,kms:kme,jms:jme))
  allocate(t_save(ims:ime,kms:kme,jms:jme),h_diabatic(ims:ime,kms:kme,jms:jme))
  allocate(ru(ims:ime,kms:kme,jms:jme),rv(ims:ime,kms:kme,jms:jme))
  allocate(ph_2(ims:ime,kms:kme,jms:jme),t_2(ims:ime,kms:kme,jms:jme))
  allocate(w_2(ims:ime,kms:kme,jms:jme))
  allocate(mu_tend(ims:ime,jms:jme),mu_tendf(ims:ime,jms:jme))
  allocate(mu_2(ims:ime,jms:jme),mut(ims:ime,jms:jme))
  allocate(msftx(ims:ime,jms:jme),msfty(ims:ime,jms:jme))
  allocate(msfux(ims:ime,jms:jme),msfuy(ims:ime,jms:jme))
  allocate(msfvx(ims:ime,jms:jme),msfvx_inv(ims:ime,jms:jme),msfvy(ims:ime,jms:jme))
  allocate(c1h(kms:kme),c2h(kms:kme),c1f(kms:kme),c2f(kms:kme))
  allocate(bv_w(jms:jme,kds:kde,boundary_width,5),bv_e(jms:jme,kds:kde,boundary_width,5))
  allocate(bv_s(ims:ime,kds:kde,boundary_width,5),bv_n(ims:ime,kds:kde,boundary_width,5))
  allocate(bt_w(jms:jme,kds:kde,boundary_width,5),bt_e(jms:jme,kds:kde,boundary_width,5))
  allocate(bt_s(ims:ime,kds:kde,boundary_width,5),bt_n(ims:ime,kds:kde,boundary_width,5))
  allocate(mu_bv_w(jms:jme,1:1,boundary_width),mu_bv_e(jms:jme,1:1,boundary_width))
  allocate(mu_bv_s(ims:ime,1:1,boundary_width),mu_bv_n(ims:ime,1:1,boundary_width))
  allocate(mu_bt_w(jms:jme,1:1,boundary_width),mu_bt_e(jms:jme,1:1,boundary_width))
  allocate(mu_bt_s(ims:ime,1:1,boundary_width),mu_bt_n(ims:ime,1:1,boundary_width))

  ru_tend=.1;rv_tend=.2;rw_tend=-.3;ph_tend=.4;t_tend=-.5
  ru_tendf=.01;rv_tendf=-.02;rw_tendf=.03;ph_tendf=-.04;t_tendf=.05
  u_save=.09;v_save=-.08;w_save=.07;ph_save=-.06;t_save=.05
  h_diabatic=.001
  ru=1.2;rv=1.3;ph_2=1.4;t_2=1.5;w_2=1.6
  mu_tend=.6;mu_tendf=-.2;mu_2=1.7;mut=10.
  msftx=9.;msfty=1.12;msfux=8.;msfuy=1.03;msfvx=.97;msfvx_inv=1./.97;msfvy=7.
  c1h=.2;c2h=.4;c1f=.55;c2f=.45
  do field=1,5
    bv_w(:,:,:,field)=1.+.1*real(field)
    bv_e(:,:,:,field)=1.5+.1*real(field)
    bv_s(:,:,:,field)=2.+.1*real(field)
    bv_n(:,:,:,field)=2.5+.1*real(field)
    bt_w(:,:,:,field)=.01*real(field)
    bt_e(:,:,:,field)=.02*real(field)
    bt_s(:,:,:,field)=.03*real(field)
    bt_n(:,:,:,field)=.04*real(field)
  enddo
  mu_bv_w=3.1;mu_bv_e=3.2;mu_bv_s=3.3;mu_bv_n=3.4
  mu_bt_w=.05;mu_bt_e=.06;mu_bt_s=.07;mu_bt_n=.08
  fcx=(/0.0,0.7,0.5,0.3,0.0/)
  gcx=(/0.0,0.1,0.07,0.04,0.0/)
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
  checksum=sum(real(ru_tend(ids:ide,kds:kde,jds:jde),real64)) &
    +sum(real(rv_tend(ids:ide,kds:kde,jds:jde),real64)) &
    +sum(real(rw_tend(ids:ide,kds:kde,jds:jde),real64)) &
    +sum(real(ph_tend(ids:ide,kds:kde,jds:jde),real64)) &
    +sum(real(t_tend(ids:ide,kds:kde,jds:jde),real64)) &
    +sum(real(u_save(ids:ide,kds:kde,jds:jde),real64)) &
    +sum(real(v_save(ids:ide,kds:kde,jds:jde),real64)) &
    +sum(real(w_save(ids:ide,kds:kde,jds:jde),real64)) &
    +sum(real(ph_save(ids:ide,kds:kde,jds:jde),real64)) &
    +sum(real(t_save(ids:ide,kds:kde,jds:jde),real64)) &
    +sum(real(mu_tend(ids:ide,jds:jde),real64)) &
    +sum(real(mu_tendf(ids:ide,jds:jde),real64))
  write(*,'(A,ES24.16)')'checksum ',checksum

contains
  subroutine invoke
    call relax_bdy_dry(config,u_save,v_save,ph_save,t_save,w_save,mu_tend, &
      c1h,c2h,c1f,c2f,ru,rv,ph_2,t_2,w_2,mu_2,mut, &
      bv_w(:,:,:,1),bv_e(:,:,:,1),bv_s(:,:,:,1),bv_n(:,:,:,1), &
      bv_w(:,:,:,2),bv_e(:,:,:,2),bv_s(:,:,:,2),bv_n(:,:,:,2), &
      bv_w(:,:,:,3),bv_e(:,:,:,3),bv_s(:,:,:,3),bv_n(:,:,:,3), &
      bv_w(:,:,:,4),bv_e(:,:,:,4),bv_s(:,:,:,4),bv_n(:,:,:,4), &
      bv_w(:,:,:,5),bv_e(:,:,:,5),bv_s(:,:,:,5),bv_n(:,:,:,5), &
      mu_bv_w,mu_bv_e,mu_bv_s,mu_bv_n, &
      bt_w(:,:,:,1),bt_e(:,:,:,1),bt_s(:,:,:,1),bt_n(:,:,:,1), &
      bt_w(:,:,:,2),bt_e(:,:,:,2),bt_s(:,:,:,2),bt_n(:,:,:,2), &
      bt_w(:,:,:,3),bt_e(:,:,:,3),bt_s(:,:,:,3),bt_n(:,:,:,3), &
      bt_w(:,:,:,4),bt_e(:,:,:,4),bt_s(:,:,:,4),bt_n(:,:,:,4), &
      bt_w(:,:,:,5),bt_e(:,:,:,5),bt_s(:,:,:,5),bt_n(:,:,:,5), &
      mu_bt_w,mu_bt_e,mu_bt_s,mu_bt_n, &
      boundary_width,spec_zone,relax_zone,0.25,fcx,gcx, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kts,kte)
    call rk_addtend_dry(ru_tend,rv_tend,rw_tend,ph_tend,t_tend, &
      ru_tendf,rv_tendf,rw_tendf,ph_tendf,t_tendf, &
      u_save,v_save,w_save,ph_save,t_save, &
      mu_tend,mu_tendf,1,c1h,c2h,h_diabatic,mut, &
      msftx,msfty,msfux,msfuy,msfvx,msfvx_inv,msfvy, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kts,kte)
    call spec_bdy_dry(config,ru_tend,rv_tend,ph_tend,t_tend,rw_tend,mu_tend, &
      bv_w(:,:,:,1),bv_e(:,:,:,1),bv_s(:,:,:,1),bv_n(:,:,:,1), &
      bv_w(:,:,:,2),bv_e(:,:,:,2),bv_s(:,:,:,2),bv_n(:,:,:,2), &
      bv_w(:,:,:,3),bv_e(:,:,:,3),bv_s(:,:,:,3),bv_n(:,:,:,3), &
      bv_w(:,:,:,4),bv_e(:,:,:,4),bv_s(:,:,:,4),bv_n(:,:,:,4), &
      bv_w(:,:,:,5),bv_e(:,:,:,5),bv_s(:,:,:,5),bv_n(:,:,:,5), &
      mu_bv_w,mu_bv_e,mu_bv_s,mu_bv_n, &
      bt_w(:,:,:,1),bt_e(:,:,:,1),bt_s(:,:,:,1),bt_n(:,:,:,1), &
      bt_w(:,:,:,2),bt_e(:,:,:,2),bt_s(:,:,:,2),bt_n(:,:,:,2), &
      bt_w(:,:,:,3),bt_e(:,:,:,3),bt_s(:,:,:,3),bt_n(:,:,:,3), &
      bt_w(:,:,:,4),bt_e(:,:,:,4),bt_s(:,:,:,4),bt_n(:,:,:,4), &
      bt_w(:,:,:,5),bt_e(:,:,:,5),bt_s(:,:,:,5),bt_n(:,:,:,5), &
      mu_bt_w,mu_bt_e,mu_bt_s,mu_bt_n, &
      boundary_width,spec_zone, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kts,kte)
  end subroutine
end program
