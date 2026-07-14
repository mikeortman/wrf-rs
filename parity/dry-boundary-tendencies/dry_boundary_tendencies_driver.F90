program dry_boundary_tendencies_driver
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_dry_boundary_tendencies, only: spec_bdy_dry
  implicit none

  call run_case('full_global',.false.,.false.,1,8,1,8,1,5,2,.false.)
  call run_case('full_nested',.false.,.true.,1,8,1,8,1,5,2,.false.)
  call run_case('periodic_nested',.true.,.true.,1,8,1,8,1,5,2,.false.)
  call run_case('south_west',.false.,.true.,1,5,1,5,1,5,2,.false.)
  call run_case('north_east',.false.,.true.,4,8,4,8,1,5,2,.false.)
  call run_case('partial_vertical',.false.,.true.,1,8,1,8,2,3,2,.false.)
  call run_case('inactive',.false.,.true.,4,5,4,5,1,5,2,.false.)
  call run_case('zero_zone',.false.,.true.,1,8,1,8,1,5,0,.false.)
  call run_case('exceptional',.false.,.true.,1,8,1,8,1,5,2,.true.)
contains
  subroutine run_case(name,periodic_x,nested,its,ite,jts,jte,kts,kte,spec_zone,exceptional)
    character(len=*),intent(in)::name
    logical,intent(in)::periodic_x,nested,exceptional
    integer,intent(in)::its,ite,jts,jte,kts,kte,spec_zone
    integer,parameter::ims=0,ime=9,jms=0,jme=9,kms=0,kme=5
    integer,parameter::ids=1,ide=9,jds=1,jde=9,kds=1,kde=5
    integer,parameter::boundary_width=3,volume_fields=5
    real::output(ims:ime,kms:kme,jms:jme,volume_fields)
    real::mu_output(ims:ime,jms:jme)
    real::state_w(jms:jme,kds:kde,boundary_width)
    real::state_e(jms:jme,kds:kde,boundary_width)
    real::state_s(ims:ime,kds:kde,boundary_width)
    real::state_n(ims:ime,kds:kde,boundary_width)
    real::tend_w(jms:jme,kds:kde,boundary_width,volume_fields)
    real::tend_e(jms:jme,kds:kde,boundary_width,volume_fields)
    real::tend_s(ims:ime,kds:kde,boundary_width,volume_fields)
    real::tend_n(ims:ime,kds:kde,boundary_width,volume_fields)
    real::mu_state_w(jms:jme,1:1,boundary_width)
    real::mu_state_e(jms:jme,1:1,boundary_width)
    real::mu_state_s(ims:ime,1:1,boundary_width)
    real::mu_state_n(ims:ime,1:1,boundary_width)
    real::mu_tend_w(jms:jme,1:1,boundary_width)
    real::mu_tend_e(jms:jme,1:1,boundary_width)
    real::mu_tend_s(ims:ime,1:1,boundary_width)
    real::mu_tend_n(ims:ime,1:1,boundary_width)
    type(grid_config_rec_type)::config
    integer::i,j,k,distance,field

    do field=1,volume_fields;do j=jms,jme;do k=kms,kme;do i=ims,ime
      output(i,k,j,field)=(-1000.0*real(field)+real(i)*11.0)+real(k)*0.25-real(j)*3.0
    enddo;enddo;enddo;enddo
    do j=jms,jme;do i=ims,ime
      mu_output(i,j)=-6000.0+real(i)*7.0-real(j)*2.0
    enddo;enddo

    state_w=-999.0;state_e=-999.0;state_s=-999.0;state_n=-999.0
    mu_state_w=-999.0;mu_state_e=-999.0;mu_state_s=-999.0;mu_state_n=-999.0
    do field=1,volume_fields;do distance=1,boundary_width;do k=kds,kde
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
      tend_s(2,1,1,1)=transfer(int(z'80000000',int32),0.0)
      tend_n(2,1,1,2)=transfer(int(z'7F800000',int32),0.0)
      tend_w(2,1,1,3)=transfer(int(z'FF800000',int32),0.0)
      tend_e(2,1,1,4)=transfer(int(z'00000001',int32),0.0)
      tend_s(3,1,1,5)=transfer(int(z'7FC12345',int32),0.0)
      mu_tend_n(2,1,1)=transfer(int(z'7F7FFFFF',int32),0.0)
    endif
    config%periodic_x=periodic_x;config%nested=nested

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
      mu_tend_w,mu_tend_e,mu_tend_s,mu_tend_n, &
      boundary_width,spec_zone,ids,ide,jds,jde,kds,kde, &
      ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde, &
      its,ite,jts,jte,kts,kte)

    call emit_volume(name,'u',output(:,:,:,1))
    call emit_volume(name,'v',output(:,:,:,2))
    call emit_volume(name,'ph',output(:,:,:,3))
    call emit_volume(name,'t',output(:,:,:,4))
    call emit_volume(name,'w',output(:,:,:,5))
    call emit_horizontal(name,'mu',mu_output)
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
end program dry_boundary_tendencies_driver
