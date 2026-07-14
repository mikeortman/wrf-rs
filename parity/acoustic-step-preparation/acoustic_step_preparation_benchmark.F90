program acoustic_step_preparation_benchmark
  use iso_fortran_env, only: int64, real64
  use module_small_step_em, only: small_step_prep
  implicit none
  integer, parameter :: nx=256,ny=256,nz=40,samples=11,iterations=10
  integer, parameter :: ims=0,ime=nx+1,jms=0,jme=ny+1,kms=0,kme=nz+1
  integer, parameter :: ids=1,ide=nx+1,jds=1,jde=ny+1,kds=1,kde=nz+1
  integer, parameter :: its=1,ite=nx+1,jts=1,jte=ny+1,kts=1,kte=nz+1
  real, allocatable :: u1(:,:,:),u2(:,:,:),v1(:,:,:),v2(:,:,:)
  real, allocatable :: w1(:,:,:),w2(:,:,:),t1(:,:,:),t2(:,:,:)
  real, allocatable :: ph1(:,:,:),ph2(:,:,:),us(:,:,:),vs(:,:,:)
  real, allocatable :: ws(:,:,:),ts(:,:,:),phs(:,:,:),wws(:,:,:)
  real, allocatable :: c2a(:,:,:),pb(:,:,:),p(:,:,:),alt(:,:,:),ww(:,:,:)
  real, allocatable :: mub(:,:),mu1(:,:),mu2(:,:),muu(:,:),muus(:,:)
  real, allocatable :: muv(:,:),muvs(:,:),mut(:,:),muts(:,:),mudf(:,:),mus(:,:)
  real, allocatable :: msfux(:,:),msfuy(:,:),msfvx(:,:),msfvxi(:,:)
  real, allocatable :: msfvy(:,:),msftx(:,:),msfty(:,:)
  real, allocatable :: c1h(:),c2h(:),c1f(:),c2f(:),c3h(:),c4h(:),c3f(:),c4f(:)
  integer(int64) :: start_count,end_count,clock_rate
  integer :: sample,iteration
  real(real64) :: milliseconds,checksum

  allocate(u1(ims:ime,kms:kme,jms:jme),u2(ims:ime,kms:kme,jms:jme))
  allocate(v1(ims:ime,kms:kme,jms:jme),v2(ims:ime,kms:kme,jms:jme))
  allocate(w1(ims:ime,kms:kme,jms:jme),w2(ims:ime,kms:kme,jms:jme))
  allocate(t1(ims:ime,kms:kme,jms:jme),t2(ims:ime,kms:kme,jms:jme))
  allocate(ph1(ims:ime,kms:kme,jms:jme),ph2(ims:ime,kms:kme,jms:jme))
  allocate(us(ims:ime,kms:kme,jms:jme),vs(ims:ime,kms:kme,jms:jme))
  allocate(ws(ims:ime,kms:kme,jms:jme),ts(ims:ime,kms:kme,jms:jme))
  allocate(phs(ims:ime,kms:kme,jms:jme),wws(ims:ime,kms:kme,jms:jme))
  allocate(c2a(ims:ime,kms:kme,jms:jme),pb(ims:ime,kms:kme,jms:jme))
  allocate(p(ims:ime,kms:kme,jms:jme),alt(ims:ime,kms:kme,jms:jme),ww(ims:ime,kms:kme,jms:jme))
  allocate(mub(ims:ime,jms:jme),mu1(ims:ime,jms:jme),mu2(ims:ime,jms:jme))
  allocate(muu(ims:ime,jms:jme),muus(ims:ime,jms:jme),muv(ims:ime,jms:jme))
  allocate(muvs(ims:ime,jms:jme),mut(ims:ime,jms:jme),muts(ims:ime,jms:jme))
  allocate(mudf(ims:ime,jms:jme),mus(ims:ime,jms:jme))
  allocate(msfux(ims:ime,jms:jme),msfuy(ims:ime,jms:jme),msfvx(ims:ime,jms:jme))
  allocate(msfvxi(ims:ime,jms:jme),msfvy(ims:ime,jms:jme))
  allocate(msftx(ims:ime,jms:jme),msfty(ims:ime,jms:jme))
  allocate(c1h(kms:kme),c2h(kms:kme),c1f(kms:kme),c2f(kms:kme))
  allocate(c3h(kms:kme),c4h(kms:kme),c3f(kms:kme),c4f(kms:kme))

  u1=1.;u2=.8;v1=2.;v2=1.6;w1=-1.;w2=-.7;t1=300.;t2=299.
  ph1=1000.;ph2=900.;us=-999.;vs=-999.;ws=-999.;ts=-999.;phs=-999.
  wws=-999.;c2a=-999.;pb=80000.;p=500.;alt=.8;ww=.3
  mub=40.;mu1=1.;mu2=-.5;muu=42.;muus=-999.;muv=43.;muvs=-999.
  mut=44.;muts=-999.;mudf=-999.;mus=-999.
  msfux=7.;msfuy=1.03;msfvx=.97;msfvxi=1./msfvx;msfvy=8.;msftx=9.;msfty=1.12
  c1h=.2;c2h=.4;c1f=.3;c2f=.5;c3h=7.;c4h=8.;c3f=9.;c4f=10.

  do iteration=1,5
    call run_once()
  enddo
  call system_clock(count_rate=clock_rate)
  do sample=1,samples
    call system_clock(start_count)
    do iteration=1,iterations
      call run_once()
    enddo
    call system_clock(end_count)
    milliseconds=1000._real64*real(end_count-start_count,real64)/real(clock_rate,real64)/iterations
    write(*,'(F0.6)') milliseconds
  enddo
  checksum=sum(real(u1,real64))+sum(real(u2,real64))+sum(real(v1,real64))+sum(real(v2,real64)) &
    +sum(real(w1,real64))+sum(real(w2,real64))+sum(real(t1,real64))+sum(real(t2,real64)) &
    +sum(real(ph1,real64))+sum(real(ph2,real64))+sum(real(us,real64))+sum(real(vs,real64)) &
    +sum(real(ws,real64))+sum(real(ts,real64))+sum(real(phs,real64))+sum(real(wws,real64)) &
    +sum(real(c2a,real64))+sum(real(mu1,real64))+sum(real(mu2,real64))+sum(real(muus,real64)) &
    +sum(real(muvs,real64))+sum(real(muts,real64))+sum(real(mudf,real64))+sum(real(mus,real64))
  write(*,'(A,ES24.16)') 'checksum ',checksum
contains
  subroutine run_once()
    call small_step_prep(u1,u2,v1,v2,w1,w2,t1,t2,ph1,ph2, &
      mub,mu1,mu2,muu,muus,muv,muvs,mut,muts,mudf, &
      c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f,us,vs,ws,ts,phs,mus, &
      ww,wws,c2a,pb,p,alt,msfux,msfuy,msfvx,msfvxi,msfvy,msftx,msfty, &
      1.,1.,1,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kts,kte)
  end subroutine
end program
