program acoustic_vertical_momentum_benchmark
  use module_configure, only: grid_config_rec_type
  use extracted_acoustic_vertical_momentum, only: advance_w
  implicit none
  integer,parameter::ims=0,ime=257,jms=0,jme=257,kms=0,kme=41
  integer,parameter::ids=1,ide=257,jds=1,jde=257,kds=1,kde=41
  real,allocatable::volume(:,:,:,:),horizontal(:,:,:),vertical(:,:)
  type(grid_config_rec_type)::config
  integer::sample,iteration
  integer(kind=8)::started,finished,rate
  real::checksum
  allocate(volume(ims:ime,kms:kme,jms:jme,20))
  allocate(horizontal(ims:ime,jms:jme,7),vertical(kms:kme,9))
  volume(:,:,:,1)=.8;volume(:,:,:,2)=.012;volume(:,:,:,3)=.35
  volume(:,:,:,4)=.31;volume(:,:,:,5)=.2;volume(:,:,:,6)=.3
  volume(:,:,:,7)=294.;volume(:,:,:,8)=300.;volume(:,:,:,9)=1.3
  volume(:,:,:,10)=20.;volume(:,:,:,11)=18.;volume(:,:,:,12)=30000.
  volume(:,:,:,13)=.05;volume(:,:,:,14)=1.1;volume(:,:,:,15)=.82
  volume(:,:,:,16)=.9;volume(:,:,:,17)=0.;volume(:,:,:,18)=-.03
  volume(:,:,:,19)=.83;volume(:,:,:,20)=-.02
  horizontal(:,:,1)=0.;horizontal(:,:,2)=11.;horizontal(:,:,3)=2.1
  horizontal(:,:,4)=12.7;horizontal(:,:,5)=140.;horizontal(:,:,6)=1.03
  horizontal(:,:,7)=.97
  vertical(:,1)=0.;vertical(:,2)=.42;vertical(:,3)=.19
  vertical(:,4)=.37;vertical(:,5)=.23;vertical(:,6)=.58
  vertical(:,7)=.42;vertical(:,8)=1.05;vertical(:,9)=.91
  config%phi_adv_z=2;config%damp_opt=3;config%dampcoef=.15;config%zdamp=220.
  do iteration=1,3
    call invoke
  enddo
  call system_clock(count_rate=rate)
  do sample=1,11
    call system_clock(started)
    call invoke
    call system_clock(finished)
    write(*,'(F12.6)')real(finished-started)/real(rate)*1000.
  enddo
  checksum=sum(volume(:,:,:,1))+sum(volume(:,:,:,10))+sum(volume(:,:,:,7))
  write(*,'(A,1X,ES16.8)')'checksum',checksum
contains
  subroutine invoke
    call advance_w(volume(:,:,:,1),volume(:,:,:,2),volume(:,:,:,3),volume(:,:,:,4), &
      volume(:,:,:,5),volume(:,:,:,6),horizontal(:,:,1),horizontal(:,:,2), &
      horizontal(:,:,3),horizontal(:,:,4),vertical(:,2),vertical(:,3),vertical(:,4), &
      vertical(:,5),vertical(:,1),vertical(:,1),vertical(:,1),vertical(:,1), &
      volume(:,:,:,7),volume(:,:,:,8),volume(:,:,:,9),volume(:,:,:,10), &
      volume(:,:,:,11),volume(:,:,:,12),volume(:,:,:,13),horizontal(:,:,5), &
      volume(:,:,:,14),volume(:,:,:,15),volume(:,:,:,16),volume(:,:,:,17), &
      volume(:,:,:,18),volume(:,:,:,19),volume(:,:,:,20),.002,.003,.4,300.,.1, &
      vertical(:,1),vertical(:,6),vertical(:,7),vertical(:,8),vertical(:,9), &
      .5,.3,.2,horizontal(:,:,6),horizontal(:,:,7),config,.false., &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,1,257,1,257,1,41)
  end subroutine invoke
end program acoustic_vertical_momentum_benchmark
