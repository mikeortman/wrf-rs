#include <netcdf.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void check(int status, const char *operation) {
  if (status != NC_NOERR) {
    fprintf(stderr, "%s: %s\n", operation, nc_strerror(status));
    exit(EXIT_FAILURE);
  }
}

static void put_text_attribute(int file, int variable, const char *name,
                               const char *value) {
  check(nc_put_att_text(file, variable, name, strlen(value), value), name);
}

static void put_float_metadata(int file, int variable, const char *memory_order,
                               const char *description, const char *units,
                               const char *stagger) {
  const int field_type = 104;
  check(nc_put_att_int(file, variable, "FieldType", NC_INT, 1, &field_type),
        "FieldType");
  put_text_attribute(file, variable, "MemoryOrder", memory_order);
  put_text_attribute(file, variable, "description", description);
  put_text_attribute(file, variable, "units", units);
  put_text_attribute(file, variable, "stagger", stagger);
}

static void fill(float *values, size_t length, size_t offset) {
  for (size_t index = 0; index < length; ++index) {
    values[index] = (float)(offset + index);
  }
}

static void write_fixture(const char *path) {
  int file;
  check(nc_create(path, NC_CLOBBER | NC_64BIT_OFFSET, &file),
        "create restart");

  int time_dimension;
  int date_string_length_dimension;
  int west_east_dimension;
  int south_north_dimension;
  int bottom_top_dimension;
  int west_east_staggered_dimension;
  int south_north_staggered_dimension;
  int bottom_top_staggered_dimension;
  check(nc_def_dim(file, "Time", NC_UNLIMITED, &time_dimension), "Time");
  check(nc_def_dim(file, "DateStrLen", 19, &date_string_length_dimension),
        "DateStrLen");
  check(nc_def_dim(file, "west_east", 4, &west_east_dimension), "west_east");
  check(nc_def_dim(file, "south_north", 3, &south_north_dimension),
        "south_north");
  check(nc_def_dim(file, "bottom_top", 2, &bottom_top_dimension),
        "bottom_top");
  check(nc_def_dim(file, "west_east_stag", 5,
                   &west_east_staggered_dimension),
        "west_east_stag");
  check(nc_def_dim(file, "south_north_stag", 4,
                   &south_north_staggered_dimension),
        "south_north_stag");
  check(nc_def_dim(file, "bottom_top_stag", 3,
                   &bottom_top_staggered_dimension),
        "bottom_top_stag");

  put_text_attribute(file, NC_GLOBAL, "TITLE", " OUTPUT FROM WRF V4.7.1 MODEL");
  put_text_attribute(file, NC_GLOBAL, "START_DATE", "2000-09-18_16:42:01");
  put_text_attribute(file, NC_GLOBAL, "SIMULATION_START_DATE",
                     "2000-09-18_16:00:00");
  const int west_east_grid_dimension = 5;
  const int south_north_grid_dimension = 4;
  const int bottom_top_grid_dimension = 3;
  const float grid_spacing = 12000.0f;
  const int restart_flag = 1;
  check(nc_put_att_int(file, NC_GLOBAL, "WEST-EAST_GRID_DIMENSION", NC_INT, 1,
                       &west_east_grid_dimension),
        "WEST-EAST_GRID_DIMENSION");
  check(nc_put_att_int(file, NC_GLOBAL, "SOUTH-NORTH_GRID_DIMENSION", NC_INT, 1,
                       &south_north_grid_dimension),
        "SOUTH-NORTH_GRID_DIMENSION");
  check(nc_put_att_int(file, NC_GLOBAL, "BOTTOM-TOP_GRID_DIMENSION", NC_INT, 1,
                       &bottom_top_grid_dimension),
        "BOTTOM-TOP_GRID_DIMENSION");
  check(nc_put_att_float(file, NC_GLOBAL, "DX", NC_FLOAT, 1, &grid_spacing),
        "DX");
  check(nc_put_att_float(file, NC_GLOBAL, "DY", NC_FLOAT, 1, &grid_spacing),
        "DY");
  put_text_attribute(file, NC_GLOBAL, "GRIDTYPE", "C");
  check(nc_put_att_int(file, NC_GLOBAL, "FLAG_RESTART", NC_INT, 1,
                       &restart_flag),
        "FLAG_RESTART");

  int times;
  int u;
  int v;
  int w;
  int ph;
  int phb;
  int temperature;
  int mu;
  int mub;
  int pressure;
  int base_pressure;
  int water_vapor;
  int model_minutes;
  const int times_dimensions[] = {time_dimension, date_string_length_dimension};
  const int u_dimensions[] = {time_dimension, bottom_top_dimension,
                              south_north_dimension,
                              west_east_staggered_dimension};
  const int v_dimensions[] = {time_dimension, bottom_top_dimension,
                              south_north_staggered_dimension,
                              west_east_dimension};
  const int vertical_staggered_dimensions[] = {
      time_dimension, bottom_top_staggered_dimension, south_north_dimension,
      west_east_dimension};
  const int mass_dimensions[] = {time_dimension, bottom_top_dimension,
                                 south_north_dimension, west_east_dimension};
  const int surface_dimensions[] = {time_dimension, south_north_dimension,
                                    west_east_dimension};
  check(nc_def_var(file, "Times", NC_CHAR, 2, times_dimensions, &times),
        "Times");
  check(nc_def_var(file, "U", NC_FLOAT, 4, u_dimensions, &u), "U");
  put_float_metadata(file, u, "XYZ", "x-wind component", "m s-1", "X");
  check(nc_def_var(file, "V", NC_FLOAT, 4, v_dimensions, &v), "V");
  put_float_metadata(file, v, "XYZ", "y-wind component", "m s-1", "Y");
  check(nc_def_var(file, "W", NC_FLOAT, 4, vertical_staggered_dimensions, &w),
        "W");
  put_float_metadata(file, w, "XYZ", "z-wind component", "m s-1", "Z");
  check(nc_def_var(file, "PH", NC_FLOAT, 4, vertical_staggered_dimensions, &ph),
        "PH");
  put_float_metadata(file, ph, "XYZ", "perturbation geopotential", "m2 s-2",
                     "Z");
  check(nc_def_var(file, "PHB", NC_FLOAT, 4, vertical_staggered_dimensions,
                   &phb),
        "PHB");
  put_float_metadata(file, phb, "XYZ", "base-state geopotential", "m2 s-2",
                     "Z");
  check(nc_def_var(file, "THM", NC_FLOAT, 4, mass_dimensions, &temperature),
        "THM");
  put_float_metadata(
      file, temperature, "XYZ",
      "either 1) pert moist pot temp=(1+Rv/Rd Qv)*(theta)-T0, or 2) pert dry pot temp=theta-T0; based on use_theta_m setting",
      "K", "");
  check(nc_def_var(file, "MU", NC_FLOAT, 3, surface_dimensions, &mu), "MU");
  put_float_metadata(file, mu, "XY ", "perturbation dry air mass in column",
                     "Pa", "");
  check(nc_def_var(file, "MUB", NC_FLOAT, 3, surface_dimensions, &mub),
        "MUB");
  put_float_metadata(file, mub, "XY ", "base state dry air mass in column",
                     "Pa", "");
  check(nc_def_var(file, "P", NC_FLOAT, 4, mass_dimensions, &pressure), "P");
  put_float_metadata(file, pressure, "XYZ", "perturbation pressure", "Pa", "");
  check(nc_def_var(file, "PB", NC_FLOAT, 4, mass_dimensions, &base_pressure),
        "PB");
  put_float_metadata(file, base_pressure, "XYZ", "BASE STATE PRESSURE ", "Pa",
                     "");
  check(nc_def_var(file, "QVAPOR", NC_FLOAT, 4, mass_dimensions, &water_vapor),
        "QVAPOR");
  put_float_metadata(file, water_vapor, "XYZ", "Water vapor mixing ratio",
                     "kg kg-1", "");
  check(nc_def_var(file, "XTIME", NC_FLOAT, 1, &time_dimension, &model_minutes),
        "XTIME");
  put_float_metadata(file, model_minutes, "0  ",
                     "minutes since YYYY-MM-DD hh:mm:ss", "minutes", "");
  check(nc_enddef(file), "end definition");

  const size_t times_start[] = {0, 0};
  const size_t times_count[] = {1, 19};
  check(nc_put_vara_text(file, times, times_start, times_count,
                         "2000-09-18_16:42:01"),
        "write Times");
  float values[36];
  fill(values, 30, 100);
  check(nc_put_var_float(file, u, values), "write U");
  fill(values, 32, 200);
  check(nc_put_var_float(file, v, values), "write V");
  fill(values, 36, 300);
  check(nc_put_var_float(file, w, values), "write W");
  fill(values, 36, 400);
  check(nc_put_var_float(file, ph, values), "write PH");
  fill(values, 36, 500);
  check(nc_put_var_float(file, phb, values), "write PHB");
  fill(values, 24, 600);
  check(nc_put_var_float(file, temperature, values), "write THM");
  fill(values, 12, 700);
  check(nc_put_var_float(file, mu, values), "write MU");
  fill(values, 12, 800);
  check(nc_put_var_float(file, mub, values), "write MUB");
  fill(values, 24, 900);
  check(nc_put_var_float(file, pressure, values), "write P");
  fill(values, 24, 1000);
  check(nc_put_var_float(file, base_pressure, values), "write PB");
  fill(values, 24, 1100);
  check(nc_put_var_float(file, water_vapor, values), "write QVAPOR");
  values[0] = 60.0f;
  check(nc_put_var_float(file, model_minutes, values), "write XTIME");
  check(nc_close(file), "close restart");
}

int main(int argument_count, char **arguments) {
  if (argument_count < 2 || argument_count > 3) {
    fprintf(stderr, "usage: minimal_wrf_restart PATH [COUNT]\n");
    return EXIT_FAILURE;
  }
  const long repetitions = argument_count == 3 ? strtol(arguments[2], NULL, 10) : 1;
  if (repetitions < 1) {
    fprintf(stderr, "COUNT must be positive\n");
    return EXIT_FAILURE;
  }
  for (long repetition = 0; repetition < repetitions; ++repetition) {
    write_fixture(arguments[1]);
  }
  return EXIT_SUCCESS;
}
