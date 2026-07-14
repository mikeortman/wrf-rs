#include <netcdf.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum {
  WRF_REAL = 104,
  WRF_DOUBLE = 105,
  WRF_INTEGER = 106,
  WRF_LOGICAL = 107,
};

static void check(int status, const char *context) {
  if (status == NC_NOERR) {
    return;
  }
  fprintf(stderr, "%s: %s\n", context, nc_strerror(status));
  exit(EXIT_FAILURE);
}

static void put_text_attribute(int file, int variable, const char *name,
                               const char *value) {
  const size_t length = strlen(value);
  if (length == 0) {
    const char null_value = '\0';
    check(nc_put_att_text(file, variable, name, 1, &null_value), name);
    return;
  }
  check(nc_put_att_text(file, variable, name, length, value), name);
}

static void put_field_metadata(int file, int variable, int field_type,
                               const char memory_order[3],
                               const char *description, const char *units,
                               const char *stagger) {
  check(nc_put_att_int(file, variable, "FieldType", NC_INT, 1, &field_type),
        "FieldType");
  check(nc_put_att_text(file, variable, "MemoryOrder", 3, memory_order),
        "MemoryOrder");
  put_text_attribute(file, variable, "description", description);
  put_text_attribute(file, variable, "units", units);
  put_text_attribute(file, variable, "stagger", stagger);
}

static float float_from_bits(uint32_t bits) {
  float value;
  memcpy(&value, &bits, sizeof(value));
  return value;
}

static double double_from_bits(uint64_t bits) {
  double value;
  memcpy(&value, &bits, sizeof(value));
  return value;
}

static void fill_float(float *values, size_t length, size_t offset) {
  for (size_t index = 0; index < length; ++index) {
    values[index] = (float)(offset + index);
  }
  if (length >= 3) {
    values[0] = float_from_bits(UINT32_C(0x80000000));
    values[1] = float_from_bits(UINT32_C(0x7fc01234));
    values[2] = float_from_bits(UINT32_C(0x7f800000));
  }
}

static void fill_double(double *values, size_t length, size_t offset) {
  for (size_t index = 0; index < length; ++index) {
    values[index] = (double)(offset + index);
  }
  if (length >= 3) {
    values[0] = double_from_bits(UINT64_C(0x8000000000000000));
    values[1] = double_from_bits(UINT64_C(0x7ff8000000001234));
    values[2] = double_from_bits(UINT64_C(0x7ff0000000000000));
  }
}

static void fill_integer(int *values, size_t length, int offset) {
  for (size_t index = 0; index < length; ++index) {
    values[index] = offset + (int)index;
  }
}

static void define_global_metadata(int file) {
  put_text_attribute(file, NC_GLOBAL, "TITLE", " OUTPUT FROM WRF V4.7.1 MODEL");
  put_text_attribute(file, NC_GLOBAL, "START_DATE", "2000-09-18_16:42:01");
  put_text_attribute(file, NC_GLOBAL, "SIMULATION_START_DATE",
                     "2000-09-18_16:00:00");

  const int west_east_grid_dimension = 5;
  const int south_north_grid_dimension = 4;
  const int bottom_top_grid_dimension = 3;
  const float spacing = 12000.0f;
  const int restart = 1;
  check(nc_put_att_int(file, NC_GLOBAL, "WEST-EAST_GRID_DIMENSION", NC_INT, 1,
                       &west_east_grid_dimension),
        "WEST-EAST_GRID_DIMENSION");
  check(nc_put_att_int(file, NC_GLOBAL, "SOUTH-NORTH_GRID_DIMENSION", NC_INT,
                       1, &south_north_grid_dimension),
        "SOUTH-NORTH_GRID_DIMENSION");
  check(nc_put_att_int(file, NC_GLOBAL, "BOTTOM-TOP_GRID_DIMENSION", NC_INT, 1,
                       &bottom_top_grid_dimension),
        "BOTTOM-TOP_GRID_DIMENSION");
  check(nc_put_att_float(file, NC_GLOBAL, "DX", NC_FLOAT, 1, &spacing), "DX");
  check(nc_put_att_float(file, NC_GLOBAL, "DY", NC_FLOAT, 1, &spacing), "DY");
  put_text_attribute(file, NC_GLOBAL, "GRIDTYPE", "C");
  check(nc_put_att_int(file, NC_GLOBAL, "FLAG_RESTART", NC_INT, 1, &restart),
        "FLAG_RESTART");
}

