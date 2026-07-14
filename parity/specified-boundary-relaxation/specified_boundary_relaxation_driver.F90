program specified_boundary_relaxation_driver
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_specified_boundary_relaxation, only: relax_bdytend, relax_bdytend_tile
  implicit none

  call run_case('mass_full','t',.false.,5,0,5,1,8,1,8,1,4,1,3,.false.,.false.)
  call run_case('west_east_full','U',.false.,5,0,5,1,9,1,8,1,4,1,3,.false.,.false.)
  call run_case('south_north_full','V',.false.,5,0,5,1,8,1,9,1,4,1,3,.false.,.false.)
  call run_case('full_level_tile_south_west','H',.false.,5,0,5,1,5,1,5,2,4,1,3,.true.,.false.)
  call run_case('full_level_tile_north_east','h',.false.,5,0,5,4,8,4,8,1,5,1,3,.true.,.false.)
  call run_case('horizontal_mass','M',.false.,1,1,1,1,8,1,8,1,1,1,3,.false.,.false.)
  call run_case('periodic_mass','t',.true.,5,0,5,1,8,1,8,1,4,1,3,.false.,.false.)
  call run_case('inactive_interior','t',.false.,5,0,5,4,5,4,5,1,4,1,3,.false.,.false.)
  call run_case('empty_relaxation_band','t',.false.,5,0,5,1,8,1,8,1,4,2,2,.false.,.false.)
  call run_case('exceptional','t',.false.,5,0,5,1,8,1,8,1,4,1,3,.false.,.true.)
contains
  subroutine run_case(name,variable,periodic_x,domain_top,kms,kme,its,ite,jts,jte,kts,kte, &
                      spec_zone,relax_zone,tile_field,exceptional)
    character(len=*),intent(in)::name
    character,intent(in)::variable
    logical,intent(in)::periodic_x,tile_field,exceptional
    integer,intent(in)::domain_top,kms,kme,its,ite,jts,jte,kts,kte,spec_zone,relax_zone
    integer,parameter::ims=0,ime=9,jms=0,jme=9
    integer,parameter::ids=1,ide=9,jds=1,jde=9,kds=1,boundary_width=4
    real,parameter::dtbc=0.25
    real,allocatable::field_full(:,:,:),field_tile_values(:,:,:),field_tend(:,:,:)
    real,allocatable::west(:,:,:),east(:,:,:),south(:,:,:),north(:,:,:)
    real,allocatable::west_tend(:,:,:),east_tend(:,:,:),south_tend(:,:,:),north_tend(:,:,:)
    real::fcx(boundary_width),gcx(boundary_width)
    type(grid_config_rec_type)::config
    integer::i,j,k,width,ixs,ixe,jxs,jxe,kxs,kxe

    allocate(field_full(ims:ime,kms:kme,jms:jme))
    allocate(field_tend(ims:ime,kms:kme,jms:jme))
    allocate(west(jms:jme,kds:domain_top,boundary_width))
    allocate(east(jms:jme,kds:domain_top,boundary_width))
    allocate(south(ims:ime,kds:domain_top,boundary_width))
    allocate(north(ims:ime,kds:domain_top,boundary_width))
    allocate(west_tend(jms:jme,kds:domain_top,boundary_width))
    allocate(east_tend(jms:jme,kds:domain_top,boundary_width))
    allocate(south_tend(ims:ime,kds:domain_top,boundary_width))
    allocate(north_tend(ims:ime,kds:domain_top,boundary_width))

    do j=jms,jme;do k=kms,kme;do i=ims,ime
      field_full(i,k,j)=((50.0+real(i)*0.5)+real(k)*0.25)-real(j)*0.125
      field_tend(i,k,j)=((-20.0+real(i)*0.25)+real(k)*0.0625)-real(j)*0.5
    enddo;enddo;enddo
    do width=1,boundary_width;do k=kds,domain_top
      do j=jms,jme
        west(j,k,width)=((100.0+real(j)*0.5)+real(k)*0.25)+real(width)*0.03125
        east(j,k,width)=((200.0+real(j)*0.5)+real(k)*0.25)+real(width)*0.03125
        west_tend(j,k,width)=((-3.0+real(j)*0.125)+real(k)*0.0625)+real(width)*0.015625
        east_tend(j,k,width)=((4.0+real(j)*0.125)+real(k)*0.0625)+real(width)*0.015625
      enddo
      do i=ims,ime
        south(i,k,width)=((300.0+real(i)*0.5)+real(k)*0.25)+real(width)*0.03125
        north(i,k,width)=((400.0+real(i)*0.5)+real(k)*0.25)+real(width)*0.03125
        south_tend(i,k,width)=((-5.0+real(i)*0.125)+real(k)*0.0625)+real(width)*0.015625
        north_tend(i,k,width)=((6.0+real(i)*0.125)+real(k)*0.0625)+real(width)*0.015625
      enddo
    enddo;enddo

    fcx=(/0.0,0.7,0.4,0.0/)
    gcx=(/0.0,0.1,0.05,0.0/)
    config%periodic_x=periodic_x
    if(exceptional)then
      south(4,1,2)=transfer(int(z'80000000',int32),0.0)
      south_tend(4,1,2)=transfer(int(z'00000001',int32),0.0)
      north(4,1,2)=transfer(int(z'7F7FFFFF',int32),0.0)
      west(4,1,2)=transfer(int(z'7F800000',int32),0.0)
      east_tend(4,1,2)=transfer(int(z'FF800000',int32),0.0)
    endif

    if(tile_field)then
      ixs=max(ims,its-1);ixe=min(ime,ite+1)
      jxs=max(jms,jts-1);jxe=min(jme,jte+1)
      kxs=kts;kxe=kte
      allocate(field_tile_values(ixs:ixe,kxs:kxe,jxs:jxe))
      do j=jxs,jxe;do k=kxs,kxe;do i=ixs,ixe
        field_tile_values(i,k,j)=((50.0+real(i)*0.5)+real(k)*0.25)-real(j)*0.125
      enddo;enddo;enddo
      call relax_bdytend_tile(field_tile_values,field_tend,west,east,south,north, &
        west_tend,east_tend,south_tend,north_tend,variable,config,boundary_width, &
        spec_zone,relax_zone,dtbc,fcx,gcx,ids,ide,jds,jde,kds,domain_top, &
        ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,domain_top, &
        its,ite,jts,jte,kts,kte,ixs,ixe,jxs,jxe,kxs,kxe)
      deallocate(field_tile_values)
    else
      call relax_bdytend(field_full,field_tend,west,east,south,north, &
        west_tend,east_tend,south_tend,north_tend,variable,config,boundary_width, &
        spec_zone,relax_zone,dtbc,fcx,gcx,ids,ide,jds,jde,kds,domain_top, &
        ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,domain_top, &
        its,ite,jts,jte,kts,kte)
    endif
    call emit(name,field_tend)
    deallocate(field_full,field_tend,west,east,south,north)
    deallocate(west_tend,east_tend,south_tend,north_tend)
  end subroutine

  subroutine emit(name,field)
    character(len=*),intent(in)::name
    real,intent(in)::field(:,:,:)
    integer::i,j,k
    do j=1,size(field,3);do k=1,size(field,2);do i=1,size(field,1)
      write(*,'(A,1X,Z8.8)')name,transfer(field(i,k,j),0_int32)
    enddo;enddo;enddo
  end subroutine
end program specified_boundary_relaxation_driver
