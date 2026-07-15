program registry_backed_arw_trajectory_benchmark
  ! Matched canonical benchmark for the accepted WRF v4.7.1 ARW trajectory.
  ! The runner extracts every routine body from the checksum-pinned sources.
  use module_configure, only: grid_config_rec_type
  use extracted_big_step_column_mass, only: calc_mu_uv, calc_mu_uv_1
  use extracted_acoustic_trajectory, only: small_step_prep, calc_p_rho, calc_coef_w, &
    advance_uv, advance_mu_t, advance_w, sumflux, small_step_finish
  use extracted_kessler_trajectory, only: moist_physics_prep_em, moist_physics_finish_em
  use module_mp_kessler, only: kessler
  implicit none

  integer, parameter :: ims=0,ime=63,jms=0,jme=63,kms=0,kme=39
  integer, parameter :: ids=1,ide=63,jds=1,jde=63,kds=1,kde=39
  integer, parameter :: its=1,ite=63,jts=1,jte=63,kts=1,kte=39
  integer, parameter :: n_moist=4,p_qv=2,p_qc=3,p_qr=4
  integer, parameter :: warmup_count=3,sample_count=21
  real, parameter :: t0=300.0,p0=100000.0,dt=1.0,acoustic_dt=0.000001
  real, parameter :: r_d=287.0,r_v=461.6,cp=7.0*r_d/2.0
  real, parameter :: xlv=2.5e6,ep2=r_d/r_v
  real, parameter :: svp1=0.6112,svp2=17.67,svp3=29.65,svpt0=273.15
  real, parameter :: rhowater=1000.0,sentinel=-7777.0

  type(grid_config_rec_type) :: config
  integer :: i,j,k,s,iteration,sample
  real :: start_time,end_time
  real(kind=8) :: elapsed_milliseconds(sample_count),checksum_accumulator
  real :: u1(ims:ime,kms:kme,jms:jme),u2(ims:ime,kms:kme,jms:jme)
  real :: v1(ims:ime,kms:kme,jms:jme),v2(ims:ime,kms:kme,jms:jme)
  real :: w1(ims:ime,kms:kme,jms:jme),w2(ims:ime,kms:kme,jms:jme)
  real :: t1(ims:ime,kms:kme,jms:jme),t2(ims:ime,kms:kme,jms:jme)
  real :: ph1(ims:ime,kms:kme,jms:jme),ph2(ims:ime,kms:kme,jms:jme)
  real :: phb(ims:ime,kms:kme,jms:jme),pb(ims:ime,kms:kme,jms:jme)
  real :: p(ims:ime,kms:kme,jms:jme),al(ims:ime,kms:kme,jms:jme)
  real :: alt(ims:ime,kms:kme,jms:jme),alb(ims:ime,kms:kme,jms:jme)
  real :: php(ims:ime,kms:kme,jms:jme)
  real :: moist(ims:ime,kms:kme,jms:jme,n_moist)
  real :: ru(ims:ime,kms:kme,jms:jme),rv(ims:ime,kms:kme,jms:jme)
  real :: rw(ims:ime,kms:kme,jms:jme),ww(ims:ime,kms:kme,jms:jme)
  real :: cqu(ims:ime,kms:kme,jms:jme),cqv(ims:ime,kms:kme,jms:jme)
  real :: cqw(ims:ime,kms:kme,jms:jme)
  real :: us(ims:ime,kms:kme,jms:jme),vs(ims:ime,kms:kme,jms:jme)
  real :: ws(ims:ime,kms:kme,jms:jme),ts(ims:ime,kms:kme,jms:jme)
  real :: phs(ims:ime,kms:kme,jms:jme),ww1(ims:ime,kms:kme,jms:jme)
  real :: c2a(ims:ime,kms:kme,jms:jme),pm1(ims:ime,kms:kme,jms:jme)
  real :: a(ims:ime,kms:kme,jms:jme),alpha(ims:ime,kms:kme,jms:jme)
  real :: gamma(ims:ime,kms:kme,jms:jme),t2save(ims:ime,kms:kme,jms:jme)
  real :: ru_m(ims:ime,kms:kme,jms:jme),rv_m(ims:ime,kms:kme,jms:jme)
  real :: ww_m(ims:ime,kms:kme,jms:jme)
  real :: ru_tend(ims:ime,kms:kme,jms:jme),rv_tend(ims:ime,kms:kme,jms:jme)
  real :: rw_tend(ims:ime,kms:kme,jms:jme),t_tend(ims:ime,kms:kme,jms:jme)
  real :: ph_tend(ims:ime,kms:kme,jms:jme)
  real :: ruf(ims:ime,kms:kme,jms:jme),rvf(ims:ime,kms:kme,jms:jme)
  real :: rwf(ims:ime,kms:kme,jms:jme),tf(ims:ime,kms:kme,jms:jme)
  real :: phf(ims:ime,kms:kme,jms:jme),h_diabatic(ims:ime,kms:kme,jms:jme)
  real :: rho(ims:ime,kms:kme,jms:jme),p8w(ims:ime,kms:kme,jms:jme)
  real :: th_phy(ims:ime,kms:kme,jms:jme),pi_phy(ims:ime,kms:kme,jms:jme)
  real :: p_phy(ims:ime,kms:kme,jms:jme),z(ims:ime,kms:kme,jms:jme)
  real :: z_at_w(ims:ime,kms:kme,jms:jme),dz8w(ims:ime,kms:kme,jms:jme)
  real :: qv_diabatic(ims:ime,kms:kme,jms:jme)
  real :: qc_diabatic(ims:ime,kms:kme,jms:jme)
  real :: th_phy_m_t0(ims:ime,kms:kme,jms:jme)
  real :: mu1(ims:ime,jms:jme),mu2(ims:ime,jms:jme),mus(ims:ime,jms:jme)
  real :: mub(ims:ime,jms:jme),muu(ims:ime,jms:jme),muv(ims:ime,jms:jme)
  real :: mut(ims:ime,jms:jme),mu_tend(ims:ime,jms:jme),muf(ims:ime,jms:jme)
  real :: muus(ims:ime,jms:jme),muvs(ims:ime,jms:jme),muts(ims:ime,jms:jme)
  real :: mudf(ims:ime,jms:jme),muave(ims:ime,jms:jme)
  real :: rainnc(ims:ime,jms:jme),rainncv(ims:ime,jms:jme)
  real :: msfux(ims:ime,jms:jme),msfuy(ims:ime,jms:jme)
  real :: msfvx(ims:ime,jms:jme),msfvx_inv(ims:ime,jms:jme)
  real :: msfvy(ims:ime,jms:jme),msftx(ims:ime,jms:jme),msfty(ims:ime,jms:jme)
  real :: ht(ims:ime,jms:jme)
  real :: c1h(kms:kme),c2h(kms:kme),c1f(kms:kme),c2f(kms:kme)
  real :: c3h(kms:kme),c4h(kms:kme),c3f(kms:kme),c4f(kms:kme)
  real :: znu(kms:kme),dnw(kms:kme),rdnw(kms:kme),rdn(kms:kme)
  real :: fnm(kms:kme),fnp(kms:kme)

  checksum_accumulator=0.0_8
  do sample=1,warmup_count
    call initialize_fixture()
    call run_accepted_trajectory()
    call consume_result()
  end do

  do sample=1,sample_count
    call initialize_fixture()
    call cpu_time(start_time)
    call run_accepted_trajectory()
    call cpu_time(end_time)
    elapsed_milliseconds(sample)=1000.0_8*dble(end_time-start_time)
    call consume_result()
  end do
  do sample=1,sample_count
    write(*,'(A,1X,F0.6)') 'canonical_continuous',elapsed_milliseconds(sample)
  end do
  call sort_samples(elapsed_milliseconds)
  if (checksum_accumulator /= checksum_accumulator) error stop 'non-finite benchmark checksum'
  write(*,'(A,1X,ES24.16)') 'checksum',checksum_accumulator

