PROGRAM clipped_tiles_oracle
  USE module_machine, ONLY: region_bounds
  IMPLICIT NONE

  INTEGER, PARAMETER :: ids = 1, ide = 13, jds = 1, jde = 8
  INTEGER, PARAMETER :: spx = 1, epx = 6, spy = 1, epy = 8
  INTEGER, PARAMETER :: ips = 0, ipe = 8, jps = 0, jpe = 9
  INTEGER, PARAMETER :: num_tiles_x = 3, num_tiles_y = 2
  INTEGER :: tile, column, row
  INTEGER :: west_east_start, west_east_end
  INTEGER :: south_north_start, south_north_end

  DO tile = 0, num_tiles_x * num_tiles_y - 1
    row = tile / num_tiles_x
    CALL region_bounds(spy, epy, num_tiles_y, row, &
                       south_north_start, south_north_end)
    IF (jps < spy .AND. south_north_start == spy) south_north_start = jps
    IF (jpe > epy .AND. south_north_end == epy) south_north_end = jpe
    south_north_start = MAX(south_north_start, jds)
    south_north_end = MIN(south_north_end, jde)

    column = MOD(tile, num_tiles_x)
    CALL region_bounds(spx, epx, num_tiles_x, column, &
                       west_east_start, west_east_end)
    IF (ips < spx .AND. west_east_start == spx) west_east_start = ips
    IF (ipe > epx .AND. west_east_end == epx) west_east_end = ipe
    west_east_start = MAX(west_east_start, ids)
    west_east_end = MIN(west_east_end, ide)

    WRITE(*,'(A,I0,A,I0,A,I0,A,I0,A,I0)') &
      'tile=', tile, ' ips=', west_east_start, ' ipe=', west_east_end, &
      ' jps=', south_north_start, ' jpe=', south_north_end
  END DO
END PROGRAM clipped_tiles_oracle
