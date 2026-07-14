program flow_dependent_inflow_policies_driver
  use iso_fortran_env, only: int32
  use ieee_arithmetic, only: ieee_positive_inf, ieee_value
  use module_configure, only: grid_config_rec_type
  use extracted_flow_dependent_inflow_policies, only: flow_dep_bdy_qnn, flow_dep_bdy_fixed_inflow
  implicit none
  call run_qnn('qnn_full',.false.,1,7,1,7,1,6,2,73.5)
  call run_qnn('qnn_periodic',.true.,1,7,1,7,1,6,2,-0.)
  call run_qnn('qnn_partial',.false.,1,4,1,4,2,4,2,ieee_value(0.,ieee_positive_inf))
  call run_fixed('fixed_full',.false.,1,7,1,7,1,6,2)
  call run_fixed('fixed_periodic',.true.,1,7,1,7,1,6,2)
  call run_fixed('fixed_partial',.false.,4,7,4,7,2,4,2)
contains
  subroutine run_qnn(name,periodic_x,its,ite,jts,jte,kts,kte,spec_zone,ccn_conc)
    character(len=*),intent(in)::name
    logical,intent(in)::periodic_x
    integer,intent(in)::its,ite,jts,jte,kts,kte,spec_zone
    real,intent(in)::ccn_conc
    real::field(0:7,0:7,0:7),u(0:7,0:7,0:7),v(0:7,0:7,0:7)
    type(grid_config_rec_type)::config
    call initialize(field,u,v)
    config%periodic_x=periodic_x
    call flow_dep_bdy_qnn(field,u,v,config,spec_zone,ccn_conc, &
      1,7,1,7,1,7,0,7,0,7,0,7,1,7,1,7,1,7, &
      its,ite,jts,jte,kts,kte)
    call emit(name,field)
  end subroutine

  subroutine run_fixed(name,periodic_x,its,ite,jts,jte,kts,kte,spec_zone)
    character(len=*),intent(in)::name
    logical,intent(in)::periodic_x
    integer,intent(in)::its,ite,jts,jte,kts,kte,spec_zone
    real::field(0:7,0:7,0:7),u(0:7,0:7,0:7),v(0:7,0:7,0:7)
    type(grid_config_rec_type)::config
    call initialize(field,u,v)
    config%periodic_x=periodic_x
    call flow_dep_bdy_fixed_inflow(field,u,v,config,spec_zone, &
      1,7,1,7,1,7,0,7,0,7,0,7,1,7,1,7,1,7, &
      its,ite,jts,jte,kts,kte)
    call emit(name,field)
  end subroutine

  subroutine initialize(field,u,v)
    real,intent(out)::field(0:7,0:7,0:7),u(0:7,0:7,0:7),v(0:7,0:7,0:7)
    integer::i,j,k
    do j=0,7;do k=0,7;do i=0,7
      field(i,k,j)=-40.+real(i)*.7+real(k)*.11-real(j)*.3
      if (mod(i+j+k,2)==0) then;u(i,k,j)=-1.;else;u(i,k,j)=1.;endif
      if (mod(i+2*j+k,3)==0) then;v(i,k,j)=-1.;else;v(i,k,j)=1.;endif
    enddo;enddo;enddo
  end subroutine

  subroutine emit(name,field)
    character(len=*),intent(in)::name
    real,intent(in)::field(0:7,0:7,0:7)
    integer::i,j,k
    do j=0,7;do k=0,7;do i=0,7
      write(*,'(A,3(1X,I0),1X,Z8.8)')name,i,k,j,transfer(field(i,k,j),0_int32)
    enddo;enddo;enddo
  end subroutine
end program flow_dependent_inflow_policies_driver
