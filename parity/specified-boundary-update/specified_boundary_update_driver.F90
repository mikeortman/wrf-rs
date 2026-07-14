program specified_boundary_update_driver
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_specified_boundary_update, only: spec_bdyupdate
  implicit none
  call run_case('mass_full','t',.false.,5,1,5,1,5,1,5,2)
  call run_case('west_east_full','u',.false.,5,1,5,1,5,1,5,2)
  call run_case('south_north_full','v',.false.,5,1,5,1,5,1,5,2)
  call run_case('full_level','h',.false.,5,1,5,1,5,1,5,2)
  call run_case('horizontal_mass','m',.false.,1,1,5,1,5,1,1,2)
  call run_case('periodic_mass','t',.true.,5,1,5,1,5,1,5,2)
  call run_case('partial_south_west','t',.false.,5,1,3,1,3,2,4,2)
  call run_case('interior','t',.false.,5,3,3,3,3,2,4,1)
contains
  subroutine run_case(name,variable,periodic_x,domain_top,its,ite,jts,jte,kts,kte,spec_zone)
    character(len=*),intent(in)::name
    character,intent(in)::variable
    logical,intent(in)::periodic_x
    integer,intent(in)::domain_top,its,ite,jts,jte,kts,kte,spec_zone
    integer,parameter::ims=0,ime=5,jms=0,jme=5,kms=0,kme=5
    integer,parameter::ids=1,ide=5,jds=1,jde=5,kds=1
    real::field(ims:ime,kms:kme,jms:jme),tendency(ims:ime,kms:kme,jms:jme)
    type(grid_config_rec_type)::config
    integer::i,j,k
    config%periodic_x=periodic_x
    do j=jms,jme;do k=kms,kme;do i=ims,ime
      field(i,k,j)=-20.+real(i)*.7+real(k)*.11-real(j)*.3
      tendency(i,k,j)=.5+real(i)*.02-real(k)*.03+real(j)*.04
    enddo;enddo;enddo
    call spec_bdyupdate(field,tendency,.25,variable,config,spec_zone, &
      ids,ide,jds,jde,kds,domain_top,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,domain_top,its,ite,jts,jte,kts,kte)
    call emit(name,field)
  end subroutine

  subroutine emit(name,field)
    character(len=*),intent(in)::name
    real,intent(in)::field(0:5,0:5,0:5)
    integer::i,j,k
    do j=0,5;do k=0,5;do i=0,5
      write(*,'(A,3(1X,I0),1X,Z8.8)')name,i,k,j,transfer(field(i,k,j),0_int32)
    enddo;enddo;enddo
  end subroutine
end program specified_boundary_update_driver
