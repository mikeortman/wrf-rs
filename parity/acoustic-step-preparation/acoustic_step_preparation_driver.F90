program acoustic_step_preparation_driver
  use iso_fortran_env, only: int32
  use module_small_step_em, only: small_step_prep
  implicit none
  call run_case('first',1,5,1,5,1,.false.)
  call run_case('later',2,3,2,3,2,.false.)
  call run_case('exceptional',1,5,1,5,1,.true.)
contains
  subroutine run_case(name,its,ite,jts,jte,rk_step,exceptional)
    character(len=*), intent(in) :: name
    integer, intent(in) :: its,ite,jts,jte,rk_step
    logical, intent(in) :: exceptional
    integer, parameter :: ims=0,ime=5,jms=0,jme=5,kms=0,kme=4
    integer, parameter :: ids=1,ide=5,jds=1,jde=5,kds=1,kde=4,kts=1,kte=4
    real :: u1(ims:ime,kms:kme,jms:jme),u2(ims:ime,kms:kme,jms:jme)
    real :: v1(ims:ime,kms:kme,jms:jme),v2(ims:ime,kms:kme,jms:jme)
    real :: w1(ims:ime,kms:kme,jms:jme),w2(ims:ime,kms:kme,jms:jme)
    real :: t1(ims:ime,kms:kme,jms:jme),t2(ims:ime,kms:kme,jms:jme)
    real :: ph1(ims:ime,kms:kme,jms:jme),ph2(ims:ime,kms:kme,jms:jme)
    real :: us(ims:ime,kms:kme,jms:jme),vs(ims:ime,kms:kme,jms:jme)
    real :: ws(ims:ime,kms:kme,jms:jme),ts(ims:ime,kms:kme,jms:jme)
    real :: phs(ims:ime,kms:kme,jms:jme),wws(ims:ime,kms:kme,jms:jme)
    real :: c2a(ims:ime,kms:kme,jms:jme),pb(ims:ime,kms:kme,jms:jme)
    real :: p(ims:ime,kms:kme,jms:jme),alt(ims:ime,kms:kme,jms:jme)
    real :: ww(ims:ime,kms:kme,jms:jme)
    real :: mub(ims:ime,jms:jme),mu1(ims:ime,jms:jme),mu2(ims:ime,jms:jme)
    real :: muu(ims:ime,jms:jme),muus(ims:ime,jms:jme),muv(ims:ime,jms:jme)
    real :: muvs(ims:ime,jms:jme),mut(ims:ime,jms:jme),muts(ims:ime,jms:jme)
    real :: mudf(ims:ime,jms:jme),mus(ims:ime,jms:jme)
    real :: msfux(ims:ime,jms:jme),msfuy(ims:ime,jms:jme),msfvx(ims:ime,jms:jme)
    real :: msfvxi(ims:ime,jms:jme),msfvy(ims:ime,jms:jme),msftx(ims:ime,jms:jme),msfty(ims:ime,jms:jme)
    real :: c1h(kms:kme),c2h(kms:kme),c1f(kms:kme),c2f(kms:kme)
    real :: c3h(kms:kme),c4h(kms:kme),c3f(kms:kme),c4f(kms:kme)
    integer :: i,j,k

    do k=kms,kme
      c1h(k)=0.2+real(k)*0.03; c2h(k)=0.4-real(k)*0.02
      c1f(k)=0.3+real(k)*0.025; c2f(k)=0.5-real(k)*0.015
      c3h(k)=7.;c4h(k)=8.;c3f(k)=9.;c4f(k)=10.
    enddo
    do j=jms,jme
      do i=ims,ime
        mub(i,j)=40.+real(i)*1.3+real(j)*0.7
        mu1(i,j)=1.+real(i)*0.11-real(j)*0.04
        mu2(i,j)=-0.5+real(i)*0.07+real(j)*0.03
        muu(i,j)=42.+real(i)*1.1+real(j)*0.6
        muv(i,j)=43.+real(i)*0.9+real(j)*0.8
        mut(i,j)=44.+real(i)*1.2+real(j)*0.5
        msfuy(i,j)=1.+real(i)*0.02+real(j)*0.01
        msfvx(i,j)=0.9+real(i)*0.015-real(j)*0.005
        msfvxi(i,j)=1./msfvx(i,j)
        msfty(i,j)=1.1+real(i)*0.01+real(j)*0.02
        msfux(i,j)=7.;msfvy(i,j)=8.;msftx(i,j)=9.
      enddo
      do k=kms,kme
        do i=ims,ime
          u1(i,k,j)=1.+real(i)*.11+real(k)*.07-real(j)*.03
          u2(i,k,j)=.8+real(i)*.09-real(k)*.05+real(j)*.02
          v1(i,k,j)=2.-real(i)*.05+real(k)*.09+real(j)*.02
          v2(i,k,j)=1.6+real(i)*.04+real(k)*.03-real(j)*.01
          w1(i,k,j)=-1.+real(i)*.04-real(k)*.08+real(j)*.06
          w2(i,k,j)=-.7-real(i)*.02+real(k)*.06+real(j)*.03
          t1(i,k,j)=300.+real(i)*.2+real(k)*.6+real(j)*.1
          t2(i,k,j)=299.-real(i)*.1+real(k)*.4+real(j)*.2
          ph1(i,k,j)=1000.+real(i)*3.+real(k)*5.-real(j)*4.
          ph2(i,k,j)=900.-real(i)*2.+real(k)*4.+real(j)*3.
          pb(i,k,j)=80000.+real(i)*11.+real(k)*17.+real(j)*13.
          p(i,k,j)=500.+real(i)*3.-real(k)*2.+real(j)*4.
          alt(i,k,j)=.8+real(i)*.01+real(k)*.02+real(j)*.015
          ww(i,k,j)=.3+real(i)*.013-real(k)*.017+real(j)*.019
        enddo
      enddo
    enddo
    us=-999.;vs=-999.;ws=-999.;ts=-999.;phs=-999.;wws=-999.;c2a=-999.
    muus=-999.;muvs=-999.;muts=-999.;mudf=-999.;mus=-999.
    if(exceptional) then
      alt(1,1,1)=0.;msfuy(2,1)=-0.;msfvxi(1,2)=huge(msfvxi)*2.
      msfty(2,2)=0.;p(3,1,1)=huge(p);pb(3,1,1)=huge(pb)
    endif
    call small_step_prep(u1,u2,v1,v2,w1,w2,t1,t2,ph1,ph2, &
      mub,mu1,mu2,muu,muus,muv,muvs,mut,muts,mudf, &
      c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f,us,vs,ws,ts,phs,mus, &
      ww,wws,c2a,pb,p,alt,msfux,msfuy,msfvx,msfvxi,msfvy,msftx,msfty, &
      1.,1.,rk_step,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kts,kte)
    call write_volume(name,'u1',u1);call write_volume(name,'u2',u2)
    call write_volume(name,'v1',v1);call write_volume(name,'v2',v2)
    call write_volume(name,'w1',w1);call write_volume(name,'w2',w2)
    call write_volume(name,'t1',t1);call write_volume(name,'t2',t2)
    call write_volume(name,'ph1',ph1);call write_volume(name,'ph2',ph2)
    call write_volume(name,'us',us);call write_volume(name,'vs',vs)
    call write_volume(name,'ws',ws);call write_volume(name,'ts',ts)
    call write_volume(name,'phs',phs);call write_volume(name,'wws',wws)
    call write_volume(name,'c2a',c2a)
    call write_horizontal(name,'mu1',mu1);call write_horizontal(name,'mu2',mu2)
    call write_horizontal(name,'muus',muus);call write_horizontal(name,'muvs',muvs)
    call write_horizontal(name,'muts',muts);call write_horizontal(name,'mudf',mudf)
    call write_horizontal(name,'mus',mus)
  end subroutine
  subroutine write_volume(case_name,field_name,field)
    character(len=*),intent(in)::case_name,field_name
    real,intent(in)::field(0:5,0:4,0:5)
    integer::i,j,k
    do j=0,5;do k=0,4;do i=0,5
      if(isnan(field(i,k,j)))then
        write(*,'(A,1X,A,3(1X,I0),1X,A)')case_name,field_name,i,k,j,'NAN'
      else
        write(*,'(A,1X,A,3(1X,I0),1X,Z8.8)')case_name,field_name,i,k,j,transfer(field(i,k,j),0_int32)
      endif
    enddo;enddo;enddo
  end subroutine
  subroutine write_horizontal(case_name,field_name,field)
    character(len=*),intent(in)::case_name,field_name
    real,intent(in)::field(0:5,0:5)
    integer::i,j
    do j=0,5;do i=0,5
      if(isnan(field(i,j)))then
        write(*,'(A,1X,A,2(1X,I0),1X,A)')case_name,field_name,i,j,'NAN'
      else
        write(*,'(A,1X,A,2(1X,I0),1X,Z8.8)')case_name,field_name,i,j,transfer(field(i,j),0_int32)
      endif
    enddo;enddo
  end subroutine
end program
