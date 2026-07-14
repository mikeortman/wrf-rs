module kessler_trajectory_projection
  ! Matched dependency-closed projection of the Kessler-relevant ARW
  ! preparation and finish expressions. The live-source oracle separately
  ! checks these expressions against pinned WRF v4.7.1 routines before this
  ! benchmark is accepted.
  implicit none
  integer, parameter :: projection_nx=128, projection_ny=128
  integer, parameter :: projection_nz=40, projection_nw=projection_nz+1
  real, parameter :: projection_dt=60.0, projection_t0=300.0
  real, parameter :: projection_p0=100000.0, projection_gravity=9.81
  real, parameter :: projection_r_d=287.0, projection_r_v=461.6
  real, parameter :: projection_rcp=projection_r_d/(7.0*projection_r_d/2.0)
  real, parameter :: projection_vapor_ratio=projection_r_v/projection_r_d
  real, parameter :: projection_maximum_theta_tendency=10.0

contains

  subroutine prepare_kessler_projection(t_new,qv,qc,al,alb,p,pb,ph,phb, &
      th_phy,h_diabatic,qv_diabatic,qc_diabatic,rho,pii,z,dz8w)
    real, intent(in) :: t_new(projection_nx,projection_nz,projection_ny)
    real, intent(in) :: qv(projection_nx,projection_nz,projection_ny)
    real, intent(in) :: qc(projection_nx,projection_nz,projection_ny)
    real, intent(in) :: al(projection_nx,projection_nz,projection_ny)
    real, intent(in) :: alb(projection_nx,projection_nz,projection_ny)
    real, intent(in) :: p(projection_nx,projection_nz,projection_ny)
    real, intent(in) :: pb(projection_nx,projection_nz,projection_ny)
    real, intent(in) :: ph(projection_nx,projection_nw,projection_ny)
    real, intent(in) :: phb(projection_nx,projection_nw,projection_ny)
    real, intent(out) :: th_phy(projection_nx,projection_nz,projection_ny)
    real, intent(out) :: h_diabatic(projection_nx,projection_nz,projection_ny)
    real, intent(out) :: qv_diabatic(projection_nx,projection_nz,projection_ny)
    real, intent(out) :: qc_diabatic(projection_nx,projection_nz,projection_ny)
    real, intent(out) :: rho(projection_nx,projection_nz,projection_ny)
    real, intent(out) :: pii(projection_nx,projection_nz,projection_ny)
    real, intent(out) :: z(projection_nx,projection_nz,projection_ny)
    real, intent(out) :: dz8w(projection_nx,projection_nz,projection_ny)
    real :: lower_height, upper_height
    integer :: i, j, k

    do j=1,projection_ny
      do k=1,projection_nz
        do i=1,projection_nx
          th_phy(i,k,j)=(t_new(i,k,j)+projection_t0) &
            /(1.0+projection_vapor_ratio*qv(i,k,j))
          h_diabatic(i,k,j)=th_phy(i,k,j)
          qv_diabatic(i,k,j)=qv(i,k,j)
          qc_diabatic(i,k,j)=qc(i,k,j)
          rho(i,k,j)=1.0/(al(i,k,j)+alb(i,k,j))
          pii(i,k,j)=((p(i,k,j)+pb(i,k,j))/projection_p0)**projection_rcp
          lower_height=(ph(i,k,j)+phb(i,k,j))/projection_gravity
          upper_height=(ph(i,k+1,j)+phb(i,k+1,j))/projection_gravity
          z(i,k,j)=0.5*(lower_height+upper_height)
          dz8w(i,k,j)=upper_height-lower_height
        end do
      end do
    end do
  end subroutine prepare_kessler_projection

  subroutine finish_kessler_projection(t_new,qv,qc,th_phy,h_diabatic, &
      qv_diabatic,qc_diabatic,th_phy_m_t0)
    real, intent(inout) :: t_new(projection_nx,projection_nz,projection_ny)
    real, intent(in) :: qv(projection_nx,projection_nz,projection_ny)
    real, intent(in) :: qc(projection_nx,projection_nz,projection_ny)
    real, intent(in) :: th_phy(projection_nx,projection_nz,projection_ny)
    real, intent(inout) :: h_diabatic(projection_nx,projection_nz,projection_ny)
    real, intent(inout) :: qv_diabatic(projection_nx,projection_nz,projection_ny)
    real, intent(inout) :: qc_diabatic(projection_nx,projection_nz,projection_ny)
    real, intent(out) :: th_phy_m_t0(projection_nx,projection_nz,projection_ny)
    real :: theta_change, qv_change, qc_change, updated_theta
    integer :: i, j, k

    do j=1,projection_ny
      do k=1,projection_nz
        do i=1,projection_nx
          qv_change=qv(i,k,j)-qv_diabatic(i,k,j)
          qc_change=qc(i,k,j)-qc_diabatic(i,k,j)
          theta_change=th_phy(i,k,j)-h_diabatic(i,k,j)
          theta_change=min(projection_maximum_theta_tendency*projection_dt,theta_change)
          theta_change=max(-projection_maximum_theta_tendency*projection_dt,theta_change)
          updated_theta=h_diabatic(i,k,j)*(1.0+projection_vapor_ratio*qv_diabatic(i,k,j)) &
            +theta_change*(1.0+projection_vapor_ratio*qv(i,k,j)) &
            +projection_vapor_ratio*qv_change*th_phy(i,k,j)-projection_t0
          t_new(i,k,j)=updated_theta
          th_phy_m_t0(i,k,j)=(updated_theta+projection_t0) &
            /(1.0+projection_vapor_ratio*qv(i,k,j))-projection_t0
          h_diabatic(i,k,j)=(theta_change*(1.0+projection_vapor_ratio*qv(i,k,j)) &
            +projection_vapor_ratio*qv_change*th_phy(i,k,j))/projection_dt
          qv_diabatic(i,k,j)=qv_change/projection_dt
          qc_diabatic(i,k,j)=qc_change/projection_dt
        end do
      end do
    end do
  end subroutine finish_kessler_projection

