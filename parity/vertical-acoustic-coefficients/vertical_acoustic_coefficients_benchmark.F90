program vertical_acoustic_coefficients_benchmark
  use iso_fortran_env, only: int64,real64
  use extracted_vertical_acoustic_coefficients, only: calc_coef_w
  implicit none
  integer,parameter::nx=256,ny=256,nz=40,samples=11,iterations=20
  integer,parameter::ims=1,ime=nx,jms=1,jme=ny,kms=1,kme=nz+1
  integer,parameter::ids=1,ide=nx+1,jds=1,jde=ny+1,kds=1,kde=nz+1
  integer,parameter::its=1,ite=nx,jts=1,jte=ny,kts=1,kte=nz
  real,allocatable::a(:,:,:),alpha(:,:,:),gamma(:,:,:)
  real,allocatable::mut(:,:),cqw(:,:,:),c2a(:,:,:)
  real,allocatable::c1h(:),c2h(:),c1f(:),c2f(:),c3h(:),c4h(:),c3f(:),c4f(:)
  real,allocatable::rdn(:),rdnw(:)
  integer(int64)::start_count,end_count,clock_rate
  integer::sample,iteration
  real(real64)::milliseconds,checksum

  allocate(a(ims:ime,kms:kme,jms:jme),alpha(ims:ime,kms:kme,jms:jme))
  allocate(gamma(ims:ime,kms:kme,jms:jme),cqw(ims:ime,kms:kme,jms:jme))
  allocate(c2a(ims:ime,kms:kme,jms:jme),mut(ims:ime,jms:jme))
  allocate(c1h(kms:kme),c2h(kms:kme),c1f(kms:kme),c2f(kms:kme))
  allocate(c3h(kms:kme),c4h(kms:kme),c3f(kms:kme),c4f(kms:kme))
  allocate(rdn(kms:kme),rdnw(kms:kme))
  a=-901.;alpha=-902.;gamma=-903.;mut=40.;cqw=.9;c2a=140000.
  c1h=.2;c2h=.4;c1f=.25;c2f=.35;c3h=7.;c4h=8.;c3f=9.;c4f=10.
  rdn=1.1;rdnw=1.3
  call system_clock(count_rate=clock_rate)
  do iteration=1,10
    call run_once()
  enddo
  do sample=1,samples
    call system_clock(start_count)
    do iteration=1,iterations
      call run_once()
    enddo
    call system_clock(end_count)
    milliseconds=1000._real64*real(end_count-start_count,real64)/real(clock_rate,real64)/iterations
    write(*,'(A,1X,F0.6)')'nonrigid',milliseconds
  enddo
  checksum=sum(real(a,real64))+sum(real(alpha,real64))+sum(real(gamma,real64))
  write(*,'(A,1X,ES24.16)')'checksum',checksum
contains
  subroutine run_once()
    call calc_coef_w(a,alpha,gamma,mut,c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f, &
      cqw,rdn,rdnw,c2a,2.5,9.81,.1,.false., &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      its,ite,jts,jte,kts,kte)
  end subroutine
end program
