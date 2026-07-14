program specified_boundary_tendencies_driver
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_specified_boundary_tendencies, only: spec_bdytend
  implicit none
  call run_case('mass_full','t',.false.,5,1,5,1,5,1,5,2,.false.)
  call run_case('west_east_full','u',.false.,5,1,5,1,5,1,5,2,.false.)
  call run_case('south_north_full','v',.false.,5,1,5,1,5,1,5,2,.false.)
  call run_case('full_level','h',.false.,5,1,5,1,5,1,5,2,.false.)
  call run_case('horizontal_mass','m',.false.,1,1,5,1,5,1,1,2,.false.)
  call run_case('periodic_mass','t',.true.,5,1,5,1,5,1,5,2,.false.)
  call run_case('partial_south_west','t',.false.,5,1,3,1,3,2,2,2,.false.)
  call run_case('partial_north_east','t',.false.,5,3,5,3,5,2,2,2,.false.)
  call run_case('inactive_interior','t',.false.,5,3,3,3,3,2,2,1,.false.)
  call run_case('full_level_partial_vertical','h',.false.,5,1,5,1,5,2,3,2,.false.)
  call run_case('exceptional','t',.false.,5,1,5,1,5,1,5,2,.true.)
  call run_case('zero_zone','t',.false.,5,1,5,1,5,1,5,0,.false.)
contains
  subroutine run_case(name,variable,periodic_x,domain_top,its,ite,jts,jte,kts,kte,spec_zone,exceptional)
    character(len=*),intent(in)::name
    character,intent(in)::variable
    logical,intent(in)::periodic_x,exceptional
    integer,intent(in)::domain_top,its,ite,jts,jte,kts,kte,spec_zone
    integer,parameter::ims=0,ime=5,jms=0,jme=5,kms=0,kme=5
    integer,parameter::ids=1,ide=5,jds=1,jde=5,kds=1,boundary_width=3
    real::field_tend(ims:ime,kms:kme,jms:jme)
    real,allocatable::west(:,:,:),east(:,:,:),south(:,:,:),north(:,:,:)
    real,allocatable::west_tend(:,:,:),east_tend(:,:,:),south_tend(:,:,:),north_tend(:,:,:)
    type(grid_config_rec_type)::config
    integer::i,j,k,width
    allocate(west(jms:jme,kds:domain_top,boundary_width))
    allocate(east(jms:jme,kds:domain_top,boundary_width))
    allocate(south(ims:ime,kds:domain_top,boundary_width))
    allocate(north(ims:ime,kds:domain_top,boundary_width))
    allocate(west_tend(jms:jme,kds:domain_top,boundary_width))
    allocate(east_tend(jms:jme,kds:domain_top,boundary_width))
    allocate(south_tend(ims:ime,kds:domain_top,boundary_width))
    allocate(north_tend(ims:ime,kds:domain_top,boundary_width))
    config%periodic_x=periodic_x
    do j=jms,jme;do k=kms,kme;do i=ims,ime
      field_tend(i,k,j)=-7000.+real(i)*11.+real(k)*.25-real(j)*3.
    enddo;enddo;enddo
    west=-999.;east=-999.;south=-999.;north=-999.
    do width=1,boundary_width;do k=kds,domain_top
      do j=jms,jme
        west_tend(j,k,width)=1000.+real(j)*10.+real(k)+real(width)*.01
        east_tend(j,k,width)=2000.+real(j)*10.+real(k)+real(width)*.01
      enddo
      do i=ims,ime
        south_tend(i,k,width)=3000.+real(i)*10.+real(k)+real(width)*.01
        north_tend(i,k,width)=4000.+real(i)*10.+real(k)+real(width)*.01
      enddo
    enddo;enddo
    if(exceptional)then
      south_tend(2,1,1)=transfer(int(z'80000000',int32),0.0)
      north_tend(2,1,1)=transfer(int(z'7F800000',int32),0.0)
      west_tend(2,1,1)=transfer(int(z'FF800000',int32),0.0)
      east_tend(2,1,1)=transfer(int(z'00000001',int32),0.0)
    endif
    call spec_bdytend(field_tend,west,east,south,north, &
      west_tend,east_tend,south_tend,north_tend,variable,config, &
      boundary_width,spec_zone,ids,ide,jds,jde,kds,domain_top, &
      ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,domain_top, &
      its,ite,jts,jte,kts,kte)
    call emit(name,field_tend)
    deallocate(west,east,south,north,west_tend,east_tend,south_tend,north_tend)
  end subroutine

  subroutine emit(name,field)
    character(len=*),intent(in)::name
    real,intent(in)::field(0:5,0:5,0:5)
    integer::i,j,k
    do j=0,5;do k=0,5;do i=0,5
      write(*,'(A,3(1X,I0),1X,Z8.8)')name,i,k,j,transfer(field(i,k,j),0_int32)
    enddo;enddo;enddo
  end subroutine
end program specified_boundary_tendencies_driver
