PROGRAM wrf_schema_order_driver
  IMPLICIT NONE

  INTEGER :: lengths(4)
  INTEGER :: dimension_count
  INTEGER :: status
  CHARACTER(LEN=80) :: names(4)
  CHARACTER(LEN=80) :: reordered_names(4)
  CHARACTER(LEN=3) :: memory_order

  lengths = 1
  lengths(1:3) = (/ 4, 2, 3 /)
  names = ''
  names(1) = 'west_east'
  names(2) = 'bottom_top'
  names(3) = 'south_north'
  CALL ExtOrder('XZY', lengths, status)
  CALL require_success('ExtOrder XZY', status)
  CALL ExtOrderStr('XZY', names, reordered_names, status)
  CALL require_success('ExtOrderStr XZY', status)
  CALL reorder('XZY', memory_order)
  CALL encode_blanks(memory_order)
  WRITE (*, '(&
    &"T|",A,"=",I0,"|",A,"=",I0,"|",A,"=",I0,"|MemoryOrder=",A)') &
    TRIM(reordered_names(3)), lengths(3), &
    TRIM(reordered_names(2)), lengths(2), &
    TRIM(reordered_names(1)), lengths(1), memory_order

  lengths = 1
  lengths(1:2) = (/ 3, 4 /)
  names = ''
  names(1) = 'south_north'
  names(2) = 'west_east'
  CALL ExtOrder('YX', lengths, status)
  CALL require_success('ExtOrder YX', status)
  CALL ExtOrderStr('YX', names, reordered_names, status)
  CALL require_success('ExtOrderStr YX', status)
  CALL reorder('YX', memory_order)
  CALL encode_blanks(memory_order)
  WRITE (*, '(&
    &"LANDMASK|",A,"=",I0,"|",A,"=",I0,"|MemoryOrder=",A)') &
    TRIM(reordered_names(2)), lengths(2), &
    TRIM(reordered_names(1)), lengths(1), memory_order

  CALL GetDim('0', dimension_count, status)
  CALL require_success('GetDim scalar', status)
  CALL reorder('0', memory_order)
  CALL encode_blanks(memory_order)
  WRITE (*, '("XTIME||MemoryOrder=",A)') memory_order

CONTAINS

  SUBROUTINE encode_blanks(value)
    CHARACTER(LEN=*), INTENT(INOUT) :: value
    INTEGER :: index

    DO index = 1, LEN(value)
      IF (value(index:index) == ' ') value(index:index) = '_'
    END DO
  END SUBROUTINE encode_blanks

  SUBROUTINE require_success(operation, actual_status)
    CHARACTER(LEN=*), INTENT(IN) :: operation
    INTEGER, INTENT(IN) :: actual_status

    IF (actual_status /= 0) THEN
      WRITE (*, '(A,": status ",I0)') TRIM(operation), actual_status
      ERROR STOP 1
    END IF
  END SUBROUTINE require_success

END PROGRAM wrf_schema_order_driver
