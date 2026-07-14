program physical_boundary_benchmark
  use iso_fortran_env,only:int64,real64
  use module_configure,only:grid_config_rec_type
  use extracted_physical_boundary,only:set_physical_bc3d
  implicit none
  integer,parameter::nx=256,ny=256,nz=40
  integer,parameter::ims=0,ime=nx+8,jms=0,jme=ny+8,kms=0,kme=nz+1
  integer,parameter::ids=4,ide=nx+4,jds=4,jde=ny+4,kds=1,kde=nz+1
  integer,parameter::samples=31,calls_per_sample=100,warmup_calls=100
  real,allocatable::field(:,:,:)
  type(grid_config_rec_type)::config
  integer(int64)::started,finished,rate
  integer::sample,iteration
  real(real64)::milliseconds,checksum

  allocate(field(ims:ime,kms:kme,jms:jme))
  field=1.2
  config=grid_config_rec_type()
  config%specified=.true.
  do iteration=1,warmup_calls;call invoke;enddo
  call system_clock(count_rate=rate)
  do sample=1,samples
    call system_clock(started)
    do iteration=1,calls_per_sample;call invoke;enddo
    call system_clock(finished)
    milliseconds=real(finished-started,real64)*1000._real64 &
      /real(rate,real64)/real(calls_per_sample,real64)
    write(*,'(A,I0,A,F12.6)')'sample_',sample,'_milliseconds_per_call ',milliseconds
  enddo
  checksum=sum(real(field,real64))
  write(*,'(A,ES24.16)')'checksum ',checksum

contains
  subroutine invoke
    call set_physical_bc3d(field,'p',config,ids,ide,jds,jde,kds,kde, &
      ims,ime,jms,jme,kms,kme,ids,ide,jds,jde,kds,kde, &
      ids,ide,jds,jde,kds,kde)
  end subroutine invoke
end program physical_boundary_benchmark
