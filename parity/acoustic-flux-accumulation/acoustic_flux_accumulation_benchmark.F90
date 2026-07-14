program acoustic_flux_accumulation_benchmark
  use extracted_acoustic_flux_accumulation, only: sumflux
  implicit none
  integer,parameter::ims=0,ime=257,jms=0,jme=257,kms=0,kme=41
  integer,parameter::ids=1,ide=257,jds=1,jde=257,kds=1,kde=41
  integer,parameter::sequences_per_sample=4
  real,allocatable::current(:,:,:,:),linear(:,:,:,:),averages(:,:,:,:)
  real,allocatable::horizontal(:,:,:),vertical(:,:)
  integer::sample,sequence,iteration
  integer(kind=8)::started,finished,rate
  allocate(current(ims:ime,kms:kme,jms:jme,3),linear(ims:ime,kms:kme,jms:jme,3))
  allocate(averages(ims:ime,kms:kme,jms:jme,3),horizontal(ims:ime,jms:jme,4))
  allocate(vertical(kms:kme,2))
  current(:,:,:,1)=.2;current(:,:,:,2)=.3;current(:,:,:,3)=.4
  linear(:,:,:,1)=.15;linear(:,:,:,2)=.25;linear(:,:,:,3)=.35
  averages=-900.;horizontal(:,:,1)=11.;horizontal(:,:,2)=12.
  horizontal(:,:,3)=1.03;horizontal(:,:,4)=.97
  vertical(:,1)=.45;vertical(:,2)=.2
  call invoke_sequence
  call system_clock(count_rate=rate)
  do sample=1,31
    call system_clock(started)
    do sequence=1,sequences_per_sample
      call invoke_sequence
    enddo
    call system_clock(finished)
    write(*,'(F12.6)')real(finished-started)/real(rate)*1000./real(sequences_per_sample)
  enddo
contains
  subroutine invoke_sequence
    do iteration=1,3
      call sumflux(current(:,:,:,1),current(:,:,:,2),current(:,:,:,3), &
        linear(:,:,:,1),linear(:,:,:,2),linear(:,:,:,3), &
        horizontal(:,:,1),horizontal(:,:,2),vertical(:,1),vertical(:,2), &
        vertical(:,1),vertical(:,2),vertical(:,1),vertical(:,2),vertical(:,1),vertical(:,2), &
        averages(:,:,:,1),averages(:,:,:,2),averages(:,:,:,3),.1, &
        horizontal(:,:,3),horizontal(:,:,3),horizontal(:,:,3),horizontal(:,:,4),horizontal(:,:,4), &
        iteration,3,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,1,257,1,257,1,41)
    enddo
  end subroutine
end program
