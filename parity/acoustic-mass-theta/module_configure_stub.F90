module module_configure
  implicit none
  type :: grid_config_rec_type
    logical :: periodic_x=.false.
    logical :: specified=.false.
    logical :: nested=.false.
  end type grid_config_rec_type
end module module_configure
