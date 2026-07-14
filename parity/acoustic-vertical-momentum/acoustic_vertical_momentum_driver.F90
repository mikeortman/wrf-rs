program acoustic_vertical_momentum_driver
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  use extracted_acoustic_vertical_momentum, only: advance_w
  implicit none

  call run_case('global_gradient', .false., .false., 2, .false., .false., 1, 5, 1, 5)
  call run_case('nested_product_rigid_damped', .true., .false., 1, .true., .true., 1, 5, 1, 5)
  call run_case('nested_periodic_gradient_damped', .true., .true., 2, .false., .true., 1, 5, 1, 5)
  call run_case('partial_product_rigid', .false., .false., 1, .true., .false., 2, 4, 2, 4)
contains
  subroutine run_case(name, nested, periodic_x, phi_adv_z, top_lid, damping, its, ite, jts, jte)
    character(len=*), intent(in) :: name
    logical, intent(in) :: nested, periodic_x, top_lid, damping
    integer, intent(in) :: phi_adv_z, its, ite, jts, jte
    integer, parameter :: ims=0, ime=5, jms=0, jme=5, kms=0, kme=5
    integer, parameter :: ids=1, ide=5, jds=1, jde=5, kds=1, kde=5
    real :: w(ims:ime,kms:kme,jms:jme), rw_tend(ims:ime,kms:kme,jms:jme)
    real :: ww(ims:ime,kms:kme,jms:jme), w_save(ims:ime,kms:kme,jms:jme)
    real :: u(ims:ime,kms:kme,jms:jme), v(ims:ime,kms:kme,jms:jme)
    real :: t_2ave(ims:ime,kms:kme,jms:jme), t_2(ims:ime,kms:kme,jms:jme)
    real :: t_1(ims:ime,kms:kme,jms:jme), ph(ims:ime,kms:kme,jms:jme)
    real :: ph_1(ims:ime,kms:kme,jms:jme), phb(ims:ime,kms:kme,jms:jme)
    real :: ph_tend(ims:ime,kms:kme,jms:jme), c2a(ims:ime,kms:kme,jms:jme)
    real :: cqw(ims:ime,kms:kme,jms:jme), alt(ims:ime,kms:kme,jms:jme)
    real :: unused_volume(ims:ime,kms:kme,jms:jme)
    real :: lower(ims:ime,kms:kme,jms:jme), inverse_diagonal(ims:ime,kms:kme,jms:jme)
    real :: upper(ims:ime,kms:kme,jms:jme)
    real :: unused_mass(ims:ime,jms:jme), mut(ims:ime,jms:jme)
    real :: muave(ims:ime,jms:jme), muts(ims:ime,jms:jme), ht(ims:ime,jms:jme)
    real :: msftx(ims:ime,jms:jme), msfty(ims:ime,jms:jme)
    real :: c1h(kms:kme), c2h(kms:kme), c1f(kms:kme), c2f(kms:kme)
    real :: fnm(kms:kme), fnp(kms:kme), rdnw(kms:kme), rdn(kms:kme)
    real :: unused_coefficient(kms:kme)
    type(grid_config_rec_type) :: config
    integer :: i, j, k

    config%nested = nested
    config%periodic_x = periodic_x
    config%phi_adv_z = phi_adv_z
    if (damping) then
      config%damp_opt = 3
    else
      config%damp_opt = 0
    endif
    config%dampcoef = 0.15
    config%zdamp = 220.0

    do k=kms,kme
      c1h(k)=0.42+real(k)*0.011
      c2h(k)=0.19-real(k)*0.004
      c1f(k)=0.37+real(k)*0.009
      c2f(k)=0.23-real(k)*0.003
      fnm(k)=0.58+real(k)*0.006
      fnp(k)=0.42-real(k)*0.006
      rdnw(k)=1.05+real(k)*0.025
      rdn(k)=0.91+real(k)*0.018
      unused_coefficient(k)=7.0
    enddo
    do j=jms,jme
      do i=ims,ime
        unused_mass(i,j)=-950.0
        mut(i,j)=11.0+real(i)*0.13-real(j)*0.09
        muave(i,j)=2.1+real(i)*0.07+real(j)*0.04
        muts(i,j)=12.7-real(i)*0.03+real(j)*0.08
        ht(i,j)=140.0+real(i)*9.0+real(j)*13.0+real(i*j)*0.7
        msftx(i,j)=1.03+real(i)*0.003-real(j)*0.002
        msfty(i,j)=0.97-real(i)*0.002+real(j)*0.004
      enddo
      do k=kms,kme
        do i=ims,ime
          w(i,k,j)=0.8+real(i)*0.02+real(k)*0.03-real(j)*0.01
          rw_tend(i,k,j)=0.012+real(i)*0.0003-real(k)*0.0002+real(j)*0.0001
          ww(i,k,j)=0.35-real(i)*0.004+real(k)*0.006+real(j)*0.003
          w_save(i,k,j)=0.31+real(i)*0.005-real(k)*0.004+real(j)*0.002
          u(i,k,j)=0.2+real(i)*0.013+real(k)*0.017+real(j)*0.019
          v(i,k,j)=0.3-real(i)*0.009+real(k)*0.014+real(j)*0.021
          t_2ave(i,k,j)=294.0+real(i)*0.2+real(k)*0.7-real(j)*0.15
          t_2(i,k,j)=300.0+real(i)*0.7+real(k)*1.1-real(j)*0.4
          t_1(i,k,j)=1.3-real(i)*0.03+real(k)*0.09+real(j)*0.05
          ph(i,k,j)=20.0+real(i)*0.8+real(k)*2.1-real(j)*0.6
          ph_1(i,k,j)=18.0-real(i)*0.4+real(k)*1.7+real(j)*0.3
          phb(i,k,j)=500.0+real(k)*1000.0+real(i)*3.0+real(j)*4.0
          ph_tend(i,k,j)=0.05+real(i)*0.002-real(k)*0.001+real(j)*0.003
          c2a(i,k,j)=1.1+real(i)*0.004+real(k)*0.008-real(j)*0.003
          cqw(i,k,j)=0.82-real(i)*0.002+real(k)*0.005+real(j)*0.001
          alt(i,k,j)=0.9+real(i)*0.006-real(k)*0.004+real(j)*0.002
          lower(i,k,j)=-0.03+real(k)*0.0007-real(i)*0.0002
          inverse_diagonal(i,k,j)=0.83-real(k)*0.002+real(j)*0.0003
          upper(i,k,j)=-0.02+real(k)*0.0004+real(i)*0.0001
          unused_volume(i,k,j)=-951.0
        enddo
      enddo
    enddo

    call advance_w(w,rw_tend,ww,w_save,u,v,unused_mass,mut,muave,muts, &
      c1h,c2h,c1f,c2f,unused_coefficient,unused_coefficient,unused_coefficient,unused_coefficient, &
      t_2ave,t_2,t_1,ph,ph_1,phb,ph_tend,ht,c2a,cqw,alt,unused_volume, &
      lower,inverse_diagonal,upper,0.002,0.003,0.4,300.0,0.1, &
      unused_coefficient,fnm,fnp,rdnw,rdn,0.5,0.3,0.2,msftx,msfty, &
      config,top_lid,ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme,its,ite,jts,jte,1,5)
    call write_volume(name,'w',w)
    call write_volume(name,'ph',ph)
    call write_volume(name,'tave',t_2ave)
  end subroutine run_case

  subroutine write_volume(case_name,field_name,field)
    character(len=*),intent(in)::case_name,field_name
    real,intent(in)::field(0:5,0:5,0:5)
    integer::i,j,k
    do j=0,5;do k=0,5;do i=0,5
      write(*,'(A,1X,A,3(1X,I0),1X,Z8.8)')case_name,field_name,i,k,j,transfer(field(i,k,j),0_int32)
    enddo;enddo;enddo
  end subroutine write_volume
end program acoustic_vertical_momentum_driver
