program acoustic_trajectory_driver
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_acoustic_trajectory, only: small_step_prep,calc_p_rho,calc_coef_w, &
    advance_uv,advance_mu_t,advance_w,sumflux
  implicit none
  integer,parameter::ims=0,ime=5,jms=0,jme=5,kms=0,kme=5
  integer,parameter::ids=1,ide=5,jds=1,jde=5,kds=1,kde=5
  real::u1(ims:ime,kms:kme,jms:jme),u2(ims:ime,kms:kme,jms:jme)
  real::v1(ims:ime,kms:kme,jms:jme),v2(ims:ime,kms:kme,jms:jme)
  real::w1(ims:ime,kms:kme,jms:jme),w2(ims:ime,kms:kme,jms:jme)
  real::t1(ims:ime,kms:kme,jms:jme),t2(ims:ime,kms:kme,jms:jme)
  real::ph1(ims:ime,kms:kme,jms:jme),ph2(ims:ime,kms:kme,jms:jme)
  real::us(ims:ime,kms:kme,jms:jme),vs(ims:ime,kms:kme,jms:jme)
  real::ws(ims:ime,kms:kme,jms:jme),ts(ims:ime,kms:kme,jms:jme)
  real::phs(ims:ime,kms:kme,jms:jme),ww1(ims:ime,kms:kme,jms:jme)
  real::c2a(ims:ime,kms:kme,jms:jme),ww(ims:ime,kms:kme,jms:jme)
  real::al(ims:ime,kms:kme,jms:jme),p(ims:ime,kms:kme,jms:jme)
  real::pm1(ims:ime,kms:kme,jms:jme),a(ims:ime,kms:kme,jms:jme)
  real::alpha(ims:ime,kms:kme,jms:jme),gamma(ims:ime,kms:kme,jms:jme)
  real::t2save(ims:ime,kms:kme,jms:jme),ru_m(ims:ime,kms:kme,jms:jme)
  real::rv_m(ims:ime,kms:kme,jms:jme),ww_m(ims:ime,kms:kme,jms:jme)
  real::pb(ims:ime,kms:kme,jms:jme),alt(ims:ime,kms:kme,jms:jme)
  real::php(ims:ime,kms:kme,jms:jme),phb(ims:ime,kms:kme,jms:jme)
  real::ru_tend(ims:ime,kms:kme,jms:jme),rv_tend(ims:ime,kms:kme,jms:jme)
  real::rw_tend(ims:ime,kms:kme,jms:jme),t_tend(ims:ime,kms:kme,jms:jme)
  real::ph_tend(ims:ime,kms:kme,jms:jme),cqu(ims:ime,kms:kme,jms:jme)
  real::cqv(ims:ime,kms:kme,jms:jme),cqw(ims:ime,kms:kme,jms:jme)
  real::mu1(ims:ime,jms:jme),mu2(ims:ime,jms:jme),mus(ims:ime,jms:jme)
  real::mub(ims:ime,jms:jme),muu(ims:ime,jms:jme),muv(ims:ime,jms:jme)
  real::mut(ims:ime,jms:jme),mu_tend(ims:ime,jms:jme),muus(ims:ime,jms:jme)
  real::muvs(ims:ime,jms:jme),muts(ims:ime,jms:jme),mudf(ims:ime,jms:jme)
  real::muave(ims:ime,jms:jme),msfux(ims:ime,jms:jme),msfuy(ims:ime,jms:jme)
  real::msfvx(ims:ime,jms:jme),msfvx_inv(ims:ime,jms:jme),msfvy(ims:ime,jms:jme)
  real::msftx(ims:ime,jms:jme),msfty(ims:ime,jms:jme),ht(ims:ime,jms:jme)
  real::c1h(kms:kme),c2h(kms:kme),c1f(kms:kme),c2f(kms:kme)
  real::c3h(kms:kme),c4h(kms:kme),c3f(kms:kme),c4f(kms:kme)
  real::znu(kms:kme),dnw(kms:kme),rdnw(kms:kme),rdn(kms:kme)
  real::fnm(kms:kme),fnp(kms:kme)
  type(grid_config_rec_type)::config
  integer::iteration

  u1=.2;u2=.2;v1=.2;v2=.2;w1=.2;w2=.2;t1=300.;t2=300.
  ph1=10.;ph2=10.;us=.2;vs=.2;ws=.2;ts=.2;phs=.2;ww1=.2;c2a=.2
  ww=.2;al=.2;p=.2;pm1=.2;a=.2;alpha=.2;gamma=.2;t2save=.2
  ru_m=.2;rv_m=.2;ww_m=.2
  mu1=1.;mu2=1.;mus=0.;muus=0.;muvs=0.;muts=0.;mudf=0.;muave=0.
  pb=80000.;alt=1.;php=10.;phb=1000.
  ru_tend=.01;rv_tend=.01;rw_tend=.01;t_tend=.01;ph_tend=.01
  cqu=1.;cqv=1.;cqw=1.
  mub=10.;muu=10.;muv=10.;mut=10.;mu_tend=.01
  msfux=1.;msfuy=1.;msfvx=1.;msfvx_inv=1.;msfvy=1.;msftx=1.;msfty=1.;ht=0.
  c1h=.60;c2h=.40;c1f=.55;c2f=.45;c3h=0.;c4h=0.;c3f=0.;c4f=0.
  znu=1.;dnw=.20;rdnw=1.;rdn=1.;fnm=.60;fnp=.40

  call small_step_prep(u1,u2,v1,v2,w1,w2,t1,t2,ph1,ph2,mub,mu1,mu2, &
    muu,muus,muv,muvs,mut,muts,mudf,c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f, &
    us,vs,ws,ts,phs,mus,ww,ww1,c2a,pb,p,alt,msfux,msfuy,msfvx,msfvx_inv, &
    msfvy,msftx,msfty,.1,.1,1,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme, &
    kms,kme,1,5,1,5,1,5)
  call calc_p_rho(al,p,ph2,alt,t2,ts,c2a,pm1,mu2,muts,c1h,c2h,c1f,c2f, &
    c3h,c4h,c3f,c4f,znu,300.,rdnw,dnw,0.,.true.,0,ids,ide,jds,jde,kds,kde, &
    ims,ime,jms,jme,kms,kme,1,5,1,5,1,5)
  call calc_coef_w(a,alpha,gamma,mut,c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f, &
    cqw,rdn,rdnw,c2a,.01,9.81,.1,.false.,ids,ide,jds,jde,kds,kde, &
    ims,ime,jms,jme,kms,kme,1,5,1,5,1,5)

  do iteration=1,3
    call advance_uv(u2,ru_tend,v2,rv_tend,p,pb,ph2,php,alt,al,mu2,muu,cqu, &
      muv,cqv,mudf,c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f,msfux,msfuy,msfvx, &
      msfvx_inv,msfvy,.1,.1,.01,.5,.3,.2,fnm,fnp,0.,rdnw,config,0, &
      .true.,.false.,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      1,5,1,5,1,5)
    call advance_mu_t(ww,ww1,u2,us,v2,vs,mu2,mut,muave,muts,muu,muv,mudf, &
      c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f,ru_m,rv_m,ww_m,t2,ts,t2save,t_tend, &
      mu_tend,.1,.1,.01,.1,dnw,fnm,fnp,rdnw,msfux,msfuy,msfvx,msfvx_inv, &
      msfvy,msftx,msfty,iteration,config,ids,ide,jds,jde,kds,kde, &
      ims,ime,jms,jme,kms,kme,1,5,1,5,1,5)
    call advance_w(w2,rw_tend,ww,ws,u2,v2,mu2,mut,muave,muts,c1h,c2h,c1f,c2f, &
      c3h,c4h,c3f,c4f,t2save,t2,ts,ph2,phs,phb,ph_tend,ht,c2a,cqw,alt,alt, &
      a,alpha,gamma,.1,.1,.01,300.,.1,dnw,fnm,fnp,rdnw,rdn,.5,.3,.2, &
      msftx,msfty,config,.false.,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme, &
      kms,kme,1,5,1,5,1,5)
    call sumflux(u2,v2,ww,us,vs,ww1,muu,muv,c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f, &
      ru_m,rv_m,ww_m,.1,msfux,msfuy,msfvx,msfvx_inv,msfvy,iteration,3, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,1,5,1,5,1,5)
    call calc_p_rho(al,p,ph2,alt,t2,ts,c2a,pm1,mu2,muts,c1h,c2h,c1f,c2f, &
      c3h,c4h,c3f,c4f,znu,300.,rdnw,dnw,0.,.true.,iteration, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,1,5,1,5,1,5)
  enddo

  call emit_volume('u2',u2);call emit_volume('v2',v2)
  call emit_volume('w2',w2);call emit_volume('t2',t2)
  call emit_volume('ph2',ph2);call emit_horizontal('mu2',mu2)
  call emit_volume('al',al);call emit_volume('p',p)
  call emit_volume('ru_m',ru_m);call emit_volume('rv_m',rv_m)
  call emit_volume('ww_m',ww_m)
contains
  subroutine emit_volume(name,field)
    character(len=*),intent(in)::name
    real,intent(in)::field(ims:ime,kms:kme,jms:jme)
    integer::i,j,k
    do j=jms,jme;do k=kms,kme;do i=ims,ime
      write(*,'(A,3(1X,I0),1X,Z8.8)')name,i,k,j,transfer(field(i,k,j),0_int32)
    enddo;enddo;enddo
  end subroutine
  subroutine emit_horizontal(name,field)
    character(len=*),intent(in)::name
    real,intent(in)::field(ims:ime,jms:jme)
    integer::i,j
    do j=jms,jme;do i=ims,ime
      write(*,'(A,2(1X,I0),1X,Z8.8)')name,i,j,transfer(field(i,j),0_int32)
    enddo;enddo
  end subroutine
end program acoustic_trajectory_driver
