program acoustic_mass_theta_benchmark
  use module_configure, only: grid_config_rec_type
  use extracted_acoustic_mass_theta, only: advance_mu_t
  implicit none
  integer,parameter::ims=0,ime=257,jms=0,jme=257,kms=0,kme=41
  integer,parameter::ids=1,ide=257,jds=1,jde=257,kds=1,kde=41
  integer,parameter::calls_per_sample=2
  real,allocatable::ww(:,:,:),ww1(:,:,:),u(:,:,:),u1(:,:,:),v(:,:,:),v1(:,:,:)
  real,allocatable::t(:,:,:),t1(:,:,:),tave(:,:,:),ft(:,:,:),unused_volume(:,:,:)
  real,allocatable::mu(:,:),mut(:,:),muave(:,:),muts(:,:),muu(:,:),muv(:,:),mudf(:,:),mutend(:,:)
  real,allocatable::maps(:,:,:),vertical(:,:)
  type(grid_config_rec_type)::config
  integer::sample,iteration
  integer(kind=8)::started,finished,rate
  allocate(ww(ims:ime,kms:kme,jms:jme),ww1(ims:ime,kms:kme,jms:jme))
  allocate(u(ims:ime,kms:kme,jms:jme),u1(ims:ime,kms:kme,jms:jme))
  allocate(v(ims:ime,kms:kme,jms:jme),v1(ims:ime,kms:kme,jms:jme))
  allocate(t(ims:ime,kms:kme,jms:jme),t1(ims:ime,kms:kme,jms:jme))
  allocate(tave(ims:ime,kms:kme,jms:jme),ft(ims:ime,kms:kme,jms:jme))
  allocate(unused_volume(ims:ime,kms:kme,jms:jme))
  allocate(mu(ims:ime,jms:jme),mut(ims:ime,jms:jme),muave(ims:ime,jms:jme))
  allocate(muts(ims:ime,jms:jme),muu(ims:ime,jms:jme),muv(ims:ime,jms:jme))
  allocate(mudf(ims:ime,jms:jme),mutend(ims:ime,jms:jme))
  allocate(maps(ims:ime,jms:jme,7),vertical(kms:kme,6))
  ww=.8;ww1=.35;u=.2;u1=.15;v=.3;v1=.12;t=300.;t1=290.;tave=-901.;ft=.012
  unused_volume=0.;mu=2.;mut=11.;muave=-902.;muts=-903.;muu=3.;muv=4.;mudf=-904.;mutend=.03
  maps=1.;maps(:,:,2)=.92;maps(:,:,4)=1.08;maps(:,:,6)=1.03;maps(:,:,7)=.97
  vertical(:,1)=.45;vertical(:,2)=.2;vertical(:,3)=.18
  vertical(:,4)=.61;vertical(:,5)=.39;vertical(:,6)=1.1
  do iteration=1,3
    call invoke
  enddo
  call system_clock(count_rate=rate)
  do sample=1,11
    call system_clock(started)
    do iteration=1,calls_per_sample
      call invoke
    enddo
    call system_clock(finished)
    write(*,'(F12.6)')real(finished-started)/real(rate)*1000./real(calls_per_sample)
  enddo
contains
  subroutine invoke
    call advance_mu_t(ww,ww1,u,u1,v,v1,mu,mut,muave,muts,muu,muv,mudf, &
      vertical(:,1),vertical(:,2),vertical(:,1),vertical(:,2),vertical(:,1), &
      vertical(:,2),vertical(:,1),vertical(:,2),unused_volume,unused_volume, &
      unused_volume,t,t1,tave,ft,mutend,.002,.003,.4,.1,vertical(:,3), &
      vertical(:,4),vertical(:,5),vertical(:,6),maps(:,:,1),maps(:,:,2), &
      maps(:,:,3),maps(:,:,4),maps(:,:,5),maps(:,:,6),maps(:,:,7),2,config, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,1,257,1,257,1,41)
  end subroutine
end program
