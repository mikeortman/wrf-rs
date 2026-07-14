program dry_tendency_assembly_benchmark
  use iso_fortran_env, only: int64, real64
  implicit none
  integer, parameter :: nx=256, ny=256, nz=40
  integer, parameter :: ims=0, ime=nx+1, jms=0, jme=ny+1, kms=0, kme=nz+1
  integer, parameter :: ids=1, ide=nx+1, jds=1, jde=ny+1, kds=1, kde=nz+1
  integer, parameter :: its=ids, ite=ide, jts=jds, jte=jde, kts=kds, kte=kde
  integer, parameter :: samples=31, calls_per_sample=20, warmup_calls=10
  real, allocatable :: ru(:,:,:),rv(:,:,:),rw(:,:,:),ph(:,:,:),t(:,:,:)
  real, allocatable :: ruf(:,:,:),rvf(:,:,:),rwf(:,:,:),phf(:,:,:),tf(:,:,:)
  real, allocatable :: us(:,:,:),vs(:,:,:),ws(:,:,:),phs(:,:,:),ts(:,:,:),heat(:,:,:)
  real, allocatable :: mu(:,:),muf(:,:),mut(:,:),msftx(:,:),msfty(:,:),msfux(:,:)
  real, allocatable :: msfuy(:,:),msfvx(:,:),msfvxi(:,:),msfvy(:,:),c1(:),c2(:)
  integer(int64) :: start_count,end_count,clock_rate
  integer :: call_index,sample
  real(real64) :: milliseconds,checksum

  allocate(ru(ims:ime,kms:kme,jms:jme),rv(ims:ime,kms:kme,jms:jme),rw(ims:ime,kms:kme,jms:jme))
  allocate(ph(ims:ime,kms:kme,jms:jme),t(ims:ime,kms:kme,jms:jme))
  allocate(ruf(ims:ime,kms:kme,jms:jme),rvf(ims:ime,kms:kme,jms:jme),rwf(ims:ime,kms:kme,jms:jme))
  allocate(phf(ims:ime,kms:kme,jms:jme),tf(ims:ime,kms:kme,jms:jme))
  allocate(us(ims:ime,kms:kme,jms:jme),vs(ims:ime,kms:kme,jms:jme),ws(ims:ime,kms:kme,jms:jme))
  allocate(phs(ims:ime,kms:kme,jms:jme),ts(ims:ime,kms:kme,jms:jme),heat(ims:ime,kms:kme,jms:jme))
  allocate(mu(ims:ime,jms:jme),muf(ims:ime,jms:jme),mut(ims:ime,jms:jme))
  allocate(msftx(ims:ime,jms:jme),msfty(ims:ime,jms:jme),msfux(ims:ime,jms:jme))
  allocate(msfuy(ims:ime,jms:jme),msfvx(ims:ime,jms:jme),msfvxi(ims:ime,jms:jme),msfvy(ims:ime,jms:jme))
  allocate(c1(kms:kme),c2(kms:kme))
  ru=1.;rv=2.;rw=-1.;ph=3.;t=-2.;ruf=.3;rvf=-.4;rwf=.5;phf=-.6;tf=.7
  us=.09;vs=-.08;ws=.07;phs=-.06;ts=.05;heat=.001
  mu=.6;muf=-.2;mut=50.;msftx=9.;msfty=1.12;msfux=8.;msfuy=1.03
  msfvx=.97;msfvxi=1./.97;msfvy=7.;c1=.2;c2=.4

  do call_index=1,warmup_calls
    call apply_kernel()
  end do
  call system_clock(count_rate=clock_rate)
  do sample=1,samples
    call system_clock(start_count)
    do call_index=1,calls_per_sample
      call apply_kernel()
    end do
    call system_clock(end_count)
    milliseconds=real(end_count-start_count,real64)*1000._real64/real(clock_rate,real64)/real(calls_per_sample,real64)
    write(*,'(A,I0,A,F12.6)') 'sample_',sample,'_milliseconds_per_call ',milliseconds
  end do
  checksum=sum(real(ru(ids:ide,kds:kde-1,jds:jde-1),real64))+sum(real(mu(ids:ide-1,jds:jde-1),real64))
  write(*,'(A,ES24.16)') 'checksum ',checksum

contains
  subroutine apply_kernel()
    call rk_addtend_dry(ru,rv,rw,ph,t,ruf,rvf,rwf,phf,tf,us,vs,ws,phs,ts, &
      mu,muf,1,c1,c2,heat,mut,msftx,msfty,msfux,msfuy,msfvx,msfvxi,msfvy, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kts,kte)
  end subroutine apply_kernel
end program dry_tendency_assembly_benchmark
