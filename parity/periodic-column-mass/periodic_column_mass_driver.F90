program periodic_column_mass_driver
  use extracted_big_step_column_mass, only: calc_mu_uv, calc_mu_uv_1
  use iso_fortran_env, only: int32
  use module_configure, only: grid_config_rec_type
  implicit none

  call run_all_split_cases()
  call run_all_full_cases()

contains

  subroutine run_all_split_cases()
    call run_split_case('interior', 0, 5, 0, 4, 1, 4, 1, 3, .false., .false., .false.)
    call run_split_case('lower', 1, 5, 1, 4, 1, 4, 1, 3, .false., .false., .false.)
    call run_split_case('upper', 0, 4, 0, 3, 1, 4, 1, 3, .false., .false., .false.)
    call run_split_case('both', 1, 4, 1, 3, 1, 4, 1, 3, .false., .false., .false.)
    call run_split_case('periodic_x', 1, 4, 1, 3, 1, 4, 1, 3, .true., .false., .false.)
    call run_split_case('periodic_y', 1, 4, 1, 3, 1, 4, 1, 3, .false., .true., .false.)
    call run_split_case('periodic_xy', 1, 4, 1, 3, 1, 4, 1, 3, .true., .true., .false.)
    call run_split_case('physical_expression', 1, 4, 1, 3, 1, 4, 1, 3, &
                        .false., .false., .true.)
  end subroutine run_all_split_cases

  subroutine run_all_full_cases()
    call run_full_case('interior', 0, 5, 0, 4, 1, 4, 1, 3, .false., .false., .false.)
    call run_full_case('lower', 1, 5, 1, 4, 1, 4, 1, 3, .false., .false., .false.)
    call run_full_case('upper', 0, 4, 0, 3, 1, 4, 1, 3, .false., .false., .false.)
    call run_full_case('both', 1, 4, 1, 3, 1, 4, 1, 3, .false., .false., .false.)
    call run_full_case('periodic_x', 1, 4, 1, 3, 1, 4, 1, 3, .true., .false., .false.)
    call run_full_case('periodic_y', 1, 4, 1, 3, 1, 4, 1, 3, .false., .true., .false.)
    call run_full_case('periodic_xy', 1, 4, 1, 3, 1, 4, 1, 3, .true., .true., .false.)
    call run_full_case('physical_expression', 1, 4, 1, 3, 1, 4, 1, 3, &
                       .false., .false., .true.)
  end subroutine run_all_full_cases

  subroutine run_split_case(case_name, ids, ide, jds, jde, its, ite, jts, jte, &
                            periodic_x, periodic_y, use_extreme)
    character(len=*), intent(in) :: case_name
    integer, intent(in) :: ids, ide, jds, jde, its, ite, jts, jte
    logical, intent(in) :: periodic_x, periodic_y, use_extreme
    integer, parameter :: ims = 0, ime = 5, jms = 0, jme = 4
    integer, parameter :: kds = 1, kde = 2, kms = 1, kme = 1, kts = 1, kte = 1
    type(grid_config_rec_type) :: config_flags
    real :: mu(ims:ime, jms:jme), mub(ims:ime, jms:jme)
    real :: muu(ims:ime, jms:jme), muv(ims:ime, jms:jme)
    integer :: i, j

    config_flags%periodic_x = periodic_x
    config_flags%periodic_y = periodic_y
    call initialize_split_mass(mu, mub)
    if (use_extreme) then
      mu(ids, jds) = huge(mu)
      mub(ids, jds) = 0.0
    end if
    muu = -999.0
    muv = -999.0
    call calc_mu_uv(config_flags, mu, mub, muu, muv, ids, ide, jds, jde, kds, kde, &
                    ims, ime, jms, jme, kms, kme, its, ite, jts, jte, kts, kte)
    call write_outputs(case_name, 'split', muu, muv)
  end subroutine run_split_case

  subroutine run_full_case(case_name, ids, ide, jds, jde, its, ite, jts, jte, &
                           periodic_x, periodic_y, use_extreme)
    character(len=*), intent(in) :: case_name
    integer, intent(in) :: ids, ide, jds, jde, its, ite, jts, jte
    logical, intent(in) :: periodic_x, periodic_y, use_extreme
    integer, parameter :: ims = 0, ime = 5, jms = 0, jme = 4
    integer, parameter :: kds = 1, kde = 2, kms = 1, kme = 1, kts = 1, kte = 1
    type(grid_config_rec_type) :: config_flags
    real :: mu(ims:ime, jms:jme), mub(ims:ime, jms:jme)
    real :: full_mass(ims:ime, jms:jme)
    real :: muu(ims:ime, jms:jme), muv(ims:ime, jms:jme)

    config_flags%periodic_x = periodic_x
    config_flags%periodic_y = periodic_y
    call initialize_split_mass(mu, mub)
    full_mass = mu + mub
    if (use_extreme) full_mass(ids, jds) = huge(full_mass)
    muu = -999.0
    muv = -999.0
    call calc_mu_uv_1(config_flags, full_mass, muu, muv, ids, ide, jds, jde, kds, kde, &
                      ims, ime, jms, jme, kms, kme, its, ite, jts, jte, kts, kte)
    call write_outputs(case_name, 'full', muu, muv)
  end subroutine run_full_case

  subroutine initialize_split_mass(mu, mub)
    real, intent(out) :: mu(0:5, 0:4), mub(0:5, 0:4)
    integer :: i, j

    do j = 0, 4
      do i = 0, 5
        mu(i, j) = real(i) * 0.25 + real(j) * 1.5 - 0.3
        mub(i, j) = 100.0 + real(i) * 0.5 - real(j) * 0.75
      end do
    end do
  end subroutine initialize_split_mass

  subroutine write_outputs(case_name, routine_name, muu, muv)
    character(len=*), intent(in) :: case_name, routine_name
    real, intent(in) :: muu(0:5, 0:4), muv(0:5, 0:4)
    integer :: i, j

    do j = 0, 4
      do i = 0, 5
        write (*, '(A,1X,A,1X,A,1X,I0,1X,I0,1X,Z8.8)') case_name, routine_name, &
          'west_east', i, j, transfer(muu(i, j), 0_int32)
      end do
    end do
    do j = 0, 4
      do i = 0, 5
        write (*, '(A,1X,A,1X,A,1X,I0,1X,I0,1X,Z8.8)') case_name, routine_name, &
          'south_north', i, j, transfer(muv(i, j), 0_int32)
      end do
    end do
  end subroutine write_outputs
end program periodic_column_mass_driver
