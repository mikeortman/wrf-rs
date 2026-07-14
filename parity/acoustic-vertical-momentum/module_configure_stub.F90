module module_configure
  implicit none
  type grid_config_rec_type
    logical :: periodic_x = .false.
    logical :: specified = .false.
    logical :: nested = .false.
    integer :: phi_adv_z = 1
    integer :: damp_opt = 0
    real :: dampcoef = 0.0
    real :: zdamp = 1.0
  end type grid_config_rec_type
end module module_configure
