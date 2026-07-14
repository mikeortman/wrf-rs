program physical_boundary_driver
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_physical_boundary, only: set_physical_bc2d,set_physical_bc3d
  implicit none

  call run_volume_case('periodic_p','p',1,4,10,4,10,1,6)
  call run_volume_case('specified_u','u',2,4,10,4,10,1,6)
  call run_volume_case('nested_v','v',3,4,10,4,10,1,6)
  call run_volume_case('specified_w','w',2,4,10,4,10,1,6)
  call run_volume_case('partial_t','t',2,4,7,4,7,2,4)
  call run_volume_case('inactive_p','p',2,5,7,5,7,2,4)
  call run_horizontal_case('horizontal_periodic_t','t',1,4,10,4,10)
  call run_horizontal_case('horizontal_specified_t','t',2,4,10,4,10)
  call run_horizontal_case('horizontal_nested_t','t',3,4,10,4,10)

contains
  subroutine configure(mode,config)
    integer,intent(in)::mode
    type(grid_config_rec_type),intent(out)::config
    config=grid_config_rec_type()
    select case(mode)
    case(1)
      config%periodic_x=.true.;config%periodic_y=.true.
    case(2)
      config%specified=.true.
    case(3)
      config%nested=.true.
    end select
  end subroutine configure

  subroutine run_volume_case(name,variable,mode,its,ite,jts,jte,kts,kte)
    character(len=*),intent(in)::name
    character,intent(in)::variable
    integer,intent(in)::mode,its,ite,jts,jte,kts,kte
    integer,parameter::ims=0,ime=14,jms=0,jme=14,kms=0,kme=6
    integer,parameter::ids=4,ide=10,jds=4,jde=10,kds=1,kde=6
    real::field(ims:ime,kms:kme,jms:jme)
    type(grid_config_rec_type)::config
    integer::i,j,k
    call configure(mode,config)
    do j=jms,jme;do k=kms,kme;do i=ims,ime
      field(i,k,j)=-20.+real(i)*.7+real(k)*.11-real(j)*.3
    enddo;enddo;enddo
    field(4,2,5)=transfer(int(z'7FC0002A',int32),field(4,2,5))
    field(9,3,6)=transfer(int(z'7F800000',int32),field(9,3,6))
    field(5,4,9)=transfer(int(z'FF800000',int32),field(5,4,9))
    field(6,1,4)=transfer(int(z'80000000',int32),field(6,1,4))
    call set_physical_bc3d(field,variable,config,ids,ide,jds,jde,kds,kde, &
      ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde, &
      its,ite,jts,jte,kts,kte)
    call emit_volume(name,field)
  end subroutine run_volume_case

  subroutine run_horizontal_case(name,variable,mode,its,ite,jts,jte)
    character(len=*),intent(in)::name
    character,intent(in)::variable
    integer,intent(in)::mode,its,ite,jts,jte
    integer,parameter::ims=0,ime=14,jms=0,jme=14
    integer,parameter::ids=4,ide=10,jds=4,jde=10
    real::field(ims:ime,jms:jme)
    type(grid_config_rec_type)::config
    integer::i,j
    call configure(mode,config)
    do j=jms,jme;do i=ims,ime
      field(i,j)=-10.+real(i)*.5-real(j)*.2
    enddo;enddo
    field(4,5)=transfer(int(z'7FC0002A',int32),field(4,5))
    field(9,6)=transfer(int(z'7F800000',int32),field(9,6))
    field(5,9)=transfer(int(z'FF800000',int32),field(5,9))
    field(6,4)=transfer(int(z'80000000',int32),field(6,4))
    call set_physical_bc2d(field,variable,config,ids,ide,jds,jde, &
      ims,ime,jms,jme,ids,ide,jds,jde,its,ite,jts,jte)
    call emit_horizontal(name,field)
  end subroutine run_horizontal_case

  subroutine emit_volume(name,field)
    character(len=*),intent(in)::name
    real,intent(in)::field(0:14,0:6,0:14)
    integer::i,j,k
    do j=0,14;do k=0,6;do i=0,14
      write(*,'(A,3(1X,I0),1X,Z8.8)')name,i,k,j,transfer(field(i,k,j),0_int32)
    enddo;enddo;enddo
  end subroutine emit_volume

  subroutine emit_horizontal(name,field)
    character(len=*),intent(in)::name
    real,intent(in)::field(0:14,0:14)
    integer::i,j
    do j=0,14;do i=0,14
      write(*,'(A,2(1X,I0),1X,Z8.8)')name,i,j,transfer(field(i,j),0_int32)
    enddo;enddo
  end subroutine emit_horizontal
end program physical_boundary_driver
