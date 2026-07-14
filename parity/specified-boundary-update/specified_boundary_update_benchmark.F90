program specified_boundary_update_benchmark
  use module_configure, only: grid_config_rec_type
  use extracted_specified_boundary_update, only: spec_bdyupdate
  implicit none
  integer,parameter::ims=0,ime=257,jms=0,jme=257,kms=0,kme=41
  integer,parameter::ids=1,ide=257,jds=1,jde=257,kds=1,kde=41
  integer,parameter::calls_per_sample=100
  real,allocatable::field(:,:,:),tendency(:,:,:)
  type(grid_config_rec_type)::config
  integer::sample,iteration
  integer(kind=8)::started,finished,rate
  allocate(field(ims:ime,kms:kme,jms:jme),tendency(ims:ime,kms:kme,jms:jme))
  field=1.2;tendency=.1;config%periodic_x=.false.
  call invoke
  call system_clock(count_rate=rate)
  do sample=1,31
    call system_clock(started)
    do iteration=1,calls_per_sample
      call invoke
    enddo
    call system_clock(finished)
    write(*,'(F12.6)')real(finished-started)/real(rate)*1000./real(calls_per_sample)
  enddo
  write(*,'(A,1X,ES16.8)')'checksum',sum(field)
contains
  subroutine invoke
    call spec_bdyupdate(field,tendency,.25,'t',config,5, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
  end subroutine
end program specified_boundary_update_benchmark
