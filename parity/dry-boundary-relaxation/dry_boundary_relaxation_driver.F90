program dry_boundary_relaxation_driver
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_dry_boundary_relaxation, only: relax_bdy_dry
  implicit none

  call run_case('full_global',.false.,.false.,1,8,1,8,1,3,.false.)
  call run_case('full_nested',.false.,.true.,1,8,1,8,1,3,.false.)
  call run_case('periodic_nested',.true.,.true.,1,8,1,8,1,3,.false.)
  call run_case('south_west',.false.,.true.,1,5,1,5,1,3,.false.)
  call run_case('north_east',.false.,.true.,4,8,4,8,1,3,.false.)
  call run_case('inactive',.false.,.true.,4,5,4,5,1,3,.false.)
  call run_case('empty_band',.false.,.true.,1,8,1,8,2,2,.false.)
  call run_case('exceptional',.false.,.true.,1,8,1,8,1,3,.true.)
contains
  subroutine run_case(name,periodic_x,nested,its,ite,jts,jte,spec_zone,relax_zone,exceptional)
    character(len=*),intent(in)::name
    logical,intent(in)::periodic_x,nested,exceptional
    integer,intent(in)::its,ite,jts,jte,spec_zone,relax_zone
    integer,parameter::ims=0,ime=9,jms=0,jme=9,kms=0,kme=5
    integer,parameter::ids=1,ide=9,jds=1,jde=9,kds=1,kde=5
    integer,parameter::boundary_width=4,kts=1,kte=5
    real::ru(ims:ime,kms:kme,jms:jme),rv(ims:ime,kms:kme,jms:jme)
    real::ph(ims:ime,kms:kme,jms:jme),theta(ims:ime,kms:kme,jms:jme)
    real::w(ims:ime,kms:kme,jms:jme),mu(ims:ime,jms:jme),mut(ims:ime,jms:jme)
    real::ru_tend(ims:ime,kms:kme,jms:jme),rv_tend(ims:ime,kms:kme,jms:jme)
    real::ph_tend(ims:ime,kms:kme,jms:jme),theta_tend(ims:ime,kms:kme,jms:jme)
    real::w_tend(ims:ime,kms:kme,jms:jme),mu_tend(ims:ime,jms:jme)
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
    real::mut_w(jms:jme,1:1,boundary_width),mut_e(jms:jme,1:1,boundary_width)
    real::mut_s(ims:ime,1:1,boundary_width),mut_n(ims:ime,1:1,boundary_width)
    real::c1h(kms:kme),c2h(kms:kme),c1f(kms:kme),c2f(kms:kme)
    real::fcx(boundary_width),gcx(boundary_width)
    type(grid_config_rec_type)::config
    integer::i,j,k

    do j=jms,jme;do k=kms,kme;do i=ims,ime
      ru(i,k,j)=((10.0+real(i)*0.5)+real(k)*0.25)-real(j)*0.125
      rv(i,k,j)=((20.0-real(i)*0.25)+real(k)*0.5)+real(j)*0.0625
      ph(i,k,j)=((30.0+real(i)*0.125)+real(k)*0.75)-real(j)*0.25
      theta(i,k,j)=((40.0-real(i)*0.0625)+real(k)*0.375)+real(j)*0.5
      w(i,k,j)=((50.0+real(i)*0.375)-real(k)*0.125)+real(j)*0.25
      ru_tend(i,k,j)=((-10.0+real(i)*0.25)+real(k)*0.0625)-real(j)*0.5
      rv_tend(i,k,j)=((-20.0-real(i)*0.125)+real(k)*0.25)+real(j)*0.375
      ph_tend(i,k,j)=((-30.0+real(i)*0.5)-real(k)*0.125)+real(j)*0.0625
      theta_tend(i,k,j)=((-40.0-real(i)*0.25)+real(k)*0.5)-real(j)*0.125
      w_tend(i,k,j)=((-50.0+real(i)*0.0625)+real(k)*0.375)+real(j)*0.25
    enddo;enddo;enddo
    do j=jms,jme;do i=ims,ime
      mu(i,j)=60.0+real(i)*0.25-real(j)*0.125
      mut(i,j)=10.0+real(i)*0.125+real(j)*0.0625
      mu_tend(i,j)=-60.0+real(i)*0.5+real(j)*0.25
    enddo;enddo

    call initialize_boundaries(u_w,u_e,u_s,u_n,ut_w,ut_e,ut_s,ut_n,100.0)
    call initialize_boundaries(v_w,v_e,v_s,v_n,vt_w,vt_e,vt_s,vt_n,200.0)
    call initialize_boundaries(ph_w,ph_e,ph_s,ph_n,pht_w,pht_e,pht_s,pht_n,300.0)
    call initialize_boundaries(t_w,t_e,t_s,t_n,tt_w,tt_e,tt_s,tt_n,400.0)
    call initialize_boundaries(w_w,w_e,w_s,w_n,wt_w,wt_e,wt_s,wt_n,500.0)
    call initialize_horizontal_boundaries(mu_w,mu_e,mu_s,mu_n,mut_w,mut_e,mut_s,mut_n,600.0)

    do k=kms,kme
      c1h(k)=0.60+real(k)*0.03125;c2h(k)=0.40-real(k)*0.015625
      c1f(k)=0.55+real(k)*0.015625;c2f(k)=0.45-real(k)*0.0078125
    enddo
    fcx=(/0.0,0.7,0.4,0.0/);gcx=(/0.0,0.1,0.05,0.0/)
    config%periodic_x=periodic_x;config%nested=nested

    if(exceptional)then
      ph_s(4,1,2)=transfer(int(z'7F800000',int32),0.0)
      pht_s(4,1,2)=transfer(int(z'FF800000',int32),0.0)
      t_n(4,1,2)=transfer(int(z'80000000',int32),0.0)
      wt_w(4,1,2)=transfer(int(z'00000001',int32),0.0)
      mut(4,1)=transfer(int(z'7F7FFFFF',int32),0.0)
    endif

    call relax_bdy_dry(config,ru_tend,rv_tend,ph_tend,theta_tend,w_tend,mu_tend, &
      c1h,c2h,c1f,c2f,ru,rv,ph,theta,w,mu,mut, &
      u_w,u_e,u_s,u_n,v_w,v_e,v_s,v_n,ph_w,ph_e,ph_s,ph_n,t_w,t_e,t_s,t_n, &
      w_w,w_e,w_s,w_n,mu_w,mu_e,mu_s,mu_n, &
      ut_w,ut_e,ut_s,ut_n,vt_w,vt_e,vt_s,vt_n,pht_w,pht_e,pht_s,pht_n, &
      tt_w,tt_e,tt_s,tt_n,wt_w,wt_e,wt_s,wt_n,mut_w,mut_e,mut_s,mut_n, &
      boundary_width,spec_zone,relax_zone,0.25,fcx,gcx, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kts,kte)

    call emit_volume(name,'u',ru_tend);call emit_volume(name,'v',rv_tend)
    call emit_volume(name,'ph',ph_tend);call emit_volume(name,'t',theta_tend)
    call emit_volume(name,'w',w_tend);call emit_horizontal(name,'mu',mu_tend)
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
      write(*,'(A,1X,A,1X,Z8.8)')case_name,field_name,transfer(field(i,k,j),0_int32)
    enddo;enddo;enddo
  end subroutine

  subroutine emit_horizontal(case_name,field_name,field)
    character(len=*),intent(in)::case_name,field_name
    real,intent(in)::field(0:9,0:9)
    integer::i,j
    do j=0,9;do i=0,9
      write(*,'(A,1X,A,1X,Z8.8)')case_name,field_name,transfer(field(i,j),0_int32)
    enddo;enddo
  end subroutine
end program dry_boundary_relaxation_driver
