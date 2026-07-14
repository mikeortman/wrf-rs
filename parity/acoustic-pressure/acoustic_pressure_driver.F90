program acoustic_pressure_driver
  use iso_fortran_env, only: int32
  use extracted_acoustic_pressure, only: calc_p_rho
  implicit none
  call run_case('nonhydro_init',.true.,0,1,5,1,5,1,5,.false.)
  call run_case('nonhydro_advance',.true.,1,2,3,2,3,2,3,.true.)
  call run_case('hydro_init',.false.,0,1,5,1,5,1,5,.false.)
  call run_case('hydro_advance',.false.,1,2,3,2,3,2,3,.true.)
contains
  subroutine run_case(name,non_hydrostatic,step,its,ite,jts,jte,kts,kte,exceptional)
    character(len=*),intent(in)::name
    logical,intent(in)::non_hydrostatic,exceptional
    integer,intent(in)::step,its,ite,jts,jte,kts,kte
    integer,parameter::ims=0,ime=5,jms=0,jme=5,kms=0,kme=5
    integer,parameter::ids=1,ide=5,jds=1,jde=5,kds=1,kde=5
    real::al(ims:ime,kms:kme,jms:jme),p(ims:ime,kms:kme,jms:jme)
    real::ph(ims:ime,kms:kme,jms:jme),alt(ims:ime,kms:kme,jms:jme)
    real::t2(ims:ime,kms:kme,jms:jme),t1(ims:ime,kms:kme,jms:jme)
    real::c2a(ims:ime,kms:kme,jms:jme),pm1(ims:ime,kms:kme,jms:jme)
    real::mu(ims:ime,jms:jme),mut(ims:ime,jms:jme)
    real::c1h(kms:kme),c2h(kms:kme),c1f(kms:kme),c2f(kms:kme)
    real::c3h(kms:kme),c4h(kms:kme),c3f(kms:kme),c4f(kms:kme)
    real::znu(kms:kme),rdnw(kms:kme),dnw(kms:kme)
    real,parameter::t0=300.,smdiv=.17
    integer::i,j,k
    do k=kms,kme
      c1h(k)=.2+real(k)*.03;c2h(k)=.4-real(k)*.02
      c3h(k)=1.1+real(k)*.04
      rdnw(k)=1.3+real(k)*.05;dnw(k)=.7-real(k)*.025
      c1f(k)=7.;c2f(k)=8.;c3f(k)=9.;c4h(k)=10.;c4f(k)=11.;znu(k)=12.
    enddo
    do j=jms,jme
      do i=ims,ime
        mu(i,j)=1.+real(i)*.11-real(j)*.04
        mut(i,j)=40.+real(i)*1.3+real(j)*.7
      enddo
      do k=kms,kme
        do i=ims,ime
          ph(i,k,j)=900.+real(i)*3.+real(k)*5.-real(j)*4.
          alt(i,k,j)=.8+real(i)*.01+real(k)*.02+real(j)*.015
          t2(i,k,j)=2.+real(i)*.09-real(k)*.05+real(j)*.02
          t1(i,k,j)=1.+real(i)*.07+real(k)*.03-real(j)*.01
          c2a(i,k,j)=140000.+real(i)*13.+real(k)*17.+real(j)*11.
          pm1(i,k,j)=500.+real(i)*3.-real(k)*2.+real(j)*4.
        enddo
      enddo
    enddo
    al=-999.;p=-999.
    if(exceptional)then
      c1h(2)=0.;c2h(2)=0.
      t1(3,2,2)=-t0
      c2a(2,2,2)=0.
      ph(2,3,2)=huge(ph)*2.
      pm1(3,3,3)=-huge(pm1)
    endif
    call calc_p_rho(al,p,ph,alt,t2,t1,c2a,pm1,mu,mut, &
      c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f,znu,t0,rdnw,dnw,smdiv, &
      non_hydrostatic,step,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      its,ite,jts,jte,kts,kte)
    call write_volume(name,'al',al)
    call write_volume(name,'p',p)
    call write_volume(name,'ph',ph)
    call write_volume(name,'pm1',pm1)
  end subroutine

  subroutine write_volume(case_name,field_name,field)
    character(len=*),intent(in)::case_name,field_name
    real,intent(in)::field(0:5,0:5,0:5)
    integer::i,j,k
    do j=0,5;do k=0,5;do i=0,5
      if(isnan(field(i,k,j)))then
        write(*,'(A,1X,A,3(1X,I0),1X,A)')case_name,field_name,i,k,j,'NAN'
      else
        write(*,'(A,1X,A,3(1X,I0),1X,Z8.8)')case_name,field_name,i,k,j,transfer(field(i,k,j),0_int32)
      endif
    enddo;enddo;enddo
  end subroutine
end program
