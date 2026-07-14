use std::str::Lines;

use wrf_compute::{ComputeBackend, CpuBackend, CpuField, FieldStorage, GridShape};

use crate::{InverseDensityKernels, InverseDensityRegion};

const MEMORY_WEST_EAST_START: i32 = -2;
const MEMORY_BOTTOM_TOP_START: i32 = -1;
const MEMORY_SOUTH_NORTH_START: i32 = 3;
const SENTINEL: f32 = -999.0;

#[test]
fn matches_upstream_fortran_for_clipping_sentinels_and_exceptional_values() {
    let backend = CpuBackend::try_with_worker_count(3).unwrap();
    let mut expected = ExpectedInverseDensityOutput::new(include_str!(
        "../../../test-data/inverse_density.out.correct"
    ));

    for case in oracle_cases() {
        let mut fixture = create_fixture(&backend, case);
        apply_fixture(&backend, &mut fixture);
        expected.assert_case(case.name, &fixture.full_inverse_density);
    }
    expected.finish();
}

#[derive(Clone, Copy)]
struct OracleCase {
    name: &'static str,
    west_east_tile: (i32, i32),
    south_north_tile: (i32, i32),
    bottom_top_tile: (i32, i32),
    exceptional: bool,
}

fn oracle_cases() -> [OracleCase; 6] {
    [
        OracleCase {
            name: "interior",
            west_east_tile: (0, 2),
            south_north_tile: (5, 6),
            bottom_top_tile: (1, 2),
            exceptional: false,
        },
        OracleCase {
            name: "x_upper",
            west_east_tile: (-1, 4),
            south_north_tile: (5, 6),
            bottom_top_tile: (1, 2),
            exceptional: false,
        },
        OracleCase {
            name: "y_upper",
            west_east_tile: (0, 2),
            south_north_tile: (4, 8),
            bottom_top_tile: (1, 2),
            exceptional: false,
        },
        OracleCase {
            name: "z_upper",
            west_east_tile: (0, 2),
            south_north_tile: (5, 6),
            bottom_top_tile: (1, 4),
            exceptional: false,
        },
        OracleCase {
            name: "all_upper",
            west_east_tile: (-1, 4),
            south_north_tile: (4, 8),
            bottom_top_tile: (1, 4),
            exceptional: false,
        },
        OracleCase {
            name: "exceptional",
            west_east_tile: (-1, 4),
            south_north_tile: (4, 8),
            bottom_top_tile: (1, 4),
            exceptional: true,
        },
    ]
}

struct OracleFixture {
    full_inverse_density: CpuField<f32>,
    perturbation_inverse_density: CpuField<f32>,
    base_state_inverse_density: CpuField<f32>,
    region: InverseDensityRegion,
}

fn create_fixture(backend: &CpuBackend, case: OracleCase) -> OracleFixture {
    let shape = GridShape::try_new(8, 7, 7).unwrap();
    let mut perturbation_inverse_density = backend.create_field(shape, 0.0).unwrap();
    let mut base_state_inverse_density = backend.create_field(shape, 0.0).unwrap();

    for south_north in 0..shape.south_north_points() {
        let logical_south_north = south_north as i32 + MEMORY_SOUTH_NORTH_START;
        for bottom_top in 0..shape.bottom_top_points() {
            let logical_bottom_top = bottom_top as i32 + MEMORY_BOTTOM_TOP_START;
            for west_east in 0..shape.west_east_points() {
                let logical_west_east = west_east as i32 + MEMORY_WEST_EAST_START;
                let index = volume_index(west_east, bottom_top, south_north, shape);
                perturbation_inverse_density.values_mut()[index] = ((0.125
                    + logical_west_east as f32 * 0.031)
                    - logical_bottom_top as f32 * 0.017)
                    + logical_south_north as f32 * 0.009;
                base_state_inverse_density.values_mut()[index] = ((0.875
                    - logical_west_east as f32 * 0.023)
                    + logical_bottom_top as f32 * 0.011)
                    - logical_south_north as f32 * 0.007;
            }
        }
    }
    if case.exceptional {
        set_exceptional_values(
            &mut perturbation_inverse_density,
            &mut base_state_inverse_density,
            shape,
        );
    }

    OracleFixture {
        full_inverse_density: backend.create_field(shape, SENTINEL).unwrap(),
        perturbation_inverse_density,
        base_state_inverse_density,
        region: InverseDensityRegion::try_new(
            shape,
            logical_range(-1, 3, MEMORY_WEST_EAST_START),
            logical_range(4, 7, MEMORY_SOUTH_NORTH_START),
            logical_range(1, 3, MEMORY_BOTTOM_TOP_START),
            inclusive_logical_range(case.west_east_tile, MEMORY_WEST_EAST_START),
            inclusive_logical_range(case.south_north_tile, MEMORY_SOUTH_NORTH_START),
            inclusive_logical_range(case.bottom_top_tile, MEMORY_BOTTOM_TOP_START),
        )
        .unwrap(),
    }
}