static void write_fixture(const char *path) {
  int file;
  check(nc_create(path, NC_CLOBBER | NC_64BIT_OFFSET, &file), "create restart");

  int time_dimension;
  int date_string_dimension;
  int west_east_dimension;
  int south_north_dimension;
  int bottom_top_dimension;
  int west_east_staggered_dimension;
  int south_north_staggered_dimension;
  int bottom_top_staggered_dimension;
  int soil_dimension;
  int soil_staggered_dimension;
  int modes_dimension;
  int modes_staggered_dimension;
  int category_dimension;
  int anonymous_dimension;
  check(nc_def_dim(file, "Time", NC_UNLIMITED, &time_dimension), "Time");
  check(nc_def_dim(file, "DateStrLen", 19, &date_string_dimension),
        "DateStrLen");
  check(nc_def_dim(file, "west_east", 4, &west_east_dimension), "west_east");
  check(nc_def_dim(file, "south_north", 3, &south_north_dimension),
        "south_north");
  check(nc_def_dim(file, "bottom_top", 2, &bottom_top_dimension), "bottom_top");
  check(nc_def_dim(file, "west_east_stag", 5, &west_east_staggered_dimension),
        "west_east_stag");
  check(nc_def_dim(file, "south_north_stag", 4,
                   &south_north_staggered_dimension),
        "south_north_stag");
  check(nc_def_dim(file, "bottom_top_stag", 3,
                   &bottom_top_staggered_dimension),
        "bottom_top_stag");
  check(nc_def_dim(file, "soil_layers", 4, &soil_dimension), "soil_layers");
  check(nc_def_dim(file, "soil_layers_stag", 4, &soil_staggered_dimension),
        "soil_layers_stag");
  check(nc_def_dim(file, "modes", 2, &modes_dimension), "modes");
  check(nc_def_dim(file, "modes_stag", 2, &modes_staggered_dimension),
        "modes_stag");
  check(nc_def_dim(file, "DIM0012", 6, &category_dimension), "DIM0012");
  check(nc_def_dim(file, "DIM0013", 7, &anonymous_dimension), "DIM0013");

  define_global_metadata(file);

  const int times_dimensions[] = {time_dimension, date_string_dimension};
  const int mass_dimensions[] = {time_dimension, bottom_top_dimension,
                                 south_north_dimension, west_east_dimension};
  const int x_staggered_dimensions[] = {
      time_dimension, bottom_top_dimension, south_north_dimension,
      west_east_staggered_dimension};
  const int y_staggered_dimensions[] = {
      time_dimension, bottom_top_dimension, south_north_staggered_dimension,
      west_east_dimension};
  const int z_staggered_dimensions[] = {
      time_dimension, bottom_top_staggered_dimension, south_north_dimension,
      west_east_dimension};
  const int surface_dimensions[] = {time_dimension, south_north_dimension,
                                    west_east_dimension};
  const int soil_dimensions[] = {time_dimension, soil_dimension};
  const int soil_staggered_dimensions[] = {time_dimension,
                                           soil_staggered_dimension};
  const int mode_dimensions[] = {time_dimension, modes_dimension};
  const int mode_staggered_dimensions[] = {time_dimension,
                                           modes_staggered_dimension};
  const int category_dimensions[] = {time_dimension, category_dimension};
  const int anonymous_dimensions[] = {time_dimension, anonymous_dimension};
  const int scalar_dimensions[] = {time_dimension};

  int times;
  int temperature;
  int u;
  int v;
  int w;
  int land_mask;
  int energy;
  int active;
  int soil;
  int soil_staggered;
  int mode;
  int mode_staggered;
  int category;
  int anonymous;
  int model_minutes;
  int tendency_1;
  int tendency_2;
  check(nc_def_var(file, "Times", NC_CHAR, 2, times_dimensions, &times), "Times");
  check(nc_def_var(file, "T", NC_FLOAT, 4, mass_dimensions, &temperature), "T");
  put_field_metadata(file, temperature, WRF_REAL, "XYZ", "potential temperature",
                     "K", "");
  check(nc_def_var(file, "U", NC_FLOAT, 4, x_staggered_dimensions, &u), "U");
  put_field_metadata(file, u, WRF_REAL, "XYZ", "x-wind component", "m s-1", "X");
  check(nc_def_var(file, "V", NC_FLOAT, 4, y_staggered_dimensions, &v), "V");
  put_field_metadata(file, v, WRF_REAL, "XYZ", "y-wind component", "m s-1", "Y");
  check(nc_def_var(file, "W", NC_FLOAT, 4, z_staggered_dimensions, &w), "W");
  put_field_metadata(file, w, WRF_REAL, "XYZ", "z-wind component", "m s-1", "Z");
  check(nc_def_var(file, "LANDMASK", NC_INT, 3, surface_dimensions, &land_mask),
        "LANDMASK");
  put_field_metadata(file, land_mask, WRF_INTEGER, "XY ", "land mask", "1", "");
  check(nc_def_var(file, "ENERGY", NC_DOUBLE, 4, mass_dimensions, &energy),
        "ENERGY");
  put_field_metadata(file, energy, WRF_DOUBLE, "XYZ", "total energy", "J", "");
  check(nc_def_var(file, "ACTIVE", NC_INT, 3, surface_dimensions, &active),
        "ACTIVE");
  put_field_metadata(file, active, WRF_LOGICAL, "XY ", "active cell", "1", "");
  check(nc_def_var(file, "SOIL", NC_FLOAT, 2, soil_dimensions, &soil), "SOIL");
  put_field_metadata(file, soil, WRF_REAL, "Z  ", "soil state", "kg kg-1", "");
  check(nc_def_var(file, "SOILSTAG", NC_FLOAT, 2, soil_staggered_dimensions,
                   &soil_staggered),
        "SOILSTAG");
  put_field_metadata(file, soil_staggered, WRF_REAL, "Z  ",
                     "staggered soil state", "kg kg-1", "Z");
  check(nc_def_var(file, "MODE", NC_FLOAT, 2, mode_dimensions, &mode), "MODE");
  put_field_metadata(file, mode, WRF_REAL, "Z  ", "mode state", "1", "");
  check(nc_def_var(file, "MODESTAG", NC_FLOAT, 2, mode_staggered_dimensions,
                   &mode_staggered),
        "MODESTAG");
  put_field_metadata(file, mode_staggered, WRF_REAL, "Z  ",
                     "staggered mode state", "1", "Z");
  check(nc_def_var(file, "CATEGORY", NC_INT, 2, category_dimensions, &category),
        "CATEGORY");
  put_field_metadata(file, category, WRF_INTEGER, "C  ", "category code", "1", "");
  check(nc_def_var(file, "ANON", NC_FLOAT, 2, anonymous_dimensions, &anonymous),
        "ANON");
  put_field_metadata(file, anonymous, WRF_REAL, "C  ", "anonymous coordinate", "1",
                     "");
  check(nc_def_var(file, "XTIME", NC_FLOAT, 1, scalar_dimensions, &model_minutes),
        "XTIME");
  put_field_metadata(file, model_minutes, WRF_REAL, "0  ", "minutes since start",
                     "minutes", "");
  check(nc_def_var(file, "TEND_1", NC_FLOAT, 4, mass_dimensions, &tendency_1),
        "TEND_1");
  put_field_metadata(file, tendency_1, WRF_REAL, "XYZ", "time-level tendency",
                     "K s-1", "");
  check(nc_def_var(file, "TEND_2", NC_FLOAT, 4, mass_dimensions, &tendency_2),
        "TEND_2");
  put_field_metadata(file, tendency_2, WRF_REAL, "XYZ", "time-level tendency",
                     "K s-1", "");

  check(nc_enddef(file), "end definition");

  const size_t times_start[] = {0, 0};
  const size_t times_count[] = {1, 19};
  check(nc_put_vara_text(file, times, times_start, times_count,
                         "2000-09-18_16:42:01"),
        "write Times");

  float float_values[36];
  fill_float(float_values, 24, 100);
  check(nc_put_var_float(file, temperature, float_values), "write T");
  fill_float(float_values, 30, 200);
  check(nc_put_var_float(file, u, float_values), "write U");
  fill_float(float_values, 32, 300);
  check(nc_put_var_float(file, v, float_values), "write V");
  fill_float(float_values, 36, 400);
  check(nc_put_var_float(file, w, float_values), "write W");

  int integer_values[12];
  fill_integer(integer_values, 12, 500);
  check(nc_put_var_int(file, land_mask, integer_values), "write LANDMASK");
  double double_values[24];
  fill_double(double_values, 24, 600);
  check(nc_put_var_double(file, energy, double_values), "write ENERGY");
  for (size_t index = 0; index < 12; ++index) {
    integer_values[index] = index % 3 == 0 ? 1 : 0;
  }
  check(nc_put_var_int(file, active, integer_values), "write ACTIVE");

  fill_float(float_values, 4, 700);
  check(nc_put_var_float(file, soil, float_values), "write SOIL");
  fill_float(float_values, 4, 800);
  check(nc_put_var_float(file, soil_staggered, float_values), "write SOILSTAG");
  fill_float(float_values, 2, 900);
  check(nc_put_var_float(file, mode, float_values), "write MODE");
  fill_float(float_values, 2, 1000);
  check(nc_put_var_float(file, mode_staggered, float_values), "write MODESTAG");
  fill_integer(integer_values, 6, 1100);
  check(nc_put_var_int(file, category, integer_values), "write CATEGORY");
  fill_float(float_values, 7, 1200);
  check(nc_put_var_float(file, anonymous, float_values), "write ANON");
  float_values[0] = 60.0f;
  check(nc_put_var_float(file, model_minutes, float_values), "write XTIME");
  fill_float(float_values, 24, 1300);
  check(nc_put_var_float(file, tendency_1, float_values), "write TEND_1");
  fill_float(float_values, 24, 1400);
  check(nc_put_var_float(file, tendency_2, float_values), "write TEND_2");

  check(nc_close(file), "close restart");
}

int main(int argument_count, char **arguments) {
  if (argument_count < 2 || argument_count > 3) {
    fprintf(stderr, "usage: minimal_wrf_restart PATH [COUNT]\n");
    return EXIT_FAILURE;
  }
  const long repetitions =
      argument_count == 3 ? strtol(arguments[2], NULL, 10) : 1;
  if (repetitions < 1) {
    fprintf(stderr, "COUNT must be positive\n");
    return EXIT_FAILURE;
  }
  for (long repetition = 0; repetition < repetitions; ++repetition) {
    write_fixture(arguments[1]);
  }
  return EXIT_SUCCESS;
}
