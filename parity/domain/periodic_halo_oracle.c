#include <mpi.h>
#include <stdio.h>
#include <stdlib.h>

#include "rsl_lite.h"

extern void RSL_LITE_INIT_PERIOD(
    int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *,
    int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *,
    int *, int *);
extern void RSL_LITE_PACK_PERIOD(
    int *, char *, int *, int *, int *, int *, int *, int *,
    int *, int *, int *, int *,
    int *, int *, int *, int *, int *, int *,
    int *, int *, int *, int *, int *, int *,
    int *, int *, int *, int *, int *, int *);
extern void RSL_LITE_EXCH_PERIOD_X(int *, int *, int *, int *, int *);
extern void RSL_LITE_EXCH_PERIOD_Y(int *, int *, int *, int *, int *);

void *rsl_malloc(char *file, int line, int size) {
    (void)file;
    (void)line;
    return malloc((size_t)size);
}

void rsl_free(char **pointer) {
    free(*pointer);
    *pointer = NULL;
}

static int field_index(
    int i,
    int k,
    int j,
    int ims,
    int ime,
    int kms,
    int kme,
    int jms) {
    const int west_east_length = ime - ims + 1;
    const int bottom_top_length = kme - kms + 1;
    return ((j - jms) * bottom_top_length + (k - kms)) * west_east_length +
           (i - ims);
}

static int is_periodic_destination(
    int process_column,
    int process_row,
    int i,
    int j,
    int ips,
    int ipe,
    int jps,
    int jpe,
    int halo_width,
    int west_east_stagger,
    int south_north_stagger) {
    const int periodic_y_transverse =
        (j >= jps && j <= jpe) ||
        (process_row == 0 && j >= jps - halo_width && j <= jps - 1) ||
        (process_row == 1 && j >= jpe &&
         j <= jpe + halo_width - 1 + south_north_stagger);
    const int periodic_x_transverse =
        (i >= ips && i <= ipe) ||
        (process_column == 0 && i >= ips - halo_width && i <= ips - 1) ||
        (process_column == 1 && i >= ipe &&
         i <= ipe + halo_width - 1 + west_east_stagger);
    const int west_destination = process_column == 0 &&
                                 i >= ips - halo_width && i <= ips - 1 &&
                                 periodic_y_transverse;
    const int east_destination =
        process_column == 1 && i >= ipe &&
        i <= ipe + halo_width - 1 + west_east_stagger &&
        periodic_y_transverse;
    const int south_destination = process_row == 0 &&
                                  j >= jps - halo_width && j <= jps - 1 &&
                                  periodic_x_transverse;
    const int north_destination =
        process_row == 1 && j >= jpe &&
        j <= jpe + halo_width - 1 + south_north_stagger &&
        periodic_x_transverse;
    return west_destination || east_destination || south_destination ||
           north_destination;
}

