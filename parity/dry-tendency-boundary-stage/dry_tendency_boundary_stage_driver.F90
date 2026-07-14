program dry_tendency_boundary_stage_driver
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_dry_boundary_tendencies, only: spec_bdy_dry
  implicit none

  call run_case('first_global',1,.false.,.false.,1,5,1,5,1,4,1,.false.)
  call run_case('first_nested',1,.true.,.false.,1,5,1,5,1,4,1,.false.)
  call run_case('later_partial',2,.true.,.false.,2,4,2,4,1,3,1,.false.)
  call run_case('periodic_nested',1,.true.,.true.,1,5,1,5,1,4,1,.false.)
  call run_case('exceptional',1,.true.,.false.,1,5,1,5,1,4,1,.true.)

contains

  subroutine run_case(name,rk_step,nested,periodic_x,its,ite,jts,jte,kts,kte,spec_zone,exceptional)
    character(len=*),intent(in)::name
    integer,intent(in)::rk_step,its,ite,jts,jte,kts,kte,spec_zone
    logical,intent(in)::nested,periodic_x,exceptional
    integer,parameter::ims=0,ime=5,jms=0,jme=5,kms=0,kme=4
    integer,parameter::ids=1,ide=5,jds=1,jde=5,kds=1,kde=4
    integer,parameter::boundary_width=2,boundary_fields=5
    real::ru_tend(ims:ime,kms:kme,jms:jme),rv_tend(ims:ime,kms:kme,jms:jme)
    real::rw_tend(ims:ime,kms:kme,jms:jme),ph_tend(ims:ime,kms:kme,jms:jme)
    real::t_tend(ims:ime,kms:kme,jms:jme),ru_tendf(ims:ime,kms:kme,jms:jme)
    real::rv_tendf(ims:ime,kms:kme,jms:jme),rw_tendf(ims:ime,kms:kme,jms:jme)
    real::ph_tendf(ims:ime,kms:kme,jms:jme),t_tendf(ims:ime,kms:kme,jms:jme)
    real::u_save(ims:ime,kms:kme,jms:jme),v_save(ims:ime,kms:kme,jms:jme)
    real::w_save(ims:ime,kms:kme,jms:jme),ph_save(ims:ime,kms:kme,jms:jme)
    real::t_save(ims:ime,kms:kme,jms:jme),h_diabatic(ims:ime,kms:kme,jms:jme)
    real::mu_tend(ims:ime,jms:jme),mu_tendf(ims:ime,jms:jme),mut(ims:ime,jms:jme)
    real::msftx(ims:ime,jms:jme),msfty(ims:ime,jms:jme),msfux(ims:ime,jms:jme)
    real::msfuy(ims:ime,jms:jme),msfvx(ims:ime,jms:jme)
    real::msfvx_inv(ims:ime,jms:jme),msfvy(ims:ime,jms:jme)
    real::c1(kms:kme),c2(kms:kme)
    real::state_w(jms:jme,kds:kde,boundary_width),state_e(jms:jme,kds:kde,boundary_width)
    real::state_s(ims:ime,kds:kde,boundary_width),state_n(ims:ime,kds:kde,boundary_width)
    real::tend_w(jms:jme,kds:kde,boundary_width,boundary_fields)
    real::tend_e(jms:jme,kds:kde,boundary_width,boundary_fields)
    real::tend_s(ims:ime,kds:kde,boundary_width,boundary_fields)
    real::tend_n(ims:ime,kds:kde,boundary_width,boundary_fields)
    real::mu_state_w(jms:jme,1:1,boundary_width),mu_state_e(jms:jme,1:1,boundary_width)
    real::mu_state_s(ims:ime,1:1,boundary_width),mu_state_n(ims:ime,1:1,boundary_width)
    real::mu_tend_w(jms:jme,1:1,boundary_width),mu_tend_e(jms:jme,1:1,boundary_width)
    real::mu_tend_s(ims:ime,1:1,boundary_width),mu_tend_n(ims:ime,1:1,boundary_width)
    type(grid_config_rec_type)::config
    integer::i,j,k,distance,field

    do k=kms,kme
      c1(k)=0.2+real(k)*0.03;c2(k)=0.4-real(k)*0.02
    enddo
    do j=jms,jme;do i=ims,ime
      mu_tend(i,j)=0.6+real(i)*0.07-real(j)*0.03
      mu_tendf(i,j)=-0.2+real(i)*0.02+real(j)*0.04
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
      enddo
    enddo;enddo

    state_w=-999.0;state_e=-999.0;state_s=-999.0;state_n=-999.0
    mu_state_w=-999.0;mu_state_e=-999.0;mu_state_s=-999.0;mu_state_n=-999.0
    do field=1,boundary_fields;do distance=1,boundary_width;do k=kds,kde
      do j=jms,jme
        tend_w(j,k,distance,field)=1000.0*real(field)+100.0+real(j)*10.0+real(k)+real(distance)*0.01
        tend_e(j,k,distance,field)=1000.0*real(field)+200.0+real(j)*10.0+real(k)+real(distance)*0.01
      enddo
      do i=ims,ime
        tend_s(i,k,distance,field)=1000.0*real(field)+300.0+real(i)*10.0+real(k)+real(distance)*0.01
        tend_n(i,k,distance,field)=1000.0*real(field)+400.0+real(i)*10.0+real(k)+real(distance)*0.01
      enddo
    enddo;enddo;enddo
    do distance=1,boundary_width
      do j=jms,jme
        mu_tend_w(j,1,distance)=6100.0+real(j)*10.0+real(distance)*0.01
        mu_tend_e(j,1,distance)=6200.0+real(j)*10.0+real(distance)*0.01
      enddo
      do i=ims,ime
        mu_tend_s(i,1,distance)=6300.0+real(i)*10.0+real(distance)*0.01
        mu_tend_n(i,1,distance)=6400.0+real(i)*10.0+real(distance)*0.01
      enddo
    enddo

    if(exceptional)then
      msfty(1,1)=0.0;msfuy(2,1)=-0.0;msfvx_inv(1,2)=huge(msfvx_inv)*2.0
      h_diabatic(2,1,2)=huge(h_diabatic)*2.0;ru_tendf(2,1,1)=-0.0
      ph_tendf(1,1,1)=huge(ph_tendf)
      tend_s(2,1,1,1)=transfer(int(z'80000000',int32),0.0)
      tend_n(2,1,1,2)=transfer(int(z'7F800000',int32),0.0)
      tend_w(2,1,1,3)=transfer(int(z'FF800000',int32),0.0)
      tend_e(2,1,1,4)=transfer(int(z'00000001',int32),0.0)
      tend_s(3,1,1,5)=transfer(int(z'7FC12345',int32),0.0)
      mu_tend_n(2,1,1)=transfer(int(z'7F7FFFFF',int32),0.0)
    endif

    call rk_addtend_dry(ru_tend,rv_tend,rw_tend,ph_tend,t_tend, &
      ru_tendf,rv_tendf,rw_tendf,ph_tendf,t_tendf,u_save,v_save,w_save,ph_save,t_save, &
      mu_tend,mu_tendf,rk_step,c1,c2,h_diabatic,mut,msftx,msfty,msfux,msfuy, &
      msfvx,msfvx_inv,msfvy,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kts,kte)

    config%periodic_x=periodic_x;config%nested=nested
    call spec_bdy_dry(config,ru_tend,rv_tend,ph_tend,t_tend,rw_tend,mu_tend, &
      state_w,state_e,state_s,state_n,state_w,state_e,state_s,state_n, &
      state_w,state_e,state_s,state_n,state_w,state_e,state_s,state_n, &
      state_w,state_e,state_s,state_n,mu_state_w,mu_state_e,mu_state_s,mu_state_n, &
      tend_w(:,:,:,1),tend_e(:,:,:,1),tend_s(:,:,:,1),tend_n(:,:,:,1), &
      tend_w(:,:,:,2),tend_e(:,:,:,2),tend_s(:,:,:,2),tend_n(:,:,:,2), &
      tend_w(:,:,:,3),tend_e(:,:,:,3),tend_s(:,:,:,3),tend_n(:,:,:,3), &
      tend_w(:,:,:,4),tend_e(:,:,:,4),tend_s(:,:,:,4),tend_n(:,:,:,4), &
      tend_w(:,:,:,5),tend_e(:,:,:,5),tend_s(:,:,:,5),tend_n(:,:,:,5), &
      mu_tend_w,mu_tend_e,mu_tend_s,mu_tend_n,boundary_width,spec_zone, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde, &
      its,ite,jts,jte,kts,kte)

    call emit_volume(name,'ru_tend',ru_tend);call emit_volume(name,'rv_tend',rv_tend)
    call emit_volume(name,'rw_tend',rw_tend);call emit_volume(name,'ph_tend',ph_tend)
    call emit_volume(name,'t_tend',t_tend);call emit_volume(name,'ru_tendf',ru_tendf)
    call emit_volume(name,'rv_tendf',rv_tendf);call emit_volume(name,'rw_tendf',rw_tendf)
    call emit_volume(name,'ph_tendf',ph_tendf);call emit_volume(name,'t_tendf',t_tendf)
    call emit_horizontal(name,'mu_tend',mu_tend);call emit_horizontal(name,'mu_tendf',mu_tendf)
  end subroutine

  subroutine emit_volume(case_name,field_name,field)
    character(len=*),intent(in)::case_name,field_name
    real,intent(in)::field(0:5,0:4,0:5)
    integer::i,j,k
    do j=0,5;do k=0,4;do i=0,5
      if(isnan(field(i,k,j)))then
        write(*,'(A,1X,A,1X,A)')case_name,field_name,'NAN'
      else
        write(*,'(A,1X,A,1X,Z8.8)')case_name,field_name,transfer(field(i,k,j),0_int32)
      endif
    enddo;enddo;enddo
  end subroutine

  subroutine emit_horizontal(case_name,field_name,field)
    character(len=*),intent(in)::case_name,field_name
    real,intent(in)::field(0:5,0:5)
    integer::i,j
    do j=0,5;do i=0,5
      if(isnan(field(i,j)))then
        write(*,'(A,1X,A,1X,A)')case_name,field_name,'NAN'
      else
        write(*,'(A,1X,A,1X,Z8.8)')case_name,field_name,transfer(field(i,j),0_int32)
      endif
    enddo;enddo
  end subroutine
end program
