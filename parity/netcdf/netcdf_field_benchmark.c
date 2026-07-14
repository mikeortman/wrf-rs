#include <netcdf.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define X_LENGTH 256
#define Y_LENGTH 256
#define Z_LENGTH 64
#define ELEMENT_COUNT ((size_t)X_LENGTH * Y_LENGTH * Z_LENGTH)

static void check(int status, const char *operation) {
  if (status != NC_NOERR) {
    fprintf(stderr, "%s: %s\n", operation, nc_strerror(status));
    exit(EXIT_FAILURE);
  }
}

static void write_field(const char *path, const float *values) {
  int file;
  int time_dimension;
  int z_dimension;
  int y_dimension;
  int x_dimension;
  int variable;
  check(nc_create(path, NC_CLOBBER | NC_64BIT_OFFSET, &file), "create");
  check(nc_def_dim(file, "Time", NC_UNLIMITED, &time_dimension), "Time");
  check(nc_def_dim(file, "bottom_top", Z_LENGTH, &z_dimension), "bottom_top");
  check(nc_def_dim(file, "south_north", Y_LENGTH, &y_dimension), "south_north");
  check(nc_def_dim(file, "west_east", X_LENGTH, &x_dimension), "west_east");
  const int dimensions[] = {time_dimension, z_dimension, y_dimension,
                            x_dimension};
  check(nc_def_var(file, "THM", NC_FLOAT, 4, dimensions, &variable), "THM");
  check(nc_enddef(file), "end definition");
  const size_t start[] = {0, 0, 0, 0};
  const size_t count[] = {1, Z_LENGTH, Y_LENGTH, X_LENGTH};
  check(nc_put_vara_float(file, variable, start, count, values), "write THM");
  check(nc_close(file), "close");
}

int main(int argument_count, char **arguments) {
  if (argument_count != 3) {
    fprintf(stderr, "usage: netcdf_field_benchmark PATH COUNT\n");
    return EXIT_FAILURE;
  }
  const long repetitions = strtol(arguments[2], NULL, 10);
  if (repetitions < 1) {
    fprintf(stderr, "COUNT must be positive\n");
    return EXIT_FAILURE;
  }
  float *values = malloc(ELEMENT_COUNT * sizeof(float));
  if (values == NULL) {
    fprintf(stderr, "allocation failed\n");
    return EXIT_FAILURE;
  }
  for (size_t index = 0; index < ELEMENT_COUNT; ++index) {
    values[index] = (float)index * 0.125f;
  }
  struct timespec start;
  struct timespec end;
  check(clock_gettime(CLOCK_MONOTONIC, &start) == 0 ? NC_NOERR : NC_EINVAL,
        "start clock");
  for (long repetition = 0; repetition < repetitions; ++repetition) {
    write_field(arguments[1], values);
  }
  check(clock_gettime(CLOCK_MONOTONIC, &end) == 0 ? NC_NOERR : NC_EINVAL,
        "end clock");
  const double elapsed = (double)(end.tv_sec - start.tv_sec) +
                         (double)(end.tv_nsec - start.tv_nsec) / 1000000000.0;
  printf("elapsed_seconds=%.6f\n", elapsed);
  free(values);
  return EXIT_SUCCESS;
}
