module module_configure
  implicit none
  type :: grid_config_rec_type
    logical :: periodic_x=.false.,periodic_y=.false.
    logical :: specified=.false.,nested=.false.,polar=.false.
    logical :: open_xs=.false.,open_xe=.false.,open_ys=.false.,open_ye=.false.
    logical :: symmetric_xs=.false.,symmetric_xe=.false.
    logical :: symmetric_ys=.false.,symmetric_ye=.false.
  end type grid_config_rec_type
end module module_configure
