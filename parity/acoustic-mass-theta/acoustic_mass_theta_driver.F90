program acoustic_mass_theta_driver
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_acoustic_mass_theta, only: advance_mu_t
  implicit none
  call run_case('global',.false.,.false.,1,5,1,5)
  call run_case('nested',.true.,.false.,1,5,1,5)
  call run_case('nested_periodic',.true.,.true.,1,5,1,5)
  call run_case('partial',.false.,.false.,2,4,2,4)
contains
  subroutine run_case(name,nested,periodic_x,its,ite,jts,jte)
    character(len=*),intent(in)::name
    logical,intent(in)::nested,periodic_x
    integer,intent(in)::its,ite,jts,jte
    integer,parameter::ims=0,ime=5,jms=0,jme=5,kms=0,kme=5
    integer,parameter::ids=1,ide=5,jds=1,jde=5,kds=1,kde=5
    real::ww(ims:ime,kms:kme,jms:jme),ww1(ims:ime,kms:kme,jms:jme)
    real::u(ims:ime,kms:kme,jms:jme),u1(ims:ime,kms:kme,jms:jme)
    real::v(ims:ime,kms:kme,jms:jme),v1(ims:ime,kms:kme,jms:jme)
    real::t(ims:ime,kms:kme,jms:jme),t1(ims:ime,kms:kme,jms:jme)
    real::tave(ims:ime,kms:kme,jms:jme),ft(ims:ime,kms:kme,jms:jme)
    real::uam(ims:ime,kms:kme,jms:jme),vam(ims:ime,kms:kme,jms:jme)
    real::wwam(ims:ime,kms:kme,jms:jme)
    real::mu(ims:ime,jms:jme),mut(ims:ime,jms:jme),muave(ims:ime,jms:jme)
    real::muts(ims:ime,jms:jme),muu(ims:ime,jms:jme),muv(ims:ime,jms:jme)
    real::mudf(ims:ime,jms:jme),mutend(ims:ime,jms:jme)
    real::msfux(ims:ime,jms:jme),msfuy(ims:ime,jms:jme)
    real::msfvx(ims:ime,jms:jme),msfvxinv(ims:ime,jms:jme)
    real::msfvy(ims:ime,jms:jme),msftx(ims:ime,jms:jme),msfty(ims:ime,jms:jme)
    real::c1h(kms:kme),c2h(kms:kme),dnw(kms:kme),fnm(kms:kme),fnp(kms:kme),rdnw(kms:kme)
    real::unused(kms:kme)
    type(grid_config_rec_type)::config
    integer::i,j,k
    config%nested=nested;config%periodic_x=periodic_x
    do k=kms,kme
      c1h(k)=.45+real(k)*.01;c2h(k)=.2-real(k)*.005
      dnw(k)=.18+real(k)*.007;fnm(k)=.61+real(k)*.002
      fnp(k)=.39-real(k)*.002;rdnw(k)=1.1+real(k)*.03;unused(k)=7.
    enddo
    do j=jms,jme
      do i=ims,ime
        mu(i,j)=2.+real(i)*.11+real(j)*.17
        mut(i,j)=11.+real(i)*.13-real(j)*.09
        muu(i,j)=3.+real(i)*.07+real(j)*.02
        muv(i,j)=4.-real(i)*.03+real(j)*.08
        mutend(i,j)=.03+real(i)*.002-real(j)*.001
        msfux(i,j)=1.;msfvx(i,j)=1.;msfvy(i,j)=1.
        msfuy(i,j)=.92+real(i)*.006+real(j)*.003
        msfvxinv(i,j)=1.08-real(i)*.004+real(j)*.002
        msftx(i,j)=1.03+real(i)*.003-real(j)*.002
        msfty(i,j)=.97-real(i)*.002+real(j)*.004
      enddo
      do k=kms,kme
        do i=ims,ime
          u(i,k,j)=.2+real(i)*.013+real(k)*.017+real(j)*.019
          u1(i,k,j)=.15-real(i)*.006+real(k)*.011+real(j)*.004
          v(i,k,j)=.3-real(i)*.009+real(k)*.014+real(j)*.021
          v1(i,k,j)=.12+real(i)*.005-real(k)*.003+real(j)*.008
          ww(i,k,j)=.8+real(i)*.02+real(k)*.03-real(j)*.01
          ww1(i,k,j)=.35-real(i)*.004+real(k)*.006+real(j)*.003
          t(i,k,j)=300.+real(i)*.7+real(k)*1.1-real(j)*.4
          t1(i,k,j)=290.-real(i)*.3+real(k)*.9+real(j)*.5
          ft(i,k,j)=.012+real(i)*.0003-real(k)*.0002+real(j)*.0001
        enddo
      enddo
    enddo
    tave=-901.;muave=-902.;muts=-903.;mudf=-904.
    uam=-905.;vam=-906.;wwam=-907.
    call advance_mu_t(ww,ww1,u,u1,v,v1,mu,mut,muave,muts,muu,muv,mudf, &
      c1h,c2h,unused,unused,unused,unused,unused,unused,uam,vam,wwam,t,t1,tave,ft,mutend, &
      .002,.003,.4,.1,dnw,fnm,fnp,rdnw,msfux,msfuy,msfvx,msfvxinv,msfvy,msftx,msfty, &
      2,config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,its,ite,jts,jte,1,5)
    call write_volume(name,'ww',ww)
    call write_volume(name,'t',t)
    call write_volume(name,'tave',tave)
    call write_horizontal(name,'mu',mu)
    call write_horizontal(name,'muave',muave)
    call write_horizontal(name,'muts',muts)
    call write_horizontal(name,'mudf',mudf)
  end subroutine

  subroutine write_volume(case_name,field_name,field)
    character(len=*),intent(in)::case_name,field_name
    real,intent(in)::field(0:5,0:5,0:5)
    integer::i,j,k
    do j=0,5;do k=0,5;do i=0,5
      write(*,'(A,1X,A,3(1X,I0),1X,Z8.8)')case_name,field_name,i,k,j,transfer(field(i,k,j),0_int32)
    enddo;enddo;enddo
  end subroutine

  subroutine write_horizontal(case_name,field_name,field)
    character(len=*),intent(in)::case_name,field_name
    real,intent(in)::field(0:5,0:5)
    integer::i,j
    do j=0,5;do i=0,5
      write(*,'(A,1X,A,2(1X,I0),1X,Z8.8)')case_name,field_name,i,j,transfer(field(i,j),0_int32)
    enddo;enddo
  end subroutine
end program
