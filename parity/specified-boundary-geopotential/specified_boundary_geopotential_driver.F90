program specified_boundary_geopotential_driver
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_specified_boundary_geopotential, only: spec_bdyupdate_ph
  implicit none
  call run_case('full_level_full','h',.false.,5,1,5,1,5,1,5,2,.false.)
  call run_case('mass_half_full','t',.false.,5,1,5,1,5,1,5,2,.false.)
  call run_case('west_east_full','u',.false.,5,1,5,1,5,1,5,2,.false.)
  call run_case('south_north_full','v',.false.,5,1,5,1,5,1,5,2,.false.)
  call run_case('horizontal_mass','m',.false.,1,1,5,1,5,1,1,2,.false.)
  call run_case('periodic_full','h',.true.,5,1,5,1,5,1,5,2,.false.)
  call run_case('partial_south_west','h',.false.,5,1,3,1,3,2,4,2,.false.)
  call run_case('interior','h',.false.,5,3,3,3,3,2,4,1,.false.)
  call run_case('exceptional_full','h',.false.,5,1,5,1,5,1,5,2,.true.)
contains
  subroutine run_case(name,variable,periodic_x,domain_top,its,ite,jts,jte,kts,kte,spec_zone,exceptional)
    character(len=*),intent(in)::name
    character,intent(in)::variable
    logical,intent(in)::periodic_x,exceptional
    integer,intent(in)::domain_top,its,ite,jts,jte,kts,kte,spec_zone
    integer,parameter::ims=0,ime=5,jms=0,jme=5,kms=0,kme=5
    integer,parameter::ids=1,ide=5,jds=1,jde=5,kds=1
    real::field(ims:ime,kms:kme,jms:jme),field_tend(ims:ime,kms:kme,jms:jme)
    real::ph_save(ims:ime,kms:kme,jms:jme),mu_tend(ims:ime,jms:jme),muts(ims:ime,jms:jme)
    real::c1(kms:kme),c2(kms:kme)
    type(grid_config_rec_type)::config
    integer::i,j,k
    config%periodic_x=periodic_x
    do k=kms,kme
      c1(k)=.4+real(k)*.05;c2(k)=2.+real(k)*.1
    enddo
    do j=jms,jme;do i=ims,ime
      muts(i,j)=10.+real(i)*.4-real(j)*.1
      mu_tend(i,j)=.3+real(i)*.02+real(j)*.01
      do k=kms,kme
        field(i,k,j)=-200.+real(i)*.7+real(k)*.11-real(j)*.3
        field_tend(i,k,j)=1.5+real(i)*.02-real(k)*.03+real(j)*.04
        ph_save(i,k,j)=100.+real(i)*.5+real(k)*.13-real(j)*.2
      enddo
    enddo;enddo
    if(exceptional)then
      c1(2)=0.;c2(2)=0.
      muts(1,1)=0.;mu_tend(1,1)=0.;c1(3)=1.;c2(3)=0.
      field_tend(1,3,1)=0.
    endif
    call spec_bdyupdate_ph(ph_save,field,field_tend,mu_tend,muts,c1,c2,.25, &
      variable,config,spec_zone,ids,ide,jds,jde,kds,domain_top, &
      ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,domain_top, &
      its,ite,jts,jte,kts,kte)
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
end program specified_boundary_geopotential_driver
