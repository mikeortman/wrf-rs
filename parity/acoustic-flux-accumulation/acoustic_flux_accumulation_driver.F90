program acoustic_flux_accumulation_driver
  use iso_fortran_env, only: int32
  use extracted_acoustic_flux_accumulation, only: sumflux
  implicit none
  integer, parameter :: ims=0, ime=4, jms=0, jme=4, kms=0, kme=4
  integer, parameter :: ids=1, ide=4, jds=1, jde=4, kds=1, kde=4
  real :: ru(ims:ime,kms:kme,jms:jme), rv(ims:ime,kms:kme,jms:jme)
  real :: ww(ims:ime,kms:kme,jms:jme), u_lin(ims:ime,kms:kme,jms:jme)
  real :: v_lin(ims:ime,kms:kme,jms:jme), ww_lin(ims:ime,kms:kme,jms:jme)
  real :: ru_m(ims:ime,kms:kme,jms:jme), rv_m(ims:ime,kms:kme,jms:jme)
  real :: ww_m(ims:ime,kms:kme,jms:jme)
  real :: muu(ims:ime,jms:jme), muv(ims:ime,jms:jme)
  real :: msfuy(ims:ime,jms:jme), msfvx_inv(ims:ime,jms:jme)
  real :: unused_map(ims:ime,jms:jme)
  real :: c1h(kms:kme), c2h(kms:kme), unused_coefficient(kms:kme)
  integer :: i, j, k, iteration

  do k=kms,kme
    c1h(k)=0.45+real(k)*0.013
    c2h(k)=0.17-real(k)*0.006
    unused_coefficient(k)=-700.0
  enddo
  do j=jms,jme
    do i=ims,ime
      muu(i,j)=10.0+real(i)*0.2-real(j)*0.1
      muv(i,j)=11.0-real(i)*0.1+real(j)*0.15
      msfuy(i,j)=0.95+real(i)*0.01+real(j)*0.005
      msfvx_inv(i,j)=1.04-real(i)*0.004+real(j)*0.003
      unused_map(i,j)=-701.0
    enddo
    do k=kms,kme
      do i=ims,ime
        u_lin(i,k,j)=0.21+real(i)*0.011+real(k)*0.017-real(j)*0.009
        v_lin(i,k,j)=0.31-real(i)*0.007+real(k)*0.013+real(j)*0.015
        ww_lin(i,k,j)=0.41+real(i)*0.005-real(k)*0.003+real(j)*0.019
        ru_m(i,k,j)=-999.0
        rv_m(i,k,j)=-999.0
        ww_m(i,k,j)=-999.0
      enddo
    enddo
  enddo

  do iteration=1,3
    do j=jms,jme
      do k=kms,kme
        do i=ims,ime
          ru(i,k,j)=real(iteration)*0.1+real(i)*0.013+real(k)*0.007-real(j)*0.005
          rv(i,k,j)=real(iteration)*0.2-real(i)*0.009+real(k)*0.011+real(j)*0.004
          ww(i,k,j)=real(iteration)*0.3+real(i)*0.003-real(k)*0.008+real(j)*0.006
        enddo
      enddo
    enddo
    call sumflux(ru,rv,ww,u_lin,v_lin,ww_lin,muu,muv, &
      c1h,c2h,unused_coefficient,unused_coefficient, &
      unused_coefficient,unused_coefficient,unused_coefficient,unused_coefficient, &
      ru_m,rv_m,ww_m,0.1,unused_map,msfuy,unused_map,msfvx_inv,unused_map, &
      iteration,3,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      1,4,1,4,1,4)
  enddo

  call emit('ru_m',ru_m)
  call emit('rv_m',rv_m)
  call emit('ww_m',ww_m)
contains
  subroutine emit(name, field)
    character(len=*), intent(in) :: name
    real, intent(in) :: field(ims:ime,kms:kme,jms:jme)
    integer :: ii, jj, kk
    do jj=jms,jme
      do kk=kms,kme
        do ii=ims,ime
          write(*,'(a,1x,i0,1x,i0,1x,i0,1x,z8.8)') name,ii,kk,jj,transfer(field(ii,kk,jj),0_int32)
        enddo
      enddo
    enddo
  end subroutine emit
end program acoustic_flux_accumulation_driver
