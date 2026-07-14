program flow_dependent_inflow_policies_benchmark
  use module_configure, only: grid_config_rec_type
  use extracted_flow_dependent_inflow_policies, only: flow_dep_bdy_qnn, flow_dep_bdy_fixed_inflow
  implicit none
  integer,parameter::ims=0,ime=257,jms=0,jme=257,kms=0,kme=41
  integer,parameter::ids=1,ide=257,jds=1,jde=257,kds=1,kde=41
  integer,parameter::calls_per_sample=100
  real,allocatable::field(:,:,:),u(:,:,:),v(:,:,:)
  type(grid_config_rec_type)::config
  integer::i,j,k,index,sample,iteration
  integer(kind=8)::started,finished,rate
  allocate(field(ims:ime,kms:kme,jms:jme))
  allocate(u(ims:ime,kms:kme,jms:jme),v(ims:ime,kms:kme,jms:jme))
  do j=jms,jme;do k=kms,kme;do i=ims,ime
    index=i+(ime-ims+1)*(k+(kme-kms+1)*j)
    field(i,k,j)=real(index)*.000001-30.
    if (mod(index,2)==0) then;u(i,k,j)=-1.;else;u(i,k,j)=1.;endif
    if (mod(index,3)==0) then;v(i,k,j)=-1.;else;v(i,k,j)=1.;endif
  enddo;enddo;enddo
  config%periodic_x=.false.
  call invoke_qnn
  call measure_qnn
  call initialize_field
  call invoke_fixed
  call measure_fixed
contains
  subroutine initialize_field
    do j=jms,jme;do k=kms,kme;do i=ims,ime
      index=i+(ime-ims+1)*(k+(kme-kms+1)*j)
      field(i,k,j)=real(index)*.000001-30.
    enddo;enddo;enddo
  end subroutine

  subroutine measure_qnn
    call system_clock(count_rate=rate)
    do sample=1,31
      call system_clock(started)
      do iteration=1,calls_per_sample
        call invoke_qnn
      enddo
      call system_clock(finished)
      write(*,'(A,1X,F12.6)')'constant',real(finished-started)/real(rate)*1000./real(calls_per_sample)
    enddo
    write(*,'(A,1X,A,1X,ES16.8)')'constant','checksum',sum(field)
  end subroutine

  subroutine measure_fixed
    call system_clock(count_rate=rate)
    do sample=1,31
      call system_clock(started)
      do iteration=1,calls_per_sample
        call invoke_fixed
      enddo
      call system_clock(finished)
      write(*,'(A,1X,F12.6)')'preserve',real(finished-started)/real(rate)*1000./real(calls_per_sample)
    enddo
    write(*,'(A,1X,A,1X,ES16.8)')'preserve','checksum',sum(field)
  end subroutine

  subroutine invoke_qnn
    call flow_dep_bdy_qnn(field,u,v,config,5,73.5, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
  end subroutine

  subroutine invoke_fixed
    call flow_dep_bdy_fixed_inflow(field,u,v,config,5, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,ids,ide,jds,jde,kds,kde)
  end subroutine
end program flow_dependent_inflow_policies_benchmark