end module kessler_trajectory_projection

program kessler_precipitation_trajectory_benchmark
  use iso_fortran_env, only: int64, real64
  use kessler_trajectory_projection, only: prepare_kessler_projection, finish_kessler_projection
  use module_mp_kessler, only: kessler
  implicit none

  integer, parameter :: nx=128, ny=128, nz=40, nw=nz+1
  integer, parameter :: ims=1, ime=nx, jms=1, jme=ny, kms=1, kme=nz
  integer, parameter :: ids=1, ide=nx+1, jds=1, jde=ny+1, kds=1, kde=nw
  integer, parameter :: its=1, ite=nx, jts=1, jte=ny, kts=1, kte=nw
  integer, parameter :: sample_count=31, calls_per_sample=1, warmup_calls=3
  integer, parameter :: trajectory_steps=3
  real, parameter :: dt=60.0
  real, parameter :: xlv=2.5e6, cp=7.0*287.0/2.0, ep2=287.0/461.6
  real, parameter :: svp1=0.6112, svp2=17.67, svp3=29.65, svpt0=273.15
  real, parameter :: rhowater=1000.0
  real, allocatable :: t_new(:,:,:), al(:,:,:), alb(:,:,:)
  real, allocatable :: p(:,:,:), pb(:,:,:), ph(:,:,:), phb(:,:,:)
  real, allocatable :: rho(:,:,:), th_phy(:,:,:), pii(:,:,:)
  real, allocatable :: z(:,:,:), dz8w(:,:,:)
  real, allocatable :: h_diabatic(:,:,:), qv_diabatic(:,:,:), qc_diabatic(:,:,:)
  real, allocatable :: qv(:,:,:), qc(:,:,:), qr(:,:,:), th_phy_m_t0(:,:,:)
  real, allocatable :: initial_t_new(:,:,:), initial_qv(:,:,:), initial_qc(:,:,:), initial_qr(:,:,:)
  real, allocatable :: rainnc(:,:), rainncv(:,:), initial_rainnc(:,:)
  integer(int64) :: started, finished, rate, elapsed
  real(real64) :: milliseconds, checksum
  integer :: sample, iteration, i, j, k

  allocate(t_new(ims:ime,kms:kme,jms:jme))
  allocate(al(ims:ime,kms:kme,jms:jme),alb(ims:ime,kms:kme,jms:jme))
  allocate(p(ims:ime,kms:kme,jms:jme),pb(ims:ime,kms:kme,jms:jme))
  allocate(ph(ims:ime,kms:nw,jms:jme),phb(ims:ime,kms:nw,jms:jme))
  allocate(rho(ims:ime,kms:kme,jms:jme))
  allocate(th_phy(ims:ime,kms:kme,jms:jme),pii(ims:ime,kms:kme,jms:jme))
  allocate(z(ims:ime,kms:kme,jms:jme),dz8w(ims:ime,kms:kme,jms:jme))
  allocate(h_diabatic(ims:ime,kms:kme,jms:jme),qv_diabatic(ims:ime,kms:kme,jms:jme))
  allocate(qc_diabatic(ims:ime,kms:kme,jms:jme),qv(ims:ime,kms:kme,jms:jme))
  allocate(qc(ims:ime,kms:kme,jms:jme),qr(ims:ime,kms:kme,jms:jme))
  allocate(th_phy_m_t0(ims:ime,kms:kme,jms:jme))
  allocate(initial_t_new(ims:ime,kms:kme,jms:jme),initial_qv(ims:ime,kms:kme,jms:jme))
  allocate(initial_qc(ims:ime,kms:kme,jms:jme),initial_qr(ims:ime,kms:kme,jms:jme))
  allocate(rainnc(ims:ime,jms:jme),rainncv(ims:ime,jms:jme))
  allocate(initial_rainnc(ims:ime,jms:jme))

  do j=jms,jme
    do k=kms,kme
      do i=ims,ime
        initial_t_new(i,k,j)=-21.0+0.007*real(i-1)+0.03*real(k-1)-0.004*real(j-1)
        initial_qv(i,k,j)=0.002+0.001*real(mod((i-1)+2*(k-1),8))
        if (mod((i-1)+(k-1),3)==0) then
          initial_qc(i,k,j)=0.002
        else
          initial_qc(i,k,j)=0.0002
        end if
        select case (mod((i-1)+(j-1),4))
        case (0); initial_qr(i,k,j)=0.0
        case (1); initial_qr(i,k,j)=0.0005
        case (2); initial_qr(i,k,j)=0.005
        case default; initial_qr(i,k,j)=0.02
        end select
        al(i,k,j)=0.83+0.006*real(k-1)
        alb(i,k,j)=0.02
        p(i,k,j)=-500.0*real(k-1)
        pb(i,k,j)=100000.0-1500.0*real(k-1)
      end do
    end do
    do k=kms,nw
      do i=ims,ime
        ph(i,k,j)=9.81*(50.0+150.0*real(k-1))
        phb(i,k,j)=0.0
      end do
    end do
  end do
  initial_rainnc=10.0

  do iteration=1,warmup_calls
    call reset_mutable_fields
    call invoke_trajectory
  end do
  call system_clock(count_rate=rate)
  do sample=1,sample_count
    elapsed=0_int64
    do iteration=1,calls_per_sample
      call reset_mutable_fields
      call system_clock(started)
      call invoke_trajectory
      call system_clock(finished)
      elapsed=elapsed+finished-started
    end do
    milliseconds=real(elapsed,real64)*1000.0_real64/real(rate,real64)/real(calls_per_sample,real64)
    write(*,'(A,I0,A,F12.6)')'sample_',sample,'_milliseconds_per_call ',milliseconds
  end do
  checksum=sum(real(t_new,real64))+sum(real(qv,real64))+sum(real(qc,real64)) &
    +sum(real(qr,real64))+sum(real(rainnc,real64))+sum(real(rainncv,real64)) &
    +sum(real(h_diabatic,real64))+sum(real(qv_diabatic,real64))+sum(real(qc_diabatic,real64))
  write(*,'(A,I0)')'grid_points_per_call ',nx*ny*nz*trajectory_steps
  write(*,'(A,ES24.16)')'checksum ',checksum

