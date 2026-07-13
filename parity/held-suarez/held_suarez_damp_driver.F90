program held_suarez_damp_driver
  use iso_fortran_env, only: int32
  use module_damping_em, only: held_suarez_damp
  implicit none

  real :: ru_tend(-1:4, 0:3, -1:4), rv_tend(-1:4, 0:3, -1:4)
  real :: ru(-1:4, 0:3, -1:4), rv(-1:4, 0:3, -1:4)
  real :: p(-1:4, 0:3, -1:4), pb(-1:4, 0:3, -1:4)
  real :: selected(16)
  integer(int32) :: bits(size(selected))
  integer :: i, j, k

  do j = -1, 4
    do k = 0, 3
      do i = -1, 4
        p(i, k, j) = real(10*i + 3*j + 2*k)
        select case (k)
        case (0)
          pb(i, k, j) = 110000.0
        case (1)
          pb(i, k, j) = 100000.0
        case (2)
          pb(i, k, j) = 80000.0
        case default
          pb(i, k, j) = 50000.0
        end select
        ru(i, k, j) = real(2*i + 3*k + 5*j)
        rv(i, k, j) = real(-i + 4*k + 2*j)
        ru_tend(i, k, j) = real(100 + i + 2*k + 3*j)
        rv_tend(i, k, j) = real(200 + 2*i + k + 4*j)
      end do
    end do
  end do

  call held_suarez_damp( &
      ru_tend, rv_tend, ru, rv, p, pb, &
      0, 4, 0, 4, 1, 4, &
      -1, 4, -1, 4, 0, 3, &
      0, 3, 0, 4, 1, 4)

  selected(1) = ru_tend(0, 1, 0)
  selected(2) = ru_tend(0, 2, 0)
  selected(3) = ru_tend(3, 3, 3)
  selected(4) = ru_tend(4, 1, 0)
  selected(5) = ru_tend(0, 0, 0)
  selected(6) = ru_tend(0, 1, 4)
  selected(7) = ru_tend(2, 2, 2)
  selected(8) = ru_tend(3, 1, 3)
  selected(9) = rv_tend(0, 1, 1)
  selected(10) = rv_tend(0, 2, 1)
  selected(11) = rv_tend(3, 3, 3)
  selected(12) = rv_tend(0, 1, 0)
  selected(13) = rv_tend(0, 1, 4)
  selected(14) = rv_tend(4, 1, 1)
  selected(15) = rv_tend(0, 0, 1)
  selected(16) = rv_tend(2, 2, 2)
  bits = transfer(selected, bits)
  write (*, '(A,*(1X,Z8.8))') 'held_suarez_boundaries', bits
end program held_suarez_damp_driver
