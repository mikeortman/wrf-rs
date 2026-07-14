program specified_boundary_finalization_driver
  use iso_fortran_env, only: int32
  use ieee_arithmetic, only: ieee_negative_inf, ieee_positive_inf, ieee_value
  use module_configure, only: grid_config_rec_type
  use extracted_specified_boundary_finalization, only: spec_bdy_final
  implicit none

  call run_volume('scalar_full','t',.false.,1,6,1,6,1,6,.false.)
  call run_volume('west_east_momentum','u',.false.,1,7,1,6,1,6,.false.)
  call run_volume('south_north_momentum','v',.false.,1,6,1,7,1,6,.false.)
  call run_volume('vertical_momentum','w',.false.,1,6,1,6,1,7,.false.)
  call run_volume('full_level','h',.false.,1,6,1,6,1,7,.false.)
  call run_horizontal_mass
  call run_volume('scalar_periodic','t',.true.,1,6,1,6,1,6,.false.)
  call run_volume('scalar_partial_south_west','t',.false.,1,4,1,4,2,4,.false.)
  call run_volume('full_partial_north_east','h',.false.,3,6,3,6,2,4,.false.)
  call run_volume('scalar_interior','t',.false.,3,4,3,4,2,4,.false.)
  call run_volume('momentum_exceptional','w',.false.,1,6,1,6,1,7,.true.)
