program acoustic_boundary_stage_driver
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_acoustic_boundary_stage, only: small_step_prep,calc_p_rho,calc_coef_w, &
    advance_uv,spec_bdyupdate,advance_mu_t,advance_w,sumflux,spec_bdyupdate_ph, &
    zero_grad_bdy,set_physical_bc3d,set_physical_bc2d
  implicit none
  call run_case('periodic',1)
  call run_case('specified',2)
  call run_case('nested',3)
  call run_case('partial',4)
  call run_case('inactive',5)
  call run_case('ieee',6)
contains
  subroutine run_case(case_name,mode)
    character(len=*),intent(in)::case_name
    integer,intent(in)::mode
    integer,parameter::ims=0,ime=14,jms=0,jme=14,kms=0,kme=6
    integer,parameter::ids=4,ide=10,jds=4,jde=10,kds=1,kde=6
    integer,parameter::spec_zone=2,small_step_count=3
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
    real::rhs(ims:ime,kms:kme,jms:jme)
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
    integer::i,j,k,iteration,its,ite,jts,jte
    character(len=48)::stage

    config=grid_config_rec_type()
    its=ids;ite=ide;jts=jds;jte=jde
    select case(mode)
    case(1)
      config%periodic_x=.true.;config%periodic_y=.true.
    case(2)
      config%specified=.true.
    case(3)
      config%nested=.true.
    case(4)
      config%specified=.true.;ite=6;jts=5;jte=8
    case(5)
      config%specified=.true.;its=5;ite=8;jts=5;jte=8
    case(6)
      config%specified=.true.
    case default
      error stop 'unknown acoustic boundary-stage case'
    end select

    do j=jms,jme;do k=kms,kme;do i=ims,ime
      u1(i,k,j)=volume_pattern(.10,1,i,k,j);u2(i,k,j)=volume_pattern(.20,2,i,k,j)
      v1(i,k,j)=volume_pattern(.30,3,i,k,j);v2(i,k,j)=volume_pattern(.40,4,i,k,j)
      w1(i,k,j)=volume_pattern(.50,5,i,k,j);w2(i,k,j)=volume_pattern(.60,6,i,k,j)
      t1(i,k,j)=volume_pattern(299.,7,i,k,j);t2(i,k,j)=volume_pattern(300.,8,i,k,j)
      ph1(i,k,j)=volume_pattern(9.,9,i,k,j);ph2(i,k,j)=volume_pattern(10.,10,i,k,j)
      us(i,k,j)=volume_pattern(.11,11,i,k,j);vs(i,k,j)=volume_pattern(.12,12,i,k,j)
      ws(i,k,j)=volume_pattern(.13,13,i,k,j);ts(i,k,j)=volume_pattern(299.5,14,i,k,j)
      phs(i,k,j)=volume_pattern(9.5,15,i,k,j);ww1(i,k,j)=volume_pattern(.16,16,i,k,j)
      c2a(i,k,j)=volume_pattern(.17,17,i,k,j);ww(i,k,j)=volume_pattern(.18,18,i,k,j)
      al(i,k,j)=volume_pattern(.19,19,i,k,j);p(i,k,j)=volume_pattern(.20,20,i,k,j)
      pm1(i,k,j)=volume_pattern(.21,21,i,k,j);a(i,k,j)=volume_pattern(.22,22,i,k,j)
      alpha(i,k,j)=volume_pattern(.23,23,i,k,j);gamma(i,k,j)=volume_pattern(.24,24,i,k,j)
      t2save(i,k,j)=volume_pattern(299.75,25,i,k,j)
      ru_m(i,k,j)=volume_pattern(.26,26,i,k,j);rv_m(i,k,j)=volume_pattern(.27,27,i,k,j)
      ww_m(i,k,j)=volume_pattern(.28,28,i,k,j);rhs(i,k,j)=volume_pattern(.29,29,i,k,j)
      pb(i,k,j)=volume_pattern(80000.,30,i,k,j);alt(i,k,j)=volume_pattern(1.,31,i,k,j)
      php(i,k,j)=volume_pattern(10.,32,i,k,j);phb(i,k,j)=volume_pattern(1000.,33,i,k,j)
      ru_tend(i,k,j)=volume_pattern(.01,34,i,k,j)
      rv_tend(i,k,j)=volume_pattern(.01,35,i,k,j)
      rw_tend(i,k,j)=volume_pattern(.01,36,i,k,j)
      t_tend(i,k,j)=volume_pattern(.01,37,i,k,j)
      ph_tend(i,k,j)=volume_pattern(.01,38,i,k,j)
      cqu(i,k,j)=volume_pattern(1.,39,i,k,j);cqv(i,k,j)=volume_pattern(1.,40,i,k,j)
      cqw(i,k,j)=volume_pattern(1.,41,i,k,j)
    enddo;enddo;enddo
    do j=jms,jme;do i=ims,ime
      mu1(i,j)=horizontal_pattern(1.,1,i,j);mu2(i,j)=horizontal_pattern(1.1,2,i,j)
      mus(i,j)=horizontal_pattern(.03,3,i,j);mub(i,j)=horizontal_pattern(10.,4,i,j)
      muu(i,j)=horizontal_pattern(10.1,5,i,j);muv(i,j)=horizontal_pattern(10.2,6,i,j)
      mut(i,j)=horizontal_pattern(10.3,7,i,j);mu_tend(i,j)=horizontal_pattern(.01,8,i,j)
      muus(i,j)=horizontal_pattern(.09,9,i,j);muvs(i,j)=horizontal_pattern(.10,10,i,j)
      muts(i,j)=horizontal_pattern(.11,11,i,j);mudf(i,j)=horizontal_pattern(.12,12,i,j)
      muave(i,j)=horizontal_pattern(.13,13,i,j);msfux(i,j)=horizontal_pattern(1.,14,i,j)
      msfuy(i,j)=horizontal_pattern(1.,15,i,j);msfvx(i,j)=horizontal_pattern(1.,16,i,j)
      msfvx_inv(i,j)=horizontal_pattern(1.,17,i,j);msfvy(i,j)=horizontal_pattern(1.,18,i,j)
      msftx(i,j)=horizontal_pattern(1.,19,i,j);msfty(i,j)=horizontal_pattern(1.,20,i,j)
      ht(i,j)=horizontal_pattern(0.,21,i,j)
    enddo;enddo
    do k=kms,kme
      c1h(k)=coefficient_pattern(.60,1,k);c2h(k)=coefficient_pattern(.40,2,k)
      c1f(k)=coefficient_pattern(.55,3,k);c2f(k)=coefficient_pattern(.45,4,k)
      c3h(k)=0.;c4h(k)=0.;c3f(k)=0.;c4f(k)=0.
      znu(k)=coefficient_pattern(1.,9,k);dnw(k)=coefficient_pattern(.20,10,k)
      rdnw(k)=coefficient_pattern(1.,11,k);rdn(k)=coefficient_pattern(1.,12,k)
      fnm(k)=coefficient_pattern(.60,13,k);fnp(k)=coefficient_pattern(.40,14,k)
    enddo
    if(mode==6)then
      ru_tend(0,0,0)=transfer(int(z'80000000',int32),0.)
      ru_tend(4,1,4)=transfer(int(z'7F800000',int32),0.)
      ru_tend(5,2,5)=transfer(int(z'7FC0002A',int32),0.)
      ru_tend(9,4,9)=transfer(int(z'FF800000',int32),0.)
      ru_tend(14,6,14)=transfer(int(z'7F7FFFFF',int32),0.)
    endif

    call small_step_prep(u1,u2,v1,v2,w1,w2,t1,t2,ph1,ph2,mub,mu1,mu2, &
      muu,muus,muv,muvs,mut,muts,mudf,c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f, &
      us,vs,ws,ts,phs,mus,ww,ww1,c2a,pb,p,alt,msfux,msfuy,msfvx,msfvx_inv, &
      msfvy,msftx,msfty,.1,.1,1,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme, &
      kms,kme,its,ite,jts,jte,kds,kde)
    call emit_stage_volume(case_name,'prepare:u1',u1);call emit_stage_volume(case_name,'prepare:u2',u2)
    call emit_stage_volume(case_name,'prepare:v1',v1);call emit_stage_volume(case_name,'prepare:v2',v2)
    call emit_stage_volume(case_name,'prepare:w1',w1);call emit_stage_volume(case_name,'prepare:w2',w2)
    call emit_stage_volume(case_name,'prepare:t1',t1);call emit_stage_volume(case_name,'prepare:t2',t2)
    call emit_stage_volume(case_name,'prepare:ph1',ph1);call emit_stage_volume(case_name,'prepare:ph2',ph2)
    call emit_stage_volume(case_name,'prepare:us',us);call emit_stage_volume(case_name,'prepare:vs',vs)
    call emit_stage_volume(case_name,'prepare:ws',ws);call emit_stage_volume(case_name,'prepare:ts',ts)
    call emit_stage_volume(case_name,'prepare:phs',phs);call emit_stage_volume(case_name,'prepare:c2a',c2a)
    call emit_stage_volume(case_name,'prepare:ww1',ww1)
    call emit_stage_horizontal(case_name,'prepare:mu1',mu1);call emit_stage_horizontal(case_name,'prepare:mu2',mu2)
    call emit_stage_horizontal(case_name,'prepare:muus',muus);call emit_stage_horizontal(case_name,'prepare:muvs',muvs)
    call emit_stage_horizontal(case_name,'prepare:muts',muts);call emit_stage_horizontal(case_name,'prepare:mudf',mudf)
    call emit_stage_horizontal(case_name,'prepare:mus',mus)

    call calc_p_rho(al,p,ph2,alt,t2,ts,c2a,pm1,mu2,muts,c1h,c2h,c1f,c2f, &
      c3h,c4h,c3f,c4f,znu,300.,rdnw,dnw,0.,.true.,0,ids,ide,jds,jde,kds,kde, &
      ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kds,kde)
    call emit_stage_volume(case_name,'pressure_initial:p',p)
    call emit_stage_volume(case_name,'pressure_initial:al',al)
    call emit_stage_volume(case_name,'pressure_initial:ph2',ph2)
    call emit_stage_volume(case_name,'pressure_initial:pm1',pm1)

    call calc_coef_w(a,alpha,gamma,mut,c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f, &
      cqw,rdn,rdnw,c2a,.01,9.81,.1,.false.,ids,ide,jds,jde,kds,kde, &
      ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kds,kde)
    call emit_stage_volume(case_name,'coefficients:a',a)
    call emit_stage_volume(case_name,'coefficients:alpha',alpha)
    call emit_stage_volume(case_name,'coefficients:gamma',gamma)

    call set_physical_bc3d(ru_tend,'u',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kds,kde)
    call set_physical_bc3d(rv_tend,'v',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kds,kde)
    call set_physical_bc3d(ph2,'w',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kds,kde)
    call set_physical_bc3d(al,'p',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kds,kde)
    call set_physical_bc3d(p,'p',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kds,kde)
    call set_physical_bc3d(t1,'p',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kds,kde)
    call set_physical_bc3d(ts,'t',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kds,kde)
    call set_physical_bc2d(mu1,'t',config,ids,ide,jds,jde,ims,ime,jms,jme,ids,ide,jds,jde,its,ite,jts,jte)
    call set_physical_bc2d(mu2,'t',config,ids,ide,jds,jde,ims,ime,jms,jme,ids,ide,jds,jde,its,ite,jts,jte)
    call set_physical_bc2d(mudf,'t',config,ids,ide,jds,jde,ims,ime,jms,jme,ids,ide,jds,jde,its,ite,jts,jte)
    call emit_stage_volume(case_name,'physical_initial:ru_tend',ru_tend)
    call emit_stage_volume(case_name,'physical_initial:rv_tend',rv_tend)
    call emit_stage_volume(case_name,'physical_initial:ph2',ph2)
    call emit_stage_volume(case_name,'physical_initial:al',al)
    call emit_stage_volume(case_name,'physical_initial:p',p)
    call emit_stage_volume(case_name,'physical_initial:t1',t1)
    call emit_stage_volume(case_name,'physical_initial:ts',ts)
    call emit_stage_horizontal(case_name,'physical_initial:mu1',mu1)
    call emit_stage_horizontal(case_name,'physical_initial:mu2',mu2)
    call emit_stage_horizontal(case_name,'physical_initial:mudf',mudf)

    do iteration=1,small_step_count
      call advance_uv(u2,ru_tend,v2,rv_tend,p,pb,ph2,php,alt,al,mu2,muu,cqu, &
        muv,cqv,mudf,c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f,msfux,msfuy,msfvx, &
        msfvx_inv,msfvy,.1,.1,.01,.5,.3,.2,fnm,fnp,0.,rdnw,config,spec_zone, &
        .true.,.false.,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
        its,ite,jts,jte,kds,kde)
      write(stage,'(A,I0,A)')'iteration_',iteration,':uv'
      call emit_stage_volume(case_name,trim(stage)//':u2',u2)
      call emit_stage_volume(case_name,trim(stage)//':v2',v2)
      if(config%specified.or.config%nested)then
        call spec_bdyupdate(u2,ru_tend,.01,'u',config,spec_zone,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kds,kde)
        call spec_bdyupdate(v2,rv_tend,.01,'v',config,spec_zone,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kds,kde)
      endif
      write(stage,'(A,I0,A)')'iteration_',iteration,':specified_uv'
      call emit_stage_volume(case_name,trim(stage)//':u2',u2)
      call emit_stage_volume(case_name,trim(stage)//':v2',v2)

      call advance_mu_t(ww,ww1,u2,us,v2,vs,mu2,mut,muave,muts,muu,muv,mudf, &
        c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f,ru_m,rv_m,ww_m,t2,ts,t2save,t_tend, &
        mu_tend,.1,.1,.01,.1,dnw,fnm,fnp,rdnw,msfux,msfuy,msfvx,msfvx_inv, &
        msfvy,msftx,msfty,iteration,config,ids,ide,jds,jde,kds,kde, &
        ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kds,kde)
      write(stage,'(A,I0,A)')'iteration_',iteration,':mass_theta'
      call emit_stage_volume(case_name,trim(stage)//':ww',ww)
      call emit_stage_volume(case_name,trim(stage)//':ww1',ww1)
      call emit_stage_volume(case_name,trim(stage)//':t2',t2)
      call emit_stage_volume(case_name,trim(stage)//':t2save',t2save)
      call emit_stage_volume(case_name,trim(stage)//':ru_m',ru_m)
      call emit_stage_volume(case_name,trim(stage)//':rv_m',rv_m)
      call emit_stage_volume(case_name,trim(stage)//':ww_m',ww_m)
      call emit_stage_horizontal(case_name,trim(stage)//':muave',muave)
      call emit_stage_horizontal(case_name,trim(stage)//':muts',muts)
      call emit_stage_horizontal(case_name,trim(stage)//':mudf',mudf)
      call emit_stage_horizontal(case_name,trim(stage)//':mu2',mu2)
      if(config%specified.or.config%nested)then
        call spec_bdyupdate(t2,t_tend,.01,'t',config,spec_zone,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kds,kde)
        call spec_bdyupdate(mu2,mu_tend,.01,'m',config,spec_zone,ids,ide,jds,jde,1,1,ims,ime,jms,jme,1,1,ids,ide,jds,jde,1,1,its,ite,jts,jte,1,1)
        call spec_bdyupdate(muts,mu_tend,.01,'m',config,spec_zone,ids,ide,jds,jde,1,1,ims,ime,jms,jme,1,1,ids,ide,jds,jde,1,1,its,ite,jts,jte,1,1)
      endif
      write(stage,'(A,I0,A)')'iteration_',iteration,':specified_mass_theta'
      call emit_stage_volume(case_name,trim(stage)//':t2',t2)
      call emit_stage_horizontal(case_name,trim(stage)//':mu2',mu2)
      call emit_stage_horizontal(case_name,trim(stage)//':muts',muts)

      call advance_w(w2,rw_tend,ww,ws,u2,v2,mu2,mut,muave,muts,c1h,c2h,c1f,c2f, &
        c3h,c4h,c3f,c4f,t2save,t2,ts,ph2,phs,phb,ph_tend,ht,c2a,cqw,alt,alt, &
        a,alpha,gamma,.1,.1,.01,300.,.1,dnw,fnm,fnp,rdnw,rdn,.5,.3,.2, &
        msftx,msfty,config,.false.,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme, &
        kms,kme,its,ite,jts,jte,kds,kde)
      write(stage,'(A,I0,A)')'iteration_',iteration,':vertical'
      call emit_stage_volume(case_name,trim(stage)//':w2',w2)
      call emit_stage_volume(case_name,trim(stage)//':ph2',ph2)
      call emit_stage_volume(case_name,trim(stage)//':t2save',t2save)

      call sumflux(u2,v2,ww,us,vs,ww1,muu,muv,c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f, &
        ru_m,rv_m,ww_m,.1,msfux,msfuy,msfvx,msfvx_inv,msfvy,iteration,small_step_count, &
        ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kds,kde)
      write(stage,'(A,I0,A)')'iteration_',iteration,':flux'
      call emit_stage_volume(case_name,trim(stage)//':ru_m',ru_m)
      call emit_stage_volume(case_name,trim(stage)//':rv_m',rv_m)
      call emit_stage_volume(case_name,trim(stage)//':ww_m',ww_m)

      if(config%specified.or.config%nested)then
        call spec_bdyupdate_ph(phs,ph2,ph_tend,mu_tend,muts,c1f,c2f,.01,'h',config,spec_zone,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kds,kde)
        if(config%specified)then
          call zero_grad_bdy(w2,'w',config,spec_zone,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kds,kde)
        else
          call spec_bdyupdate(w2,rw_tend,.01,'h',config,spec_zone,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kds,kde)
        endif
      endif
      write(stage,'(A,I0,A)')'iteration_',iteration,':specified_vertical'
      call emit_stage_volume(case_name,trim(stage)//':ph2',ph2)
      call emit_stage_volume(case_name,trim(stage)//':w2',w2)

      call calc_p_rho(al,p,ph2,alt,t2,ts,c2a,pm1,mu2,muts,c1h,c2h,c1f,c2f, &
        c3h,c4h,c3f,c4f,znu,300.,rdnw,dnw,0.,.true.,iteration, &
        ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kds,kde)
      write(stage,'(A,I0,A)')'iteration_',iteration,':pressure'
      call emit_stage_volume(case_name,trim(stage)//':p',p)
      call emit_stage_volume(case_name,trim(stage)//':al',al)
      call emit_stage_volume(case_name,trim(stage)//':ph2',ph2)
      call emit_stage_volume(case_name,trim(stage)//':pm1',pm1)

      call set_physical_bc3d(ph2,'w',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kds,kde)
      call set_physical_bc3d(al,'p',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kds,kde)
      call set_physical_bc3d(p,'p',config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kds,kde)
      call set_physical_bc2d(muts,'t',config,ids,ide,jds,jde,ims,ime,jms,jme,ids,ide,jds,jde,its,ite,jts,jte)
      call set_physical_bc2d(mu2,'t',config,ids,ide,jds,jde,ims,ime,jms,jme,ids,ide,jds,jde,its,ite,jts,jte)
      call set_physical_bc2d(mudf,'t',config,ids,ide,jds,jde,ims,ime,jms,jme,ids,ide,jds,jde,its,ite,jts,jte)
      write(stage,'(A,I0,A)')'iteration_',iteration,':physical'
      call emit_stage_volume(case_name,trim(stage)//':ph2',ph2)
      call emit_stage_volume(case_name,trim(stage)//':al',al)
      call emit_stage_volume(case_name,trim(stage)//':p',p)
      call emit_stage_horizontal(case_name,trim(stage)//':muts',muts)
      call emit_stage_horizontal(case_name,trim(stage)//':mu2',mu2)
      call emit_stage_horizontal(case_name,trim(stage)//':mudf',mudf)
    enddo

    call emit_volume(case_name,'final:u2',u2);call emit_volume(case_name,'final:v2',v2)
    call emit_volume(case_name,'final:w2',w2);call emit_volume(case_name,'final:t2',t2)
    call emit_volume(case_name,'final:ph2',ph2);call emit_volume(case_name,'final:al',al)
    call emit_volume(case_name,'final:p',p);call emit_volume(case_name,'final:ru_m',ru_m)
    call emit_volume(case_name,'final:rv_m',rv_m);call emit_volume(case_name,'final:ww_m',ww_m)
    call emit_horizontal(case_name,'final:mu2',mu2)
    call emit_horizontal(case_name,'final:muts',muts)
    call emit_horizontal(case_name,'final:mudf',mudf)
  end subroutine run_case

  pure real function volume_pattern(base,role,i,k,j)
    real,intent(in)::base
    integer,intent(in)::role,i,k,j
    volume_pattern=base+real(role*64+i+16*k+128*j)/4096.
  end function volume_pattern

  pure real function horizontal_pattern(base,role,i,j)
    real,intent(in)::base
    integer,intent(in)::role,i,j
    horizontal_pattern=base+real(role*64+i+128*j)/4096.
  end function horizontal_pattern

  pure real function coefficient_pattern(base,role,k)
    real,intent(in)::base
    integer,intent(in)::role,k
    coefficient_pattern=base+real(role*8+k)/65536.
  end function coefficient_pattern

  subroutine emit_stage_volume(case_name,stage,field)
    character(len=*),intent(in)::case_name,stage
    real,intent(in)::field(0:14,0:6,0:14)
    call emit_volume(case_name,stage,field)
  end subroutine emit_stage_volume

  subroutine emit_stage_horizontal(case_name,stage,field)
    character(len=*),intent(in)::case_name,stage
    real,intent(in)::field(0:14,0:14)
    call emit_horizontal(case_name,stage,field)
  end subroutine emit_stage_horizontal

  subroutine emit_volume(case_name,stage,field)
    character(len=*),intent(in)::case_name,stage
    real,intent(in)::field(0:14,0:6,0:14)
    integer::i,j,k
    write(*,'(A)',advance='no')trim(case_name)//':'//trim(stage)
    do j=0,14;do k=0,6;do i=0,14
      write(*,'(1X,Z8.8)',advance='no')transfer(field(i,k,j),0_int32)
    enddo;enddo;enddo
    write(*,*)
  end subroutine emit_volume

  subroutine emit_horizontal(case_name,stage,field)
    character(len=*),intent(in)::case_name,stage
    real,intent(in)::field(0:14,0:14)
    integer::i,j
    write(*,'(A)',advance='no')trim(case_name)//':'//trim(stage)
    do j=0,14;do i=0,14
      write(*,'(1X,Z8.8)',advance='no')transfer(field(i,j),0_int32)
    enddo;enddo
    write(*,*)
  end subroutine emit_horizontal
end program acoustic_boundary_stage_driver
