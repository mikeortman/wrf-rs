program dry_large_step_boundary_stage_driver
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_dry_boundary_relaxation, only: relax_bdy_dry
  use extracted_dry_boundary_tendencies, only: spec_bdy_dry
  implicit none

  call run_case('first_global',1,.false.,.false.,1,8,1,8,1,5,1,3,.false.)
  call run_case('first_nested',1,.true.,.false.,1,8,1,8,1,5,1,3,.false.)
  call run_case('later_nested',2,.true.,.false.,1,8,1,8,1,4,1,3,.false.)
  call run_case('periodic_nested',1,.true.,.true.,1,8,1,8,1,5,1,3,.false.)
  call run_case('south_west',1,.true.,.false.,1,5,1,5,1,5,1,3,.false.)
  call run_case('north_east',1,.true.,.false.,4,8,4,8,1,5,1,3,.false.)
  call run_case('inactive',1,.true.,.false.,4,5,4,5,1,5,1,3,.false.)
  call run_case('empty_band',1,.true.,.false.,1,8,1,8,1,5,2,2,.false.)
  call run_case('exceptional',1,.true.,.false.,1,8,1,8,1,5,1,3,.true.)

contains

  subroutine run_case(name,rk_step,nested,periodic_x,its,ite,jts,jte,kts,kte, &
      spec_zone,relax_zone,exceptional)
    character(len=*),intent(in)::name
    integer,intent(in)::rk_step,its,ite,jts,jte,kts,kte,spec_zone,relax_zone
    logical,intent(in)::nested,periodic_x,exceptional
    integer,parameter::ims=0,ime=9,jms=0,jme=9,kms=0,kme=5
    integer,parameter::ids=1,ide=9,jds=1,jde=9,kds=1,kde=5
    integer,parameter::boundary_width=4
    real::ru_tend(ims:ime,kms:kme,jms:jme),rv_tend(ims:ime,kms:kme,jms:jme)
    real::rw_tend(ims:ime,kms:kme,jms:jme),ph_tend(ims:ime,kms:kme,jms:jme)
    real::t_tend(ims:ime,kms:kme,jms:jme),ru_tendf(ims:ime,kms:kme,jms:jme)
    real::rv_tendf(ims:ime,kms:kme,jms:jme),rw_tendf(ims:ime,kms:kme,jms:jme)
    real::ph_tendf(ims:ime,kms:kme,jms:jme),t_tendf(ims:ime,kms:kme,jms:jme)
    real::u_save(ims:ime,kms:kme,jms:jme),v_save(ims:ime,kms:kme,jms:jme)
    real::w_save(ims:ime,kms:kme,jms:jme),ph_save(ims:ime,kms:kme,jms:jme)
    real::t_save(ims:ime,kms:kme,jms:jme),h_diabatic(ims:ime,kms:kme,jms:jme)
    real::ru(ims:ime,kms:kme,jms:jme),rv(ims:ime,kms:kme,jms:jme)
    real::ph_2(ims:ime,kms:kme,jms:jme),t_2(ims:ime,kms:kme,jms:jme)
    real::w_2(ims:ime,kms:kme,jms:jme)
    real::mu_tend(ims:ime,jms:jme),mu_tendf(ims:ime,jms:jme)
    real::mu_2(ims:ime,jms:jme),mut(ims:ime,jms:jme)
    real::msftx(ims:ime,jms:jme),msfty(ims:ime,jms:jme),msfux(ims:ime,jms:jme)
    real::msfuy(ims:ime,jms:jme),msfvx(ims:ime,jms:jme)
    real::msfvx_inv(ims:ime,jms:jme),msfvy(ims:ime,jms:jme)
    real::c1h(kms:kme),c2h(kms:kme),c1f(kms:kme),c2f(kms:kme)
    real::fcx(boundary_width),gcx(boundary_width)
    real::u_w(jms:jme,kds:kde,boundary_width),u_e(jms:jme,kds:kde,boundary_width)
    real::u_s(ims:ime,kds:kde,boundary_width),u_n(ims:ime,kds:kde,boundary_width)
    real::ut_w(jms:jme,kds:kde,boundary_width),ut_e(jms:jme,kds:kde,boundary_width)
    real::ut_s(ims:ime,kds:kde,boundary_width),ut_n(ims:ime,kds:kde,boundary_width)
    real::v_w(jms:jme,kds:kde,boundary_width),v_e(jms:jme,kds:kde,boundary_width)
    real::v_s(ims:ime,kds:kde,boundary_width),v_n(ims:ime,kds:kde,boundary_width)
    real::vt_w(jms:jme,kds:kde,boundary_width),vt_e(jms:jme,kds:kde,boundary_width)
    real::vt_s(ims:ime,kds:kde,boundary_width),vt_n(ims:ime,kds:kde,boundary_width)
    real::ph_w(jms:jme,kds:kde,boundary_width),ph_e(jms:jme,kds:kde,boundary_width)
    real::ph_s(ims:ime,kds:kde,boundary_width),ph_n(ims:ime,kds:kde,boundary_width)
    real::pht_w(jms:jme,kds:kde,boundary_width),pht_e(jms:jme,kds:kde,boundary_width)
    real::pht_s(ims:ime,kds:kde,boundary_width),pht_n(ims:ime,kds:kde,boundary_width)
    real::t_w(jms:jme,kds:kde,boundary_width),t_e(jms:jme,kds:kde,boundary_width)
    real::t_s(ims:ime,kds:kde,boundary_width),t_n(ims:ime,kds:kde,boundary_width)
    real::tt_w(jms:jme,kds:kde,boundary_width),tt_e(jms:jme,kds:kde,boundary_width)
    real::tt_s(ims:ime,kds:kde,boundary_width),tt_n(ims:ime,kds:kde,boundary_width)
    real::w_w(jms:jme,kds:kde,boundary_width),w_e(jms:jme,kds:kde,boundary_width)
    real::w_s(ims:ime,kds:kde,boundary_width),w_n(ims:ime,kds:kde,boundary_width)
    real::wt_w(jms:jme,kds:kde,boundary_width),wt_e(jms:jme,kds:kde,boundary_width)
    real::wt_s(ims:ime,kds:kde,boundary_width),wt_n(ims:ime,kds:kde,boundary_width)
    real::mu_w(jms:jme,1:1,boundary_width),mu_e(jms:jme,1:1,boundary_width)
    real::mu_s(ims:ime,1:1,boundary_width),mu_n(ims:ime,1:1,boundary_width)
    real::mubt_w(jms:jme,1:1,boundary_width),mubt_e(jms:jme,1:1,boundary_width)
    real::mubt_s(ims:ime,1:1,boundary_width),mubt_n(ims:ime,1:1,boundary_width)
    type(grid_config_rec_type)::config
    integer::i,j,k

    do j=jms,jme;do i=ims,ime
      mu_tend(i,j)=0.6+real(i)*0.07-real(j)*0.03
      mu_tendf(i,j)=-0.2+real(i)*0.02+real(j)*0.04
      mu_2(i,j)=60.0+real(i)*0.25-real(j)*0.125
      mut(i,j)=50.0+real(i)*2.0+real(j)*3.0
      msftx(i,j)=9.0;msfty(i,j)=1.1+real(i)*0.01+real(j)*0.02
      msfux(i,j)=8.0;msfuy(i,j)=1.0+real(i)*0.02+real(j)*0.01
      msfvx(i,j)=0.9+real(i)*0.015-real(j)*0.005
      msfvx_inv(i,j)=1.0/msfvx(i,j);msfvy(i,j)=7.0
      do k=kms,kme
        ru_tend(i,k,j)=1.0+real(i)*0.11+real(k)*0.07-real(j)*0.03
        rv_tend(i,k,j)=2.0-real(i)*0.05+real(k)*0.09+real(j)*0.02
        rw_tend(i,k,j)=-1.0+real(i)*0.04-real(k)*0.08+real(j)*0.06
        ph_tend(i,k,j)=3.0+real(i)*0.03+real(k)*0.05-real(j)*0.04
        t_tend(i,k,j)=-2.0+real(i)*0.02+real(k)*0.06+real(j)*0.01
        ru_tendf(i,k,j)=0.3+real(i)*0.013-real(k)*0.017+real(j)*0.019
        rv_tendf(i,k,j)=-0.4+real(i)*0.021+real(k)*0.015-real(j)*0.011
        rw_tendf(i,k,j)=0.5-real(i)*0.014+real(k)*0.012+real(j)*0.016
        ph_tendf(i,k,j)=-0.6+real(i)*0.018-real(k)*0.013+real(j)*0.009
        t_tendf(i,k,j)=0.7-real(i)*0.012+real(k)*0.014-real(j)*0.008
        u_save(i,k,j)=0.09+real(i)*0.003+real(k)*0.002-real(j)*0.001
        v_save(i,k,j)=-0.08+real(i)*0.002-real(k)*0.003+real(j)*0.001
        w_save(i,k,j)=0.07-real(i)*0.001+real(k)*0.002+real(j)*0.003
        ph_save(i,k,j)=-0.06+real(i)*0.004-real(k)*0.001+real(j)*0.002
        t_save(i,k,j)=0.05+real(i)*0.002+real(k)*0.003-real(j)*0.004
        h_diabatic(i,k,j)=0.001+real(i)*0.0001+real(k)*0.0002+real(j)*0.0003
        ru(i,k,j)=((10.0+real(i)*0.5)+real(k)*0.25)-real(j)*0.125
        rv(i,k,j)=((20.0-real(i)*0.25)+real(k)*0.5)+real(j)*0.0625
        ph_2(i,k,j)=((30.0+real(i)*0.125)+real(k)*0.75)-real(j)*0.25
        t_2(i,k,j)=((40.0-real(i)*0.0625)+real(k)*0.375)+real(j)*0.5
        w_2(i,k,j)=((50.0+real(i)*0.375)-real(k)*0.125)+real(j)*0.25
      enddo
    enddo;enddo

    do k=kms,kme
      c1h(k)=0.2+real(k)*0.03;c2h(k)=0.4-real(k)*0.02
      c1f(k)=0.55+real(k)*0.015625;c2f(k)=0.45-real(k)*0.0078125
    enddo
    fcx=(/0.0,0.7,0.4,0.0/);gcx=(/0.0,0.1,0.05,0.0/)

    call initialize_boundaries(u_w,u_e,u_s,u_n,ut_w,ut_e,ut_s,ut_n,100.0)
    call initialize_boundaries(v_w,v_e,v_s,v_n,vt_w,vt_e,vt_s,vt_n,200.0)
    call initialize_boundaries(ph_w,ph_e,ph_s,ph_n,pht_w,pht_e,pht_s,pht_n,300.0)
    call initialize_boundaries(t_w,t_e,t_s,t_n,tt_w,tt_e,tt_s,tt_n,400.0)
    call initialize_boundaries(w_w,w_e,w_s,w_n,wt_w,wt_e,wt_s,wt_n,500.0)
    call initialize_horizontal_boundaries(mu_w,mu_e,mu_s,mu_n,mubt_w,mubt_e,mubt_s,mubt_n,600.0)

    if(exceptional)then
      ph_s(4,1,2)=transfer(int(z'7F800000',int32),0.0)
      pht_s(4,1,2)=transfer(int(z'FF800000',int32),0.0)
      t_n(4,1,2)=transfer(int(z'80000000',int32),0.0)
      wt_w(4,1,2)=transfer(int(z'00000001',int32),0.0)
      tt_s(3,1,1)=transfer(int(z'7FC12345',int32),0.0)
      mubt_n(2,1,1)=transfer(int(z'7F7FFFFF',int32),0.0)
      mut(4,1)=transfer(int(z'7F7FFFFF',int32),0.0)
      msfty(1,1)=0.0;msfuy(2,1)=-0.0;msfvx_inv(1,2)=huge(msfvx_inv)*2.0
      h_diabatic(2,1,2)=huge(h_diabatic)*2.0;ru_tendf(2,1,1)=-0.0
      ph_tendf(1,1,1)=huge(ph_tendf)
    endif

    config%periodic_x=periodic_x;config%nested=nested

    if(rk_step==1)then
      call relax_bdy_dry(config,u_save,v_save,ph_save,t_save,w_save,mu_tend, &
        c1h,c2h,c1f,c2f,ru,rv,ph_2,t_2,w_2,mu_2,mut, &
        u_w,u_e,u_s,u_n,v_w,v_e,v_s,v_n,ph_w,ph_e,ph_s,ph_n,t_w,t_e,t_s,t_n, &
        w_w,w_e,w_s,w_n,mu_w,mu_e,mu_s,mu_n, &
        ut_w,ut_e,ut_s,ut_n,vt_w,vt_e,vt_s,vt_n,pht_w,pht_e,pht_s,pht_n, &
        tt_w,tt_e,tt_s,tt_n,wt_w,wt_e,wt_s,wt_n,mubt_w,mubt_e,mubt_s,mubt_n, &
        boundary_width,spec_zone,relax_zone,0.25,fcx,gcx, &
        ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
        ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kts,kte)
    endif

    call rk_addtend_dry(ru_tend,rv_tend,rw_tend,ph_tend,t_tend, &
      ru_tendf,rv_tendf,rw_tendf,ph_tendf,t_tendf,u_save,v_save,w_save,ph_save,t_save, &
      mu_tend,mu_tendf,rk_step,c1h,c2h,h_diabatic,mut,msftx,msfty,msfux,msfuy, &
      msfvx,msfvx_inv,msfvy,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kts,kte)

    call spec_bdy_dry(config,ru_tend,rv_tend,ph_tend,t_tend,rw_tend,mu_tend, &
      u_w,u_e,u_s,u_n,v_w,v_e,v_s,v_n,ph_w,ph_e,ph_s,ph_n,t_w,t_e,t_s,t_n, &
      w_w,w_e,w_s,w_n,mu_w,mu_e,mu_s,mu_n, &
      ut_w,ut_e,ut_s,ut_n,vt_w,vt_e,vt_s,vt_n,pht_w,pht_e,pht_s,pht_n, &
      tt_w,tt_e,tt_s,tt_n,wt_w,wt_e,wt_s,wt_n,mubt_w,mubt_e,mubt_s,mubt_n, &
      boundary_width,spec_zone, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde, &
      its,ite,jts,jte,kts,kte)

    call emit_volume(name,'ru_tend',ru_tend);call emit_volume(name,'rv_tend',rv_tend)
    call emit_volume(name,'rw_tend',rw_tend);call emit_volume(name,'ph_tend',ph_tend)
    call emit_volume(name,'t_tend',t_tend);call emit_volume(name,'ru_tendf',ru_tendf)
    call emit_volume(name,'rv_tendf',rv_tendf);call emit_volume(name,'rw_tendf',rw_tendf)
    call emit_volume(name,'ph_tendf',ph_tendf);call emit_volume(name,'t_tendf',t_tendf)
    call emit_volume(name,'u_save',u_save);call emit_volume(name,'v_save',v_save)
    call emit_volume(name,'w_save',w_save);call emit_volume(name,'ph_save',ph_save)
    call emit_volume(name,'t_save',t_save)
    call emit_horizontal(name,'mu_tend',mu_tend)
    call emit_horizontal(name,'mu_tendf',mu_tendf)
  end subroutine

  subroutine initialize_boundaries(west,east,south,north,west_tend,east_tend,south_tend,north_tend,base)
    integer,parameter::ims=0,ime=9,jms=0,jme=9,kds=1,kde=5,boundary_width=4
    real,intent(out)::west(jms:jme,kds:kde,boundary_width),east(jms:jme,kds:kde,boundary_width)
    real,intent(out)::south(ims:ime,kds:kde,boundary_width),north(ims:ime,kds:kde,boundary_width)
    real,intent(out)::west_tend(jms:jme,kds:kde,boundary_width),east_tend(jms:jme,kds:kde,boundary_width)
    real,intent(out)::south_tend(ims:ime,kds:kde,boundary_width),north_tend(ims:ime,kds:kde,boundary_width)
    real,intent(in)::base
    integer::line,k,distance
    do distance=1,boundary_width;do k=kds,kde
      do line=jms,jme
        west(line,k,distance)=base+10.0+real(line)*0.5+real(k)*0.25+real(distance)*0.03125
        east(line,k,distance)=base+20.0+real(line)*0.5+real(k)*0.25+real(distance)*0.03125
        west_tend(line,k,distance)=-base*0.01+real(line)*0.125+real(k)*0.0625+real(distance)*0.015625
        east_tend(line,k,distance)=base*0.01+real(line)*0.125+real(k)*0.0625+real(distance)*0.015625
      enddo
      do line=ims,ime
        south(line,k,distance)=base+30.0+real(line)*0.5+real(k)*0.25+real(distance)*0.03125
        north(line,k,distance)=base+40.0+real(line)*0.5+real(k)*0.25+real(distance)*0.03125
        south_tend(line,k,distance)=-base*0.02+real(line)*0.125+real(k)*0.0625+real(distance)*0.015625
        north_tend(line,k,distance)=base*0.02+real(line)*0.125+real(k)*0.0625+real(distance)*0.015625
      enddo
    enddo;enddo
  end subroutine

  subroutine initialize_horizontal_boundaries(west,east,south,north,west_tend,east_tend,south_tend,north_tend,base)
    integer,parameter::ims=0,ime=9,jms=0,jme=9,boundary_width=4
    real,intent(out)::west(jms:jme,1:1,boundary_width),east(jms:jme,1:1,boundary_width)
    real,intent(out)::south(ims:ime,1:1,boundary_width),north(ims:ime,1:1,boundary_width)
    real,intent(out)::west_tend(jms:jme,1:1,boundary_width),east_tend(jms:jme,1:1,boundary_width)
    real,intent(out)::south_tend(ims:ime,1:1,boundary_width),north_tend(ims:ime,1:1,boundary_width)
    real,intent(in)::base
    integer::line,distance
    do distance=1,boundary_width
      do line=jms,jme
        west(line,1,distance)=base+10.0+real(line)*0.5+real(distance)*0.03125
        east(line,1,distance)=base+20.0+real(line)*0.5+real(distance)*0.03125
        west_tend(line,1,distance)=-base*0.01+real(line)*0.125+real(distance)*0.015625
        east_tend(line,1,distance)=base*0.01+real(line)*0.125+real(distance)*0.015625
      enddo
      do line=ims,ime
        south(line,1,distance)=base+30.0+real(line)*0.5+real(distance)*0.03125
        north(line,1,distance)=base+40.0+real(line)*0.5+real(distance)*0.03125
        south_tend(line,1,distance)=-base*0.02+real(line)*0.125+real(distance)*0.015625
        north_tend(line,1,distance)=base*0.02+real(line)*0.125+real(distance)*0.015625
      enddo
    enddo
  end subroutine

  subroutine emit_volume(case_name,field_name,field)
    character(len=*),intent(in)::case_name,field_name
    real,intent(in)::field(0:9,0:5,0:9)
    integer::i,j,k
    do j=0,9;do k=0,5;do i=0,9
      if(isnan(field(i,k,j)))then
        write(*,'(A,1X,A,1X,A)')case_name,field_name,'NAN'
      else
        write(*,'(A,1X,A,1X,Z8.8)')case_name,field_name,transfer(field(i,k,j),0_int32)
      endif
    enddo;enddo;enddo
  end subroutine

  subroutine emit_horizontal(case_name,field_name,field)
    character(len=*),intent(in)::case_name,field_name
    real,intent(in)::field(0:9,0:9)
    integer::i,j
    do j=0,9;do i=0,9
      if(isnan(field(i,j)))then
        write(*,'(A,1X,A,1X,A)')case_name,field_name,'NAN'
      else
        write(*,'(A,1X,A,1X,Z8.8)')case_name,field_name,transfer(field(i,j),0_int32)
      endif
    enddo;enddo
  end subroutine
end program dry_large_step_boundary_stage_driver