contains
  subroutine run_volume(name,variable,periodic_x,its,ite,jts,jte,kts,kte,exceptional)
    character(len=*),intent(in)::name
    character,intent(in)::variable
    logical,intent(in)::periodic_x,exceptional
    integer,intent(in)::its,ite,jts,jte,kts,kte
    real::field(0:7,0:7,0:7),mu(0:7,0:7),msf(0:7,0:7)
    real::c1(0:7),c2(0:7)
    real::west(0:7,1:7,3),east(0:7,1:7,3)
    real::south(0:7,1:7,3),north(0:7,1:7,3)
    real::west_tendency(0:7,1:7,3),east_tendency(0:7,1:7,3)
    real::south_tendency(0:7,1:7,3),north_tendency(0:7,1:7,3)
    type(grid_config_rec_type)::config

    call initialize_volume(field,mu,msf,c1,c2)
    call initialize_boundaries(west,east,south,north,west_tendency,east_tendency, &
      south_tendency,north_tendency)
    if (exceptional) then
      west(3,3,2)=ieee_value(0.,ieee_positive_inf)
      east(4,2,1)=ieee_value(0.,ieee_negative_inf)
      south(3,4,1)=-0.
      south_tendency(3,4,1)=-0.
      msf(3,6)=-0.
    endif
    config%periodic_x=periodic_x
    call spec_bdy_final(field,mu,c1,c2,msf,west,east,south,north, &
      west_tendency,east_tendency,south_tendency,north_tendency, &
      variable,config,3,2,.25, &
      1,7,1,7,1,7,0,7,0,7,0,7,1,7,1,7,1,7, &
      its,ite,jts,jte,kts,kte)
    call emit_volume(name,field)
  end subroutine

  subroutine run_horizontal_mass
    real::field(0:7,0:0,0:7),mu(0:7,0:7),msf(0:7,0:7)
    real::c1(0:0),c2(0:0)
    real::west(0:7,0:0,3),east(0:7,0:0,3)
    real::south(0:7,0:0,3),north(0:7,0:0,3)
    real::west_tendency(0:7,0:0,3),east_tendency(0:7,0:0,3)
    real::south_tendency(0:7,0:0,3),north_tendency(0:7,0:0,3)
    type(grid_config_rec_type)::config
    integer::i,j,b

    do j=0,7;do i=0,7
      field(i,0,j)=-30.+real(i)*.7-real(j)*.2
      mu(i,j)=5.+real(i)*.03+real(j)*.02
      msf(i,j)=.9+real(i)*.004-real(j)*.003
    enddo;enddo
    c1(0)=.4;c2(0)=1.3
    do b=1,3;do i=0,7
      west(i,0,b)=10.+real(i)*.5+.009*real(b)
      east(i,0,b)=-8.+real(i)*.4+.008*real(b)
      south(i,0,b)=4.-real(i)*.3-.007*real(b)
      north(i,0,b)=-2.+real(i)*.2+.006*real(b)
      west_tendency(i,0,b)=.03-real(i)*.002+.0009*real(b)
      east_tendency(i,0,b)=-.02+real(i)*.001+.0008*real(b)
      south_tendency(i,0,b)=.01+real(i)*.0015-.0007*real(b)
      north_tendency(i,0,b)=-.015-real(i)*.001+.0006*real(b)
    enddo;enddo
    config%periodic_x=.false.
    call spec_bdy_final(field,mu,c1,c2,msf,west,east,south,north, &
      west_tendency,east_tendency,south_tendency,north_tendency, &
      'm',config,3,2,.25, &
      1,7,1,7,0,0,0,7,0,7,0,0,1,7,1,7,0,0, &
      1,6,1,6,0,0)
    call emit_horizontal_mass('horizontal_mass',field)
  end subroutine

  subroutine initialize_volume(field,mu,msf,c1,c2)
    real,intent(out)::field(0:7,0:7,0:7),mu(0:7,0:7),msf(0:7,0:7)
    real,intent(out)::c1(0:7),c2(0:7)
    integer::i,j,k
    do j=0,7;do k=0,7;do i=0,7
      field(i,k,j)=-30.+real(i)*.7+real(k)*.11-real(j)*.2
    enddo;enddo;enddo
    do j=0,7;do i=0,7
      mu(i,j)=5.+real(i)*.03+real(j)*.02
      msf(i,j)=.9+real(i)*.004-real(j)*.003
    enddo;enddo
    do k=0,7
      c1(k)=.4+real(k)*.01
      c2(k)=1.3-real(k)*.015
    enddo
  end subroutine

  subroutine initialize_boundaries(west,east,south,north,west_tendency, &
      east_tendency,south_tendency,north_tendency)
    real,intent(out)::west(0:7,1:7,3),east(0:7,1:7,3)
    real,intent(out)::south(0:7,1:7,3),north(0:7,1:7,3)
    real,intent(out)::west_tendency(0:7,1:7,3),east_tendency(0:7,1:7,3)
    real,intent(out)::south_tendency(0:7,1:7,3),north_tendency(0:7,1:7,3)
    integer::line,k,b
    do b=1,3;do k=1,7;do line=0,7
      west(line,k,b)=10.+real(line)*.5+real(k)*.07+.009*real(b)
      east(line,k,b)=-8.+real(line)*.4-real(k)*.05+.008*real(b)
      south(line,k,b)=4.-real(line)*.3+real(k)*.06-.007*real(b)
      north(line,k,b)=-2.+real(line)*.2+real(k)*.04+.006*real(b)
      west_tendency(line,k,b)=.03-real(line)*.002+real(k)*.0004+.0009*real(b)
      east_tendency(line,k,b)=-.02+real(line)*.001-real(k)*.0003+.0008*real(b)
      south_tendency(line,k,b)=.01+real(line)*.0015+real(k)*.0002-.0007*real(b)
      north_tendency(line,k,b)=-.015-real(line)*.001+real(k)*.0005+.0006*real(b)
    enddo;enddo;enddo
  end subroutine

  subroutine emit_volume(name,field)
    character(len=*),intent(in)::name
    real,intent(in)::field(0:7,0:7,0:7)
    integer::i,j,k
    do j=0,7;do k=0,7;do i=0,7
      write(*,'(A,3(1X,I0),1X,Z8.8)')name,i,k,j,transfer(field(i,k,j),0_int32)
    enddo;enddo;enddo
  end subroutine

  subroutine emit_horizontal_mass(name,field)
    character(len=*),intent(in)::name
    real,intent(in)::field(0:7,0:0,0:7)
    integer::i,j
    do j=0,7;do i=0,7
      write(*,'(A,3(1X,I0),1X,Z8.8)')name,i,0,j,transfer(field(i,0,j),0_int32)
    enddo;enddo
  end subroutine
end program specified_boundary_finalization_driver
