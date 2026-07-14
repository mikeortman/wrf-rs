program zero_gradient_boundary_driver
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_zero_gradient_boundary, only: zero_grad_bdy
  implicit none
  call run_case('vertical_full','w',.false.,1,7,1,7,1,7,2)
  call run_case('mass_half','t',.false.,1,7,1,7,1,7,2)
  call run_case('west_east','u',.false.,1,7,1,7,1,7,2)
  call run_case('south_north','v',.false.,1,7,1,7,1,7,2)
  call run_case('periodic_vertical','w',.true.,1,7,1,7,1,7,2)
  call run_case('partial_south_west','w',.false.,1,4,1,4,2,6,2)
  call run_case('interior','w',.false.,4,4,4,4,2,6,1)
contains
  subroutine run_case(name,variable,periodic_x,its,ite,jts,jte,kts,kte,spec_zone)
    character(len=*),intent(in)::name
    character,intent(in)::variable
    logical,intent(in)::periodic_x
    integer,intent(in)::its,ite,jts,jte,kts,kte,spec_zone
    integer,parameter::ims=0,ime=7,jms=0,jme=7,kms=0,kme=7
    integer,parameter::ids=1,ide=7,jds=1,jde=7,kds=1,kde=7
    real::field(ims:ime,kms:kme,jms:jme)
    type(grid_config_rec_type)::config
    integer::i,j,k
    config%periodic_x=periodic_x
    do j=jms,jme;do k=kms,kme;do i=ims,ime
      field(i,k,j)=-50.+real(i)*.7+real(k)*.11-real(j)*.3
    enddo;enddo;enddo
    call zero_grad_bdy(field,variable,config,spec_zone, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kts,kte)
    call emit(name,field)
  end subroutine

  subroutine emit(name,field)
    character(len=*),intent(in)::name
    real,intent(in)::field(0:7,0:7,0:7)
    integer::i,j,k
    do j=0,7;do k=0,7;do i=0,7
      write(*,'(A,3(1X,I0),1X,Z8.8)')name,i,k,j,transfer(field(i,k,j),0_int32)
    enddo;enddo;enddo
  end subroutine
end program zero_gradient_boundary_driver
