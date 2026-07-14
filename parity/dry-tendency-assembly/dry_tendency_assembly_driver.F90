program dry_tendency_assembly_driver
  use iso_fortran_env, only: int32
  implicit none

  call run_case('first', 1, 5, 1, 5, 1, 4, 1, .false.)
  call run_case('later', 2, 3, 2, 3, 1, 2, 2, .false.)
  call run_case('exceptional', 1, 5, 1, 5, 1, 4, 1, .true.)

contains

  subroutine run_case(case_name, its, ite, jts, jte, kts, kte, rk_step, exceptional)
    character(len=*), intent(in) :: case_name
    integer, intent(in) :: its, ite, jts, jte, kts, kte, rk_step
    logical, intent(in) :: exceptional
    integer, parameter :: ims = 0, ime = 5, jms = 0, jme = 5, kms = 0, kme = 4
    integer, parameter :: ids = 1, ide = 5, jds = 1, jde = 5, kds = 1, kde = 4
    real :: ru_tend(ims:ime,kms:kme,jms:jme), rv_tend(ims:ime,kms:kme,jms:jme)
    real :: rw_tend(ims:ime,kms:kme,jms:jme), ph_tend(ims:ime,kms:kme,jms:jme)
    real :: t_tend(ims:ime,kms:kme,jms:jme), ru_tendf(ims:ime,kms:kme,jms:jme)
    real :: rv_tendf(ims:ime,kms:kme,jms:jme), rw_tendf(ims:ime,kms:kme,jms:jme)
    real :: ph_tendf(ims:ime,kms:kme,jms:jme), t_tendf(ims:ime,kms:kme,jms:jme)
    real :: u_save(ims:ime,kms:kme,jms:jme), v_save(ims:ime,kms:kme,jms:jme)
    real :: w_save(ims:ime,kms:kme,jms:jme), ph_save(ims:ime,kms:kme,jms:jme)
    real :: t_save(ims:ime,kms:kme,jms:jme), h_diabatic(ims:ime,kms:kme,jms:jme)
    real :: mu_tend(ims:ime,jms:jme), mu_tendf(ims:ime,jms:jme), mut(ims:ime,jms:jme)
    real :: msftx(ims:ime,jms:jme), msfty(ims:ime,jms:jme)
    real :: msfux(ims:ime,jms:jme), msfuy(ims:ime,jms:jme)
    real :: msfvx(ims:ime,jms:jme), msfvx_inv(ims:ime,jms:jme), msfvy(ims:ime,jms:jme)
    real :: c1(kms:kme), c2(kms:kme)
    integer :: i, j, k

    do k = kms, kme
      c1(k) = 0.2 + real(k) * 0.03
      c2(k) = 0.4 - real(k) * 0.02
    end do
    do j = jms, jme
      do i = ims, ime
        mu_tend(i,j) = 0.6 + real(i) * 0.07 - real(j) * 0.03
        mu_tendf(i,j) = -0.2 + real(i) * 0.02 + real(j) * 0.04
        mut(i,j) = 50.0 + real(i) * 2.0 + real(j) * 3.0
        msftx(i,j) = 9.0
        msfty(i,j) = 1.1 + real(i) * 0.01 + real(j) * 0.02
        msfux(i,j) = 8.0
        msfuy(i,j) = 1.0 + real(i) * 0.02 + real(j) * 0.01
        msfvx(i,j) = 0.9 + real(i) * 0.015 - real(j) * 0.005
        msfvx_inv(i,j) = 1.0 / msfvx(i,j)
        msfvy(i,j) = 7.0
      end do
      do k = kms, kme
        do i = ims, ime
          ru_tend(i,k,j) = 1.0 + real(i) * 0.11 + real(k) * 0.07 - real(j) * 0.03
          rv_tend(i,k,j) = 2.0 - real(i) * 0.05 + real(k) * 0.09 + real(j) * 0.02
          rw_tend(i,k,j) = -1.0 + real(i) * 0.04 - real(k) * 0.08 + real(j) * 0.06
          ph_tend(i,k,j) = 3.0 + real(i) * 0.03 + real(k) * 0.05 - real(j) * 0.04
          t_tend(i,k,j) = -2.0 + real(i) * 0.02 + real(k) * 0.06 + real(j) * 0.01
          ru_tendf(i,k,j) = 0.3 + real(i) * 0.013 - real(k) * 0.017 + real(j) * 0.019
          rv_tendf(i,k,j) = -0.4 + real(i) * 0.021 + real(k) * 0.015 - real(j) * 0.011
          rw_tendf(i,k,j) = 0.5 - real(i) * 0.014 + real(k) * 0.012 + real(j) * 0.016
          ph_tendf(i,k,j) = -0.6 + real(i) * 0.018 - real(k) * 0.013 + real(j) * 0.009
          t_tendf(i,k,j) = 0.7 - real(i) * 0.012 + real(k) * 0.014 - real(j) * 0.008
          u_save(i,k,j) = 0.09 + real(i) * 0.003 + real(k) * 0.002 - real(j) * 0.001
          v_save(i,k,j) = -0.08 + real(i) * 0.002 - real(k) * 0.003 + real(j) * 0.001
          w_save(i,k,j) = 0.07 - real(i) * 0.001 + real(k) * 0.002 + real(j) * 0.003
          ph_save(i,k,j) = -0.06 + real(i) * 0.004 - real(k) * 0.001 + real(j) * 0.002
          t_save(i,k,j) = 0.05 + real(i) * 0.002 + real(k) * 0.003 - real(j) * 0.004
          h_diabatic(i,k,j) = 0.001 + real(i) * 0.0001 + real(k) * 0.0002 + real(j) * 0.0003
        end do
      end do
    end do

    if (exceptional) then
      msfty(1,1) = 0.0
      msfuy(2,1) = -0.0
      msfvx_inv(1,2) = huge(msfvx_inv) * 2.0
      h_diabatic(2,1,2) = huge(h_diabatic) * 2.0
      ru_tendf(2,1,1) = -0.0
      ph_tendf(1,1,1) = huge(ph_tendf)
    end if

    call rk_addtend_dry(ru_tend,rv_tend,rw_tend,ph_tend,t_tend, &
      ru_tendf,rv_tendf,rw_tendf,ph_tendf,t_tendf, &
      u_save,v_save,w_save,ph_save,t_save,mu_tend,mu_tendf,rk_step,c1,c2, &
      h_diabatic,mut,msftx,msfty,msfux,msfuy,msfvx,msfvx_inv,msfvy, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      ids,ide,jds,jde,kds,kde,its,ite,jts,jte,kts,kte)

    call write_volume(case_name,'ru_tend',ru_tend)
    call write_volume(case_name,'rv_tend',rv_tend)
    call write_volume(case_name,'rw_tend',rw_tend)
    call write_volume(case_name,'ph_tend',ph_tend)
    call write_volume(case_name,'t_tend',t_tend)
    call write_volume(case_name,'ru_tendf',ru_tendf)
    call write_volume(case_name,'rv_tendf',rv_tendf)
    call write_volume(case_name,'rw_tendf',rw_tendf)
    call write_volume(case_name,'ph_tendf',ph_tendf)
    call write_volume(case_name,'t_tendf',t_tendf)
    call write_horizontal(case_name,'mu_tend',mu_tend)
    call write_horizontal(case_name,'mu_tendf',mu_tendf)
  end subroutine run_case

  subroutine write_volume(case_name, field_name, field)
    character(len=*), intent(in) :: case_name, field_name
    real, intent(in) :: field(0:5,0:4,0:5)
    integer :: i, j, k
    do j = 0, 5
      do k = 0, 4
        do i = 0, 5
          if (isnan(field(i,k,j))) then
            write (*,'(A,1X,A,1X,I0,1X,I0,1X,I0,1X,A)') case_name,field_name,i,k,j,'NAN'
          else
            write (*,'(A,1X,A,1X,I0,1X,I0,1X,I0,1X,Z8.8)') case_name,field_name,i,k,j,transfer(field(i,k,j),0_int32)
          end if
        end do
      end do
    end do
  end subroutine write_volume

  subroutine write_horizontal(case_name, field_name, field)
    character(len=*), intent(in) :: case_name, field_name
    real, intent(in) :: field(0:5,0:5)
    integer :: i, j
    do j = 0, 5
      do i = 0, 5
        if (isnan(field(i,j))) then
          write (*,'(A,1X,A,1X,I0,1X,I0,1X,A)') case_name,field_name,i,j,'NAN'
        else
          write (*,'(A,1X,A,1X,I0,1X,I0,1X,Z8.8)') case_name,field_name,i,j,transfer(field(i,j),0_int32)
        end if
      end do
    end do
  end subroutine write_horizontal

end program dry_tendency_assembly_driver
