#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
oracle_directory="$repository_root/upstream/WRF/external/esmf_time_f90"

for required_command in clang diff gfortran make sed; do
    if ! command -v "$required_command" >/dev/null 2>&1; then
        echo "missing command required by WRF time oracle: $required_command" >&2
        exit 1
    fi
done

if [ ! -f "$oracle_directory/Test1.F90" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

fortran_command='gfortran -ffree-form -ffree-line-length-none -fallow-argument-mismatch'
preprocessor_command='clang -E -x c -P -traditional-cpp -DTIME_F90_ONLY'

cd "$oracle_directory"
make superclean >/dev/null
make libesmf_time.a Test1_ESMF.f Test1_WRFU.f \
    "FC=$fortran_command" \
    "CPP=$preprocessor_command" \
    "WRF_SRC_ROOT_DIR=$repository_root/upstream/WRF"

# WRF v4.7.1's test uses an obsolete keyword. Patch generated build copies only;
# the pinned upstream Test1.F90 remains byte-for-byte unchanged.
for generated_test in Test1_ESMF.f Test1_WRFU.f; do
    sed 's/defaultCalendar=/defaultcalkind=/' "$generated_test" > "$generated_test.fixed"
    mv "$generated_test.fixed" "$generated_test"
done

for interface in ESMF WRFU; do
    make "Test1_${interface}.exe" \
        "FC=$fortran_command" \
        "CPP=$preprocessor_command" \
        "WRF_SRC_ROOT_DIR=$repository_root/upstream/WRF"
    "./Test1_${interface}.exe" > "Test1_${interface}.out"
    diff -u Test1.out.correct "Test1_${interface}.out"
    echo "PASS WRF time oracle: $interface"
done

make superclean >/dev/null