fn set_exceptional_values(
    perturbation: &mut CpuField<f32>,
    base_state: &mut CpuField<f32>,
    shape: GridShape,
) {
    set_logical_value(perturbation, -1, 1, 4, f32::MAX, shape);
    set_logical_value(base_state, -1, 1, 4, f32::MAX, shape);
    set_logical_value(perturbation, 0, 1, 4, f32::MAX, shape);
    set_logical_value(base_state, 0, 1, 4, -f32::MAX, shape);
    set_logical_value(perturbation, 1, 1, 4, -0.0, shape);
    set_logical_value(base_state, 1, 1, 4, -0.0, shape);
    set_logical_value(perturbation, 2, 1, 4, f32::INFINITY, shape);
    set_logical_value(base_state, 2, 1, 4, f32::NEG_INFINITY, shape);
}

fn set_logical_value(
    field: &mut CpuField<f32>,
    west_east: i32,
    bottom_top: i32,
    south_north: i32,
    value: f32,
    shape: GridShape,
) {
    let index = volume_index(
        (west_east - MEMORY_WEST_EAST_START) as usize,
        (bottom_top - MEMORY_BOTTOM_TOP_START) as usize,
        (south_north - MEMORY_SOUTH_NORTH_START) as usize,
        shape,
    );
    field.values_mut()[index] = value;
}

fn apply_fixture(backend: &CpuBackend, fixture: &mut OracleFixture) {
    backend
        .calculate_full_inverse_density(
            &mut fixture.full_inverse_density,
            &fixture.perturbation_inverse_density,
            &fixture.base_state_inverse_density,
            &fixture.region,
        )
        .unwrap();
}

fn logical_range(start: i32, inclusive_end: i32, memory_start: i32) -> std::ops::Range<usize> {
    (start - memory_start) as usize..(inclusive_end - memory_start + 1) as usize
}

fn inclusive_logical_range(
    (start, inclusive_end): (i32, i32),
    memory_start: i32,
) -> std::ops::Range<usize> {
    logical_range(start, inclusive_end, memory_start)
}

fn volume_index(
    west_east: usize,
    bottom_top: usize,
    south_north: usize,
    shape: GridShape,
) -> usize {
    (south_north * shape.bottom_top_points() + bottom_top) * shape.west_east_points() + west_east
}

struct ExpectedInverseDensityOutput<'a> {
    lines: Lines<'a>,
}

impl<'a> ExpectedInverseDensityOutput<'a> {
    fn new(output: &'a str) -> Self {
        Self {
            lines: output.lines(),
        }
    }

    fn assert_case(&mut self, case_name: &str, field: &CpuField<f32>) {
        let shape = field.shape();
        for south_north in 0..shape.south_north_points() {
            for bottom_top in 0..shape.bottom_top_points() {
                for west_east in 0..shape.west_east_points() {
                    let actual =
                        field.values()[volume_index(west_east, bottom_top, south_north, shape)];
                    self.assert_next(
                        case_name,
                        west_east as i32 + MEMORY_WEST_EAST_START,
                        bottom_top as i32 + MEMORY_BOTTOM_TOP_START,
                        south_north as i32 + MEMORY_SOUTH_NORTH_START,
                        actual,
                    );
                }
            }
        }
    }

    fn assert_next(
        &mut self,
        case_name: &str,
        west_east: i32,
        bottom_top: i32,
        south_north: i32,
        actual: f32,
    ) {
        let line = self
            .lines
            .next()
            .unwrap_or_else(|| panic!("Fortran output ended before {case_name}"));
        let columns: Vec<_> = line.split_whitespace().collect();
        assert_eq!(columns.len(), 5, "unexpected Fortran output: {line}");
        assert_eq!(columns[0], case_name, "unexpected case at {line}");
        assert_eq!(columns[1].parse::<i32>().unwrap(), west_east);
        assert_eq!(columns[2].parse::<i32>().unwrap(), bottom_top);
        assert_eq!(columns[3].parse::<i32>().unwrap(), south_north);
        if columns[4] == "NAN" {
            assert!(actual.is_nan(), "expected NaN at {line}, got {actual:?}");
        } else {
            let expected_bits = u32::from_str_radix(columns[4], 16).unwrap();
            assert_eq!(
                actual.to_bits(),
                expected_bits,
                "value mismatch at {line}: got {:08X}",
                actual.to_bits()
            );
        }
    }

    fn finish(mut self) {
        assert_eq!(
            self.lines.next(),
            None,
            "Fortran expected output contains unconsumed values"
        );
    }
}