contains

  subroutine run_accepted_trajectory()
    call calculate_full(mut,mub,mu2,ids,ide,jds,jde,1,2,ims,ime,jms,jme,1,1,its,ite,jts,jte,1,1)
    call calc_mu_uv(config,mu2,mub,muu,muv,ids,ide,jds,jde,kds,kde, &
      ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kts,kte)
    call couple_momentum(muu,ru,u2,msfuy,muv,rv,v2,msfvx,msfvx_inv,mut,rw,w2,msfty, &
      c1h,c2h,c1f,c2f,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      its,ite,jts,jte,kts,kte)
    call calc_ww_cp(u2,v2,mu2,mub,c1h,c2h,ww,.1,.1,msftx,msfty,msfux,msfuy, &
      msfvx,msfvx_inv,msfvy,dnw,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      its,ite,jts,jte,kts,kte)
    call calc_cq(moist,cqu,cqv,cqw,n_moist,ids,ide,jds,jde,kds,kde, &
      ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kts,kte)
    call calc_alt(alt,al,alb,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      its,ite,jts,jte,kts,kte)
    call calc_php(php,ph2,phb,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      its,ite,jts,jte,kts,kte)

    call rk_addtend_dry(ru_tend,rv_tend,rw_tend,ph_tend,t_tend,ruf,rvf,rwf,phf,tf, &
      us,vs,ws,phs,ts,mu_tend,muf,1,c1h,c2h,h_diabatic,mut,msftx,msfty, &
      msfux,msfuy,msfvx,msfvx_inv,msfvy,ids,ide,jds,jde,kds,kde, &
      ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kts,kte)

    call small_step_prep(u1,u2,v1,v2,w1,w2,t1,t2,ph1,ph2,mub,mu1,mu2, &
      muu,muus,muv,muvs,mut,muts,mudf,c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f, &
      us,vs,ws,ts,phs,mus,ww,ww1,c2a,pb,p,alt,msfux,msfuy,msfvx,msfvx_inv, &
      msfvy,msftx,msfty,.1,.1,1,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme, &
      kms,kme,its,ite,jts,jte,kts,kte)
    call calc_p_rho(al,p,ph2,alt,t2,ts,c2a,pm1,mu2,muts,c1h,c2h,c1f,c2f, &
      c3h,c4h,c3f,c4f,znu,t0,rdnw,dnw,0.,.true.,0,ids,ide,jds,jde,kds,kde, &
      ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kts,kte)
    call calc_coef_w(a,alpha,gamma,mut,c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f, &
      cqw,rdn,rdnw,c2a,acoustic_dt,9.81,.1,.false.,ids,ide,jds,jde,kds,kde, &
      ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kts,kte)

    do iteration=1,3
      call advance_uv(u2,ru_tend,v2,rv_tend,p,pb,ph2,php,alt,al,mu2,muu,cqu, &
        muv,cqv,mudf,c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f,msfux,msfuy,msfvx, &
        msfvx_inv,msfvy,.1,.1,acoustic_dt,.5,.3,.2,fnm,fnp,0.,rdnw,config,0, &
        .true.,.false.,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
        its,ite,jts,jte,kts,kte)
      call advance_mu_t(ww,ww1,u2,us,v2,vs,mu2,mut,muave,muts,muu,muv,mudf, &
        c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f,ru_m,rv_m,ww_m,t2,ts,t2save,t_tend, &
        mu_tend,.1,.1,acoustic_dt,.1,dnw,fnm,fnp,rdnw,msfux,msfuy,msfvx,msfvx_inv, &
        msfvy,msftx,msfty,iteration,config,ids,ide,jds,jde,kds,kde, &
        ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kts,kte)
      call advance_w(w2,rw_tend,ww,ws,u2,v2,mu2,mut,muave,muts,c1h,c2h,c1f,c2f, &
        c3h,c4h,c3f,c4f,t2save,t2,ts,ph2,phs,phb,ph_tend,ht,c2a,cqw,alt,al, &
        a,alpha,gamma,.1,.1,acoustic_dt,t0,.1,dnw,fnm,fnp,rdnw,rdn,.5,.3,.2, &
        msftx,msfty,config,.false.,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme, &
        kms,kme,its,ite,jts,jte,kts,kte)
      call sumflux(u2,v2,ww,us,vs,ww1,muu,muv,c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f, &
        ru_m,rv_m,ww_m,.1,msfux,msfuy,msfvx,msfvx_inv,msfvy,iteration,3, &
        ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kts,kte)
      call calc_p_rho(al,p,ph2,alt,t2,ts,c2a,pm1,mu2,muts,c1h,c2h,c1f,c2f, &
        c3h,c4h,c3f,c4f,znu,t0,rdnw,dnw,0.,.true.,iteration, &
        ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kts,kte)
    end do

    call calc_mu_uv_1(config,muts,muus,muvs,ids,ide,jds,jde,kds,kde, &
      ims,ime,jms,jme,kms,kme,its,ite,jts,jte,kts,kte)
    call small_step_finish(u2,u1,v2,v1,w2,w1,t2,t1,ph2,ph1,ww,ww1,mu2,mu1, &
      mut,muts,muu,muus,muv,muvs,c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f, &
      us,vs,ws,ts,phs,mus,msfux,msfuy,msfvx,msfvy,msftx,msfty,h_diabatic, &
      3,acoustic_dt,1,3,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      its,ite,jts,jte,kts,kte)

    call moist_physics_prep_em(t2,t1,t0,rho,al,alb,p,p8w,p0,pb,ph2,phb, &
      th_phy,pi_phy,p_phy,z,z_at_w,dz8w,dt,h_diabatic, &
      moist(:,:,:,p_qv),qv_diabatic,moist(:,:,:,p_qc),qc_diabatic, &
      config,fnm,fnp,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      its,ide-1,jts,jde-1,kts,kte)
    call kessler(th_phy,moist(:,:,:,p_qv),moist(:,:,:,p_qc),moist(:,:,:,p_qr), &
      rho,pi_phy,dt,z,xlv,cp,ep2,svp1,svp2,svp3,svpt0,rhowater,dz8w,rainnc,rainncv, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,its,ide-1,jts,jde-1,kts,kde-1)
    call moist_physics_finish_em(t2,t1,t0,muts,th_phy,h_diabatic,dt, &
      moist(:,:,:,p_qv),qv_diabatic,moist(:,:,:,p_qc),qc_diabatic,th_phy_m_t0, &
      config,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      its,ide-1,jts,jde-1,kts,kte)
  end subroutine run_accepted_trajectory

  subroutine initialize_fixture()
    config=grid_config_rec_type()
    config%periodic_x=.false.;config%periodic_y=.false.
    config%use_theta_m=1;config%no_mp_heating=0;config%mp_tend_lim=10.0
    do k=kms,kme
      c1h(k)=.60+.002*real(k);c2h(k)=.40-.001*real(k)
      c1f(k)=.55+.002*real(k);c2f(k)=.45-.001*real(k)
      c3h(k)=0.;c4h(k)=0.;c3f(k)=0.;c4f(k)=0.
      znu(k)=1.;dnw(k)=.20;rdnw(k)=1.;rdn(k)=1.;fnm(k)=.60;fnp(k)=.40
    end do
    do j=jms,jme
      do i=ims,ime
        mu1(i,j)=1.+.01*real(i+j);mu2(i,j)=mu1(i,j);mus(i,j)=.2
        mub(i,j)=10.+.1*real(i)+.2*real(j);muu(i,j)=sentinel;muv(i,j)=sentinel
        mut(i,j)=sentinel;mu_tend(i,j)=.01+.001*real(i-j);muf(i,j)=.02
        muus(i,j)=.2;muvs(i,j)=.2;muts(i,j)=.2;mudf(i,j)=0.;muave(i,j)=0.
        rainnc(i,j)=1.+.1*real(i+j);rainncv(i,j)=sentinel
        msfux(i,j)=1.;msfuy(i,j)=1.;msfvx(i,j)=1.;msfvx_inv(i,j)=1.
        msfvy(i,j)=1.;msftx(i,j)=1.;msfty(i,j)=1.;ht(i,j)=0.
      end do
      do k=kms,kme
        do i=ims,ime
          u1(i,k,j)=.20+.001*real(i+2*k-j);u2(i,k,j)=u1(i,k,j)
          v1(i,k,j)=.15+.001*real(2*i-k+j);v2(i,k,j)=v1(i,k,j)
          w1(i,k,j)=.05+.001*real(i+k+j);w2(i,k,j)=w1(i,k,j)
          t1(i,k,j)=-10.+.02*real(i+k-j);t2(i,k,j)=t1(i,k,j)
          ph1(i,k,j)=10.+real(k)+.01*real(i-j);ph2(i,k,j)=ph1(i,k,j)
          phb(i,k,j)=1000.+50.*real(k);pb(i,k,j)=80000.-1000.*real(k)
          p(i,k,j)=100.+real(i-j);al(i,k,j)=.03+.001*real(i+j);alt(i,k,j)=sentinel
          alb(i,k,j)=.84+.02*real(k);php(i,k,j)=sentinel
          us(i,k,j)=.09;vs(i,k,j)=-.08;ws(i,k,j)=.07;ts(i,k,j)=.05;phs(i,k,j)=-.06
          ww1(i,k,j)=.2;c2a(i,k,j)=.2;pm1(i,k,j)=.2
          a(i,k,j)=.2;alpha(i,k,j)=.2;gamma(i,k,j)=.2;t2save(i,k,j)=.2
          ru_m(i,k,j)=.2;rv_m(i,k,j)=.2;ww_m(i,k,j)=.2
          ru_tend(i,k,j)=.01+.0001*real(i+k+j)
          rv_tend(i,k,j)=.011+.0001*real(i+k+j)
          rw_tend(i,k,j)=.012+.0001*real(i+k+j)
          ph_tend(i,k,j)=.013+.0001*real(i+k+j)
          t_tend(i,k,j)=.014+.0001*real(i+k+j)
          ruf(i,k,j)=.003;rvf(i,k,j)=.004;rwf(i,k,j)=.005;phf(i,k,j)=.006;tf(i,k,j)=.007
          h_diabatic(i,k,j)=.0001
          rho(i,k,j)=sentinel;p8w(i,k,j)=sentinel;th_phy(i,k,j)=sentinel
          pi_phy(i,k,j)=sentinel;p_phy(i,k,j)=sentinel;z(i,k,j)=sentinel
          z_at_w(i,k,j)=sentinel;dz8w(i,k,j)=sentinel
          qv_diabatic(i,k,j)=0.;qc_diabatic(i,k,j)=0.;th_phy_m_t0(i,k,j)=sentinel
          do s=1,n_moist
            moist(i,k,j,s)=sentinel
          end do
          moist(i,k,j,p_qv)=.006+.0001*real(mod(i+2*k+j,7))
          moist(i,k,j,p_qc)=.001+.0001*real(mod(i+k,3))
          moist(i,k,j,p_qr)=.0005+.0002*real(mod(i+j,4))
          ru(i,k,j)=sentinel;rv(i,k,j)=sentinel;rw(i,k,j)=sentinel;ww(i,k,j)=sentinel
          cqu(i,k,j)=sentinel;cqv(i,k,j)=sentinel;cqw(i,k,j)=sentinel
        end do
      end do
    end do
  end subroutine initialize_fixture

  subroutine consume_result()
    checksum_accumulator=checksum_accumulator+dble(t2(17,19,23))+dble(ph2(31,37,29)) &
      +dble(moist(47,11,43,p_qv))+dble(rainnc(53,59))+dble(mu2(61,7))
  end subroutine consume_result

  subroutine sort_samples(values)
    real(kind=8),intent(inout) :: values(:)
    integer :: current,previous
    real(kind=8) :: value
    do current=2,size(values)
      value=values(current)
      previous=current-1
      do while (previous >= 1)
        if (values(previous) <= value) exit
        values(previous+1)=values(previous)
        previous=previous-1
      end do
      values(previous+1)=value
    end do
  end subroutine sort_samples
end program registry_backed_arw_trajectory_benchmark
