program positive_definite_slab_driver
  use iso_fortran_env, only: int32
  use module_positive_definite, only: positive_definite_slab
  implicit none

  real :: field(-1:4, -1:2, 0:3)
  real :: selected(20)
  integer(int32) :: bits(size(selected))

  field = 8.0
  field(0:3, 0, 1) = [-1.0, 1.0, 2.0, 4.0]
  field(0:3, 1, 1) = [-1.0, -2.0, 0.0, 0.0]
  field(0:3, 0, 2) = [1.0, 2.0, 3.0, 4.0]
  field(0:3, 1, 2) = [-1.0, -1.0, -1.0, 4.0]

  call positive_definite_slab( &
      field, &
      0, 4, 1, 3, 0, 3, &
      -1, 4, 0, 3, -1, 2, &
      0, 3, 1, 3, 0, 2)

  selected(1:4) = field(0:3, 0, 1)
  selected(5:8) = field(0:3, 1, 1)
  selected(9:12) = field(0:3, 0, 2)
  selected(13:16) = field(0:3, 1, 2)
  selected(17) = field(-1, -1, 0)
  selected(18) = field(4, 0, 1)
  selected(19) = field(0, 2, 1)
  selected(20) = field(0, 0, 3)
  bits = transfer(selected, bits)
  write (*, '(A,*(1X,Z8.8))') 'slab_boundaries', bits
end program positive_definite_slab_driver
