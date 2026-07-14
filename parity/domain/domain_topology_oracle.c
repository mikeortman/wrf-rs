#include <limits.h>
#include <stdio.h>

#include "rsl_lite.h"

struct topology_case {
    int ids;
    int ide;
    int jds;
    int jde;
    int process_columns;
    int process_rows;
};

int main(void) {
    const struct topology_case cases[] = {
        {1, 13, 1, 8, 5, 3},
        {1, 16, 1, 11, 4, 2},
        {1, 17, 1, 9, 6, 4},
    };
    const int case_count = (int)(sizeof(cases) / sizeof(cases[0]));
    const int minimum_x = 1;
    const int minimum_y = 1;

    for (int case_index = 0; case_index < case_count; ++case_index) {
        const struct topology_case current = cases[case_index];
        for (int process_row = 0; process_row < current.process_rows; ++process_row) {
            for (int process_column = 0; process_column < current.process_columns; ++process_column) {
                int patch_start_x = INT_MAX;
                int patch_end_x = INT_MIN;
                int patch_start_y = INT_MAX;
                int patch_end_y = INT_MIN;
                for (int j = current.jds; j <= current.jde; ++j) {
                    for (int i = current.ids; i <= current.ide; ++i) {
                        int owner_x = -1;
                        int owner_y = -1;
                        int error = 0;
                        int mutable_i = i;
                        int mutable_j = j;
                        int ids = current.ids;
                        int ide = current.ide;
                        int jds = current.jds;
                        int jde = current.jde;
                        int process_columns = current.process_columns;
                        int process_rows = current.process_rows;
                        int mutable_minimum_x = minimum_x;
                        int mutable_minimum_y = minimum_y;
                        TASK_FOR_POINT(
                            &mutable_i,
                            &mutable_j,
                            &ids,
                            &ide,
                            &jds,
                            &jde,
                            &process_columns,
                            &process_rows,
                            &owner_x,
                            &owner_y,
                            &mutable_minimum_x,
                            &mutable_minimum_y,
                            &error);
                        if (error != 0) {
                            return error;
                        }
                        if (owner_x == process_column && owner_y == process_row) {
                            if (i < patch_start_x) patch_start_x = i;
                            if (i > patch_end_x) patch_end_x = i;
                            if (j < patch_start_y) patch_start_y = j;
                            if (j > patch_end_y) patch_end_y = j;
                        }
                    }
                }
                printf(
                    "case=%d patch=%d column=%d row=%d ips=%d ipe=%d jps=%d jpe=%d\n",
                    case_index,
                    process_row * current.process_columns + process_column,
                    process_column,
                    process_row,
                    patch_start_x,
                    patch_end_x,
                    patch_start_y,
                    patch_end_y);
            }
        }
    }
    return 0;
}
