use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, GridShape, ParallelExecutionError};

use crate::{
    PressurePointGeopotentialError, PressurePointGeopotentialField,
    PressurePointGeopotentialKernels, PressurePointGeopotentialRegion,
    PressurePointGeopotentialResult,
};

impl PressurePointGeopotentialKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn calculate_pressure_point_geopotential(
        &self,
        pressure_point_geopotential: &mut Self::Field,
        perturbation_full_level_geopotential: &Self::Field,
        base_state_full_level_geopotential: &Self::Field,
        region: &PressurePointGeopotentialRegion,
    ) -> PressurePointGeopotentialResult<()> {
        validate_shape(
            pressure_point_geopotential,
            PressurePointGeopotentialField::PressurePointOutput,
            region.shape(),
        )?;
        validate_shape(
            perturbation_full_level_geopotential,
            PressurePointGeopotentialField::PerturbationFullLevel,
            region.shape(),
        )?;
        validate_shape(
            base_state_full_level_geopotential,
            PressurePointGeopotentialField::BaseStateFullLevel,
            region.shape(),
        )?;

        calculate_rows(
            self,
            pressure_point_geopotential,
            perturbation_full_level_geopotential,
            base_state_full_level_geopotential,
            region,
        )
    }
}

fn calculate_rows(
    backend: &CpuBackend,
    pressure_point_geopotential: &mut CpuField<f32>,
    perturbation_full_level_geopotential: &CpuField<f32>,
    base_state_full_level_geopotential: &CpuField<f32>,
    region: &PressurePointGeopotentialRegion,
) -> PressurePointGeopotentialResult<()> {
    let shape = region.shape();
    let west_east_points = shape.west_east_points();
    let bottom_top_points = shape.bottom_top_points();
    let perturbation = perturbation_full_level_geopotential.values();
    let base_state = base_state_full_level_geopotential.values();
    let (west_east, south_north, bottom_top) = region.output_ranges();

    backend
        .try_for_each_output_block(
            pressure_point_geopotential.values_mut(),
            west_east_points,
            |row_index, output_row| {
                let south_north_index = row_index / bottom_top_points;
                let bottom_top_index = row_index % bottom_top_points;
                if south_north.contains(&south_north_index)
                    && bottom_top.contains(&bottom_top_index)
                {
                    let current_row_start = row_index * west_east_points;
                    let upper_row_start = current_row_start + west_east_points;
                    let current_range =
                        current_row_start + west_east.start..current_row_start + west_east.end;
                    let upper_range =
                        upper_row_start + west_east.start..upper_row_start + west_east.end;
                    let output_values = &mut output_row[west_east.clone()];
                    let current_perturbation = &perturbation[current_range.clone()];
                    let upper_perturbation = &perturbation[upper_range.clone()];
                    let current_base_state = &base_state[current_range];
                    let upper_base_state = &base_state[upper_range];

                    for (offset, output) in output_values.iter_mut().enumerate() {
                        *output = 0.5
                            * (((current_base_state[offset] + upper_base_state[offset])
                                + current_perturbation[offset])
                                + upper_perturbation[offset]);
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

fn validate_shape(
    field: &CpuField<f32>,
    role: PressurePointGeopotentialField,
    expected: GridShape,
) -> PressurePointGeopotentialResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(PressurePointGeopotentialError::FieldShapeMismatch {
            field: role,
            expected,
            actual,
        });
    }
    Ok(())
}

fn map_parallel_error(error: ParallelExecutionError<Infallible>) -> PressurePointGeopotentialError {
    match error {
        ParallelExecutionError::Kernel(never) => match never {},
        ParallelExecutionError::WorkerPanicked => PressurePointGeopotentialError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. } => {
            unreachable!("validated shapes produce complete non-empty geopotential rows")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::Lines;

    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;

    #[test]
    fn is_bitwise_deterministic_across_worker_counts() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();
        let shape = GridShape::try_new(9, 8, 8).unwrap();
        let region =
            PressurePointGeopotentialRegion::try_new(shape, 1..8, 1..7, 1..6, 1..9, 1..8, 1..7)
                .unwrap();
        let perturbation = create_pattern(&one_worker, shape, 0.25);
        let base_state = create_pattern(&one_worker, shape, 0.75);
        let mut single_output = one_worker.create_field(shape, -999.0).unwrap();
        let mut parallel_output = four_workers.create_field(shape, -999.0).unwrap();

        one_worker
            .calculate_pressure_point_geopotential(
                &mut single_output,
                &perturbation,
                &base_state,
                &region,
            )
            .unwrap();
        four_workers
            .calculate_pressure_point_geopotential(
                &mut parallel_output,
                &perturbation,
                &base_state,
                &region,
            )
            .unwrap();

        assert_eq!(single_output, parallel_output);
    }

    #[test]
    fn rejects_each_field_role_before_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let expected = GridShape::try_new(5, 5, 6).unwrap();
        let wrong = GridShape::try_new(4, 5, 6).unwrap();
        let region =
            PressurePointGeopotentialRegion::try_new(expected, 1..4, 1..4, 1..4, 1..5, 1..5, 1..5)
                .unwrap();

        for role in [
            PressurePointGeopotentialField::PressurePointOutput,
            PressurePointGeopotentialField::PerturbationFullLevel,
            PressurePointGeopotentialField::BaseStateFullLevel,
        ] {
            let mut output = backend.create_field(expected, -999.0).unwrap();
            let valid = backend.create_field(expected, 1.0).unwrap();
            let invalid = backend.create_field(wrong, 1.0).unwrap();
            let result = match role {
                PressurePointGeopotentialField::PressurePointOutput => {
                    let mut invalid_output = backend.create_field(wrong, -999.0).unwrap();
                    backend.calculate_pressure_point_geopotential(
                        &mut invalid_output,
                        &valid,
                        &valid,
                        &region,
                    )
                }
                PressurePointGeopotentialField::PerturbationFullLevel => backend
                    .calculate_pressure_point_geopotential(&mut output, &invalid, &valid, &region),
                PressurePointGeopotentialField::BaseStateFullLevel => backend
                    .calculate_pressure_point_geopotential(&mut output, &valid, &invalid, &region),
            };

            assert_eq!(
                result,
                Err(PressurePointGeopotentialError::FieldShapeMismatch {
                    field: role,
                    expected,
                    actual: wrong,
                })
            );
            assert!(output.values().iter().all(|value| *value == -999.0));
        }
    }

    #[test]
    fn matches_upstream_fortran_clipping_sentinels_and_exceptional_order() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let mut expected = ExpectedPressurePointGeopotentialOutput::new(include_str!(
            "../../../test-data/pressure_point_geopotential.out.correct"
        ));

        for case in oracle_cases() {
            let mut fixture = create_oracle_fixture(&backend, case);
            backend
                .calculate_pressure_point_geopotential(
                    &mut fixture.pressure_point_geopotential,
                    &fixture.perturbation_full_level_geopotential,
                    &fixture.base_state_full_level_geopotential,
                    &fixture.region,
                )
                .unwrap();
            expected.assert_case(case.name, &fixture.pressure_point_geopotential);
        }
        expected.finish();
    }

    const MEMORY_WEST_EAST_START: i32 = -2;
    const MEMORY_BOTTOM_TOP_START: i32 = -1;
    const MEMORY_SOUTH_NORTH_START: i32 = 3;
    const SENTINEL: f32 = -999.0;

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
        pressure_point_geopotential: CpuField<f32>,
        perturbation_full_level_geopotential: CpuField<f32>,
        base_state_full_level_geopotential: CpuField<f32>,
        region: PressurePointGeopotentialRegion,
    }

    fn create_oracle_fixture(backend: &CpuBackend, case: OracleCase) -> OracleFixture {
        let shape = GridShape::try_new(8, 7, 7).unwrap();
        let mut perturbation = backend.create_field(shape, 0.0).unwrap();
        let mut base_state = backend.create_field(shape, 0.0).unwrap();

        for south_north in 0..shape.south_north_points() {
            let logical_south_north = south_north as i32 + MEMORY_SOUTH_NORTH_START;
            for bottom_top in 0..shape.bottom_top_points() {
                let logical_bottom_top = bottom_top as i32 + MEMORY_BOTTOM_TOP_START;
                for west_east in 0..shape.west_east_points() {
                    let logical_west_east = west_east as i32 + MEMORY_WEST_EAST_START;
                    let index = volume_index(west_east, bottom_top, south_north, shape);
                    perturbation.values_mut()[index] = ((125.0 + logical_west_east as f32 * 3.125)
                        - logical_bottom_top as f32 * 1.75)
                        + logical_south_north as f32 * 0.875;
                    base_state.values_mut()[index] = ((875.0 - logical_west_east as f32 * 2.375)
                        + logical_bottom_top as f32 * 1.125)
                        - logical_south_north as f32 * 0.625;
                }
            }
        }
        if case.exceptional {
            set_exceptional_values(&mut perturbation, &mut base_state, shape);
        }

        OracleFixture {
            pressure_point_geopotential: backend.create_field(shape, SENTINEL).unwrap(),
            perturbation_full_level_geopotential: perturbation,
            base_state_full_level_geopotential: base_state,
            region: PressurePointGeopotentialRegion::try_new(
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
        set_logical_value(base_state, -1, 1, 4, f32::MAX, shape);
        set_logical_value(base_state, -1, 2, 4, f32::MAX, shape);
        set_logical_value(perturbation, -1, 1, 4, -f32::MAX, shape);
        set_logical_value(perturbation, -1, 2, 4, -f32::MAX, shape);

        for bottom_top in [1, 2] {
            set_logical_value(base_state, 0, bottom_top, 4, -0.0, shape);
            set_logical_value(perturbation, 0, bottom_top, 4, -0.0, shape);
        }

        set_logical_value(base_state, 1, 1, 4, f32::INFINITY, shape);
        set_logical_value(base_state, 1, 2, 4, f32::NEG_INFINITY, shape);
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
        (south_north * shape.bottom_top_points() + bottom_top) * shape.west_east_points()
            + west_east
    }

    struct ExpectedPressurePointGeopotentialOutput<'a> {
        lines: Lines<'a>,
    }

    impl<'a> ExpectedPressurePointGeopotentialOutput<'a> {
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

    fn create_pattern(backend: &CpuBackend, shape: GridShape, offset: f32) -> CpuField<f32> {
        let mut field = backend.create_field(shape, 0.0).unwrap();
        for (index, value) in field.values_mut().iter_mut().enumerate() {
            *value = offset + index as f32 * 0.000_1;
        }
        field
    }
}
