program acoustic_horizontal_momentum_driver
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_acoustic_horizontal_momentum, only: advance_uv
  implicit none
  integer,parameter::ims=0,ime=5,jms=0,jme=5,kms=0,kme=5
  integer,parameter::ids=1,ide=5,jds=1,jde=5,kds=1,kde=5
  real::u(ims:ime,kms:kme,jms:jme),v(ims:ime,kms:kme,jms:jme)
  real::ru(ims:ime,kms:kme,jms:jme),rv(ims:ime,kms:kme,jms:jme)
  real::p(ims:ime,kms:kme,jms:jme),pb(ims:ime,kms:kme,jms:jme)
  real::ph(ims:ime,kms:kme,jms:jme),php(ims:ime,kms:kme,jms:jme)
  real::alt(ims:ime,kms:kme,jms:jme),al(ims:ime,kms:kme,jms:jme)
  real::cqu(ims:ime,kms:kme,jms:jme),cqv(ims:ime,kms:kme,jms:jme)
  real::mu(ims:ime,jms:jme),muu(ims:ime,jms:jme),muv(ims:ime,jms:jme),mudf(ims:ime,jms:jme)
  real::msfux(ims:ime,jms:jme),msfuy(ims:ime,jms:jme),msfvx(ims:ime,jms:jme)
  real::msfvx_inv(ims:ime,jms:jme),msfvy(ims:ime,jms:jme)
  real::c1h(kms:kme),c2h(kms:kme),fnm(kms:kme),fnp(kms:kme),rdnw(kms:kme)
  real::unused(kms:kme)
  type(grid_config_rec_type)::config
  integer::i,j,k
  u=1.;v=1.;ru=2.;rv=2.;pb=4.;ph=5.;alt=.8;al=.1;cqu=1.;cqv=1.
  mu=1.;muu=2.;muv=2.;msfux=1.;msfuy=1.;msfvx=1.;msfvx_inv=1.;msfvy=1.
  c1h=.5;c2h=.25;fnm=.6;fnp=.4;rdnw=1.2;unused=0.
  do j=jms,jme
    do i=ims,ime
      mudf(i,j)=real(i)+2.*real(j)
    enddo
    do k=kms,kme
      do i=ims,ime
        p(i,k,j)=1.+real(i)*.2+real(j)*.3+real(k)*.4
        php(i,k,j)=2.+real(i)*.15+real(j)*.25
      enddo
    enddo
  enddo
  call advance_uv(u,ru,v,rv,p,pb,ph,php,alt,al,mu,muu,cqu,muv,cqv,mudf, &
    c1h,c2h,unused,unused,unused,unused,unused,unused, &
    msfux,msfuy,msfvx,msfvx_inv,msfvy,2.,4.,.5,.7,.2,.1,fnm,fnp,.1,rdnw, &
    config,0,.true.,.true.,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
    1,5,1,5,1,5)
  call write_volume('u',u)
  call write_volume('v',v)
contains
  subroutine write_volume(name,field)
    character(len=*),intent(in)::name
    real,intent(in)::field(0:5,0:5,0:5)
    integer::i,j,k
    do j=0,5;do k=0,5;do i=0,5
      write(*,'(A,3(1X,I0),1X,Z8.8)')name,i,k,j,transfer(field(i,k,j),0_int32)
    enddo;enddo;enddo
  end subroutine
end program