int main(int argc, char **argv) {
    MPI_Init(&argc, &argv);

    int world_size = 0;
    MPI_Comm_size(MPI_COMM_WORLD, &world_size);
    if (world_size != 4) {
        MPI_Abort(MPI_COMM_WORLD, 2);
    }

    int dimensions[2] = {2, 2};
    int periodic[2] = {1, 1};
    MPI_Comm cartesian;
    MPI_Cart_create(MPI_COMM_WORLD, 2, dimensions, periodic, 0, &cartesian);

    int rank = 0;
    int coordinates[2] = {0, 0};
    MPI_Comm_rank(cartesian, &rank);
    MPI_Cart_coords(cartesian, rank, 2, coordinates);

    int communicator = (int)MPI_Comm_c2f(cartesian);
    int process_columns = 2;
    int process_rows = 2;
    int halo_width = 2;
    int ids = 1;
    int ide = 10;
    int jds = 1;
    int jde = 8;
    int kds = 1;
    int kde = 2;
    int ips = coordinates[1] * 5 + 1;
    int ipe = ips + 4;
    int jps = coordinates[0] * 4 + 1;
    int jpe = jps + 3;
    int kps = 1;
    int kpe = 2;
    int ims = ips - 3;
    int ime = ipe + 3;
    int jms = jps - 3;
    int jme = jpe + 3;
    int kms = 1;
    int kme = 2;
    int point_count = (ime - ims + 1) * (kme - kms + 1) * (jme - jms + 1);
    int *field = malloc((size_t)point_count * sizeof(int));
    for (int index = 0; index < point_count; ++index) field[index] = -1;
    for (int j = jps; j <= jpe; ++j) {
        for (int k = kps; k <= kpe; ++k) {
            for (int i = ips; i <= ipe; ++i) {
                field[field_index(i, k, j, ims, ime, kms, kme, jms)] =
                    (i - 1) + 100 * (j - 1) + 10000 * (k - 1);
            }
        }
    }

    int zero = 0;
    int one = 1;
    int integer_size = (int)sizeof(int);
    RSL_LITE_INIT_PERIOD(
        &communicator,
        &halo_width,
        &zero,
        &zero,
        &zero,
        &one,
        &zero,
        &integer_size,
        &zero,
        &zero,
        &zero,
        &zero,
        &zero,
        &zero,
        &rank,
        &world_size,
        &process_columns,
        &process_rows,
        &ips,
        &ipe,
        &jps,
        &jpe,
        &kps,
        &kpe);

    int memory_order = 5;
    int pack = 0;
    int unpack = 1;
    int south_north_axis = 0;
    int west_east_axis = 1;
    int west_east_stagger = 1;
    int south_north_stagger = 0;

#define PERIOD_ARGUMENTS(axis, operation, stagger)                               \
    &communicator, (char *)field, &halo_width, &integer_size, axis, operation,  \
        &memory_order, stagger, &rank, &world_size, &process_columns,            \
        &process_rows, &ids, &ide, &jds, &jde, &kds, &kde, &ims, &ime, &jms,   \
        &jme, &kms, &kme, &ips, &ipe, &jps, &jpe, &kps, &kpe

    RSL_LITE_PACK_PERIOD(PERIOD_ARGUMENTS(
        &south_north_axis, &pack, &south_north_stagger));
    RSL_LITE_EXCH_PERIOD_Y(
        &communicator, &rank, &world_size, &process_columns, &process_rows);
    RSL_LITE_PACK_PERIOD(PERIOD_ARGUMENTS(
        &south_north_axis, &unpack, &south_north_stagger));
    RSL_LITE_PACK_PERIOD(
        PERIOD_ARGUMENTS(&west_east_axis, &pack, &west_east_stagger));
    RSL_LITE_EXCH_PERIOD_X(
        &communicator, &rank, &world_size, &process_columns, &process_rows);
    RSL_LITE_PACK_PERIOD(
        PERIOD_ARGUMENTS(&west_east_axis, &unpack, &west_east_stagger));

    int *gathered = rank == 0
                        ? malloc((size_t)point_count * (size_t)world_size * sizeof(int))
                        : NULL;
    MPI_Gather(
        field,
        point_count,
        MPI_INT,
        gathered,
        point_count,
        MPI_INT,
        0,
        cartesian);

    if (rank == 0) {
        for (int output_rank = 0; output_rank < world_size; ++output_rank) {
            int output_coordinates[2] = {0, 0};
            MPI_Cart_coords(cartesian, output_rank, 2, output_coordinates);
            const int output_ips = output_coordinates[1] * 5 + 1;
            const int output_ipe = output_ips + 4;
            const int output_jps = output_coordinates[0] * 4 + 1;
            const int output_jpe = output_jps + 3;
            const int output_ims = output_ips - 3;
            const int output_ime = output_ipe + 3;
            const int output_jms = output_jps - 3;
            const int output_jme = output_jpe + 3;
            const int *output = gathered + output_rank * point_count;
            for (int j = output_jms; j <= output_jme; ++j) {
                for (int k = kms; k <= kme; ++k) {
                    for (int i = output_ims; i <= output_ime; ++i) {
                        if (is_periodic_destination(
                                output_coordinates[1],
                                output_coordinates[0],
                                i,
                                j,
                                output_ips,
                                output_ipe,
                                output_jps,
                                output_jpe,
                                halo_width,
                                west_east_stagger,
                                south_north_stagger)) {
                            printf(
                                "rank=%d i=%d k=%d j=%d value=%d\n",
                                output_rank,
                                i,
                                k,
                                j,
                                output[field_index(
                                    i,
                                    k,
                                    j,
                                    output_ims,
                                    output_ime,
                                    kms,
                                    kme,
                                    output_jms)]);
                        }
                    }
                }
            }
        }
    }

    free(gathered);
    free(field);
    MPI_Comm_free(&cartesian);
    MPI_Finalize();
    return 0;
}
