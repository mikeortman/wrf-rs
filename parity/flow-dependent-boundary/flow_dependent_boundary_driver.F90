program flow_dependent_boundary_driver
  use iso_fortran_env, only: int32
  use ieee_arithmetic, only: ieee_positive_inf, ieee_negative_inf, ieee_quiet_nan, ieee_value
  use module_configure, only: grid_config_rec_type
  use extracted_flow_dependent_boundary, only: flow_dep_bdy
  implicit none
  call run_case('mixed_full',.false.,1,7,1,7,1,6,2,.false.)
  call run_case('periodic_mixed',.true.,1,7,1,7,1,6,2,.false.)
  call run_case('partial_south_west',.false.,1,4,1,4,2,4,2,.false.)
  call run_case('partial_north_east',.false.,4,7,4,7,2,4,2,.false.)
  call run_case('interior',.false.,4,4,4,4,2,4,1,.false.)
  call run_case('exceptional_signs',.false.,1,7,1,7,1,6,2,.true.)
contains
  subroutine run_case(name,periodic_x,its,ite,jts,jte,kts,kte,spec_zone,exceptional)
    character(len=*),intent(in)::name
    logical,intent(in)::periodic_x,exceptional
    integer,intent(in)::its,ite,jts,jte,kts,kte,spec_zone
    integer,parameter::ims=0,ime=7,jms=0,jme=7,kms=0,kme=7
    integer,parameter::ids=1,ide=7,jds=1,jde=7,kds=1,kde=7
    real::field(ims:ime,kms:kme,jms:jme)
    real::u(ims:ime,kms:kme,jms:jme),v(ims:ime,kms:kme,jms:jme)
    type(grid_config_rec_type)::config
    integer::i,j,k
    config%periodic_x=periodic_x
    do j=jms,jme;do k=kms,kme;do i=ims,ime
      field(i,k,j)=-40.+real(i)*.7+real(k)*.11-real(j)*.3
      if (mod(i+j+k,2)==0) then
        u(i,k,j)=-1.
      else
        u(i,k,j)=1.
      endif
      if (mod(i+2*j+k,3)==0) then
        v(i,k,j)=-1.
      else
        v(i,k,j)=1.
      endif
    enddo;enddo;enddo
    if (exceptional) call set_exceptional_velocities(u,v)
    call flow_dep_bdy(field,u,v,config,spec_zone, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kts,kte)
    call emit(name,field)
  end subroutine

  subroutine set_exceptional_velocities(u,v)
    real,intent(inout)::u(0:7,0:7,0:7),v(0:7,0:7,0:7)
    u(1,1,3)=-0.
    u(2,1,3)=ieee_value(0.,ieee_quiet_nan)
    u(6,1,3)=ieee_value(0.,ieee_positive_inf)
    u(7,1,3)=ieee_value(0.,ieee_negative_inf)
    v(1,1,1)=-0.
    v(2,1,1)=ieee_value(0.,ieee_negative_inf)
    v(3,1,1)=ieee_value(0.,ieee_quiet_nan)
    v(1,1,7)=ieee_value(0.,ieee_positive_inf)
  end subroutine

  subroutine emit(name,field)
    character(len=*),intent(in)::name
    real,intent(in)::field(0:7,0:7,0:7)
    integer::i,j,k
    do j=0,7;do k=0,7;do i=0,7
      write(*,'(A,3(1X,I0),1X,Z8.8)')name,i,k,j,transfer(field(i,k,j),0_int32)
    enddo;enddo;enddo
  end subroutine
end program flow_dependent_boundary_driver
