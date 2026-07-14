module module_configure
  implicit none
  type :: grid_config_rec_type
    logical :: nested=.false.,specified=.false.,periodic_x=.false.
    logical :: open_xs=.false.,open_xe=.false.,symmetric_xs=.false.,symmetric_xe=.false.
    logical :: open_ys=.false.,open_ye=.false.,symmetric_ys=.false.,symmetric_ye=.false.
    logical :: polar=.false.
  end type grid_config_rec_type
end module module_configure
