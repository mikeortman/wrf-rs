program acoustic_pressure_benchmark
  use iso_fortran_env, only: int64,real64
  use extracted_acoustic_pressure, only: calc_p_rho
  implicit none
  integer,parameter::nx=256,ny=256,nz=40,samples=11,iterations=20
  integer,parameter::ims=0,ime=nx+1,jms=0,jme=ny+1,kms=0,kme=nz+1
  integer,parameter::ids=1,ide=nx+1,jds=1,jde=ny+1,kds=1,kde=nz+1
  integer,parameter::its=1,ite=nx+1,jts=1,jte=ny+1,kts=1,kte=nz+1
  real,allocatable::al(:,:,:),p(:,:,:),ph(:,:,:),alt(:,:,:),t2(:,:,:),t1(:,:,:),c2a(:,:,:),pm1(:,:,:)
  real,allocatable::mu(:,:),mut(:,:),c1h(:),c2h(:),c1f(:),c2f(:),c3h(:),c4h(:),c3f(:),c4f(:)
  real,allocatable::znu(:),rdnw(:),dnw(:)
  integer(int64)::start_count,end_count,clock_rate
  integer::sample,iteration
  real(real64)::milliseconds,checksum

  allocate(al(ims:ime,kms:kme,jms:jme),p(ims:ime,kms:kme,jms:jme))
  allocate(ph(ims:ime,kms:kme,jms:jme),alt(ims:ime,kms:kme,jms:jme))
  allocate(t2(ims:ime,kms:kme,jms:jme),t1(ims:ime,kms:kme,jms:jme))
  allocate(c2a(ims:ime,kms:kme,jms:jme),pm1(ims:ime,kms:kme,jms:jme))
  allocate(mu(ims:ime,jms:jme),mut(ims:ime,jms:jme))
  allocate(c1h(kms:kme),c2h(kms:kme),c1f(kms:kme),c2f(kms:kme))
  allocate(c3h(kms:kme),c4h(kms:kme),c3f(kms:kme),c4f(kms:kme))
  allocate(znu(kms:kme),rdnw(kms:kme),dnw(kms:kme))
  call system_clock(count_rate=clock_rate)
  call benchmark_mode(.true.,'nonhydrostatic')
  call benchmark_mode(.false.,'hydrostatic')
contains
  subroutine initialize_fields()
    al=-999.;p=-999.;ph=900.;alt=.8;t2=2.;t1=1.;c2a=140000.;pm1=500.
    mu=1.;mut=40.;c1h=.2;c2h=.4;c3h=1.1;rdnw=1.3;dnw=.7
    c1f=7.;c2f=8.;c4h=10.;c3f=9.;c4f=11.;znu=12.
  end subroutine

  subroutine benchmark_mode(non_hydrostatic,name)
    logical,intent(in)::non_hydrostatic
    character(len=*),intent(in)::name
    call initialize_fields()
    do iteration=1,5
      call run_once(non_hydrostatic)
    enddo
    do sample=1,samples
      call system_clock(start_count)
      do iteration=1,iterations
        call run_once(non_hydrostatic)
      enddo
      call system_clock(end_count)
      milliseconds=1000._real64*real(end_count-start_count,real64)/real(clock_rate,real64)/iterations
      write(*,'(A,1X,F0.6)')name,milliseconds
    enddo
    checksum=sum(real(al,real64))+sum(real(p,real64))+sum(real(ph,real64))+sum(real(pm1,real64))
    write(*,'(A,1X,A,1X,ES24.16)')'checksum',name,checksum
  end subroutine

  subroutine run_once(non_hydrostatic)
    logical,intent(in)::non_hydrostatic
    call calc_p_rho(al,p,ph,alt,t2,t1,c2a,pm1,mu,mut, &
      c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f,znu,300.,rdnw,dnw,.17, &
      non_hydrostatic,0,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      its,ite,jts,jte,kts,kte)
  end subroutine
end program
