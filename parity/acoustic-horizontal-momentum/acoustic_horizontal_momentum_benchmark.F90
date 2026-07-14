program acoustic_horizontal_momentum_benchmark
  use module_configure, only: grid_config_rec_type
  use extracted_acoustic_horizontal_momentum, only: advance_uv
  implicit none
  integer,parameter::ims=0,ime=257,jms=0,jme=257,kms=0,kme=41
  integer,parameter::ids=1,ide=257,jds=1,jde=257,kds=1,kde=41
  integer,parameter::calls_per_sample=3
  real,allocatable::u(:,:,:),v(:,:,:),volume(:,:,:,:),horizontal(:,:,:),vertical(:,:)
  type(grid_config_rec_type)::config
  integer::sample,iteration
  integer(kind=8)::started,finished,rate
  allocate(u(ims:ime,kms:kme,jms:jme),v(ims:ime,kms:kme,jms:jme))
  allocate(volume(ims:ime,kms:kme,jms:jme,10))
  allocate(horizontal(ims:ime,jms:jme,9),vertical(kms:kme,5))
  u=1.;v=1.;volume=1.;horizontal=1.;vertical=1.
  volume(:,:,:,1)=.01;volume(:,:,:,2)=.02;volume(:,:,:,3)=3.;volume(:,:,:,4)=4.
  volume(:,:,:,5)=5.;volume(:,:,:,6)=2.;volume(:,:,:,7)=.8;volume(:,:,:,8)=.1
  volume(:,:,:,9)=.95;volume(:,:,:,10)=.96
  horizontal(:,:,1)=1.;horizontal(:,:,2)=2.;horizontal(:,:,3)=2.;horizontal(:,:,4)=.5
  vertical(:,1)=.5;vertical(:,2)=.25;vertical(:,3)=.6;vertical(:,4)=.4;vertical(:,5)=1.2
  do iteration=1,3
    call invoke
  enddo
  call system_clock(count_rate=rate)
  do sample=1,31
    call system_clock(started)
    do iteration=1,calls_per_sample
      call invoke
    enddo
    call system_clock(finished)
    write(*,'(F12.6)')real(finished-started)/real(rate)*1000./real(calls_per_sample)
  enddo
contains
  subroutine invoke
    call advance_uv(u,volume(:,:,:,1),v,volume(:,:,:,2), &
      volume(:,:,:,3),volume(:,:,:,4),volume(:,:,:,5),volume(:,:,:,6), &
      volume(:,:,:,7),volume(:,:,:,8),horizontal(:,:,1),horizontal(:,:,2), &
      volume(:,:,:,9),horizontal(:,:,3),volume(:,:,:,10),horizontal(:,:,4), &
      vertical(:,1),vertical(:,2),vertical(:,1),vertical(:,2),vertical(:,1), &
      vertical(:,2),vertical(:,1),vertical(:,2),horizontal(:,:,5),horizontal(:,:,6), &
      horizontal(:,:,7),horizontal(:,:,8),horizontal(:,:,9),.001,.001,.5,.7,.2,.1, &
      vertical(:,3),vertical(:,4),.1,vertical(:,5),config,0,.true.,.false., &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,1,257,1,257,1,41)
  end subroutine
end program
