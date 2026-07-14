module module_configure
  implicit none
  type :: grid_config_rec_type
    logical :: periodic_x=.false.,periodic_y=.false.,specified=.false.,nested=.false.
    logical :: open_xs=.false.,open_xe=.false.,symmetric_xs=.false.,symmetric_xe=.false.
    logical :: open_ys=.false.,open_ye=.false.,symmetric_ys=.false.,symmetric_ye=.false.
    logical :: polar=.false.
    integer :: phi_adv_z=2,damp_opt=0
    real :: dampcoef=0.0,zdamp=1.0
  end type grid_config_rec_type
end module module_configure