contains

  subroutine reset_mutable_fields
    t_new=initial_t_new
    qv=initial_qv
    qc=initial_qc
    qr=initial_qr
    rainnc=initial_rainnc
    rainncv=0.0
    rho=0.0; th_phy=0.0; pii=0.0; z=0.0; dz8w=0.0
    h_diabatic=0.0; qv_diabatic=0.0; qc_diabatic=0.0; th_phy_m_t0=0.0
  end subroutine reset_mutable_fields

  subroutine invoke_trajectory
    integer :: step
    do step=1,trajectory_steps
      call prepare_kessler_projection(t_new,qv,qc,al,alb,p,pb,ph,phb, &
        th_phy,h_diabatic,qv_diabatic,qc_diabatic,rho,pii,z,dz8w)
      call kessler(th_phy,qv,qc,qr,rho,pii,dt,z,xlv,cp,ep2,svp1,svp2,svp3,svpt0, &
        rhowater,dz8w,rainnc,rainncv,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme, &
        kms,kme,its,ite,jts,jte,kts,nz)
      call finish_kessler_projection(t_new,qv,qc,th_phy,h_diabatic, &
        qv_diabatic,qc_diabatic,th_phy_m_t0)
    end do
  end subroutine invoke_trajectory

end program kessler_precipitation_trajectory_benchmark
