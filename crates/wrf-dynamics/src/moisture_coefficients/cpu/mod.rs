mod south_north;
mod vertical;
mod west_east;

use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, GridShape, ParallelExecutionError};

use crate::{
    MoistureCoefficientError, MoistureCoefficientField, MoistureCoefficientKernels,
    MoistureCoefficientOutputs, MoistureCoefficientRegion, MoistureCoefficientResult,
    MoistureSpecies,
};

impl MoistureCoefficientKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn calculate_moisture_coefficients(
        &self,
        outputs: MoistureCoefficientOutputs<'_, Self::Field>,
        moisture: MoistureSpecies<'_, Self::Field>,
        region: &MoistureCoefficientRegion,
    ) -> MoistureCoefficientResult<()> {
        validate_fields(&outputs, moisture, region.shape())?;
        if !moisture.is_empty() {
            region.validate_active_species_neighbors()?;
        }

        let MoistureCoefficientOutputs {
            west_east: west_east_output,
            south_north: south_north_output,
            vertical: vertical_output,
        } = outputs;
        west_east::calculate(self, west_east_output, moisture, region)?;
        south_north::calculate(self, south_north_output, moisture, region)?;
        vertical::calculate(self, vertical_output, moisture, region)
    }
}

fn validate_fields(
    outputs: &MoistureCoefficientOutputs<'_, CpuField<f32>>,
    moisture: MoistureSpecies<'_, CpuField<f32>>,
    expected: GridShape,
) -> MoistureCoefficientResult<()> {
    for (field, role) in [
        (
            &*outputs.west_east,
            MoistureCoefficientField::WestEastOutput,
        ),
        (
            &*outputs.south_north,
            MoistureCoefficientField::SouthNorthOutput,
        ),
        (&*outputs.vertical, MoistureCoefficientField::VerticalOutput),
    ] {
        validate_output_shape(field, role, expected)?;
    }
    for (active_species_index, field) in moisture.active.iter().enumerate() {
        let actual = field.shape();
        if actual != expected {
            return Err(MoistureCoefficientError::SpeciesShapeMismatch {
                active_species_index,
                expected,
                actual,
            });
        }
    }
    Ok(())
}

fn validate_output_shape(
    field: &CpuField<f32>,
    role: MoistureCoefficientField,
    expected: GridShape,
) -> MoistureCoefficientResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(MoistureCoefficientError::OutputShapeMismatch {
            field: role,
            expected,
            actual,
        });
    }
    Ok(())
}

fn map_parallel_error(error: ParallelExecutionError<Infallible>) -> MoistureCoefficientError {
    match error {
        ParallelExecutionError::Kernel(never) => match never {},
        ParallelExecutionError::WorkerPanicked => MoistureCoefficientError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. } => {
            unreachable!("validated field shapes produce complete non-empty moisture rows")
        }
    }
}

#[cfg(test)]
mod tests {
    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;

    const MEMORY_WEST_EAST_START: i32 = -2;
    const MEMORY_SOUTH_NORTH_START: i32 = 3;
    const MEMORY_BOTTOM_TOP_START: i32 = -1;
    const SENTINEL_WEST_EAST: f32 = -901.0;
    const SENTINEL_SOUTH_NORTH: f32 = -902.0;
    const SENTINEL_VERTICAL: f32 = -903.0;

    #[test]
    fn matches_upstream_fortran_for_all_species_and_stagger_cases() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let mut expected = ExpectedMoistureOutput::new(include_str!(
            "../../../test-data/moisture_coefficients.out.correct"
        ));

        for case in oracle_cases() {
            let mut fixture = create_oracle_fixture(&backend, case);
            apply_fixture(&backend, &mut fixture).unwrap();
            expected.assert_case(case.name, &fixture);
        }
        expected.finish();
    }

    #[test]
    fn is_bitwise_deterministic_across_worker_counts() {
        let single_worker_backend = CpuBackend::try_with_worker_count(1).unwrap();
        let four_worker_backend = CpuBackend::try_with_worker_count(4).unwrap();
        let case = oracle_cases()[3];
        let mut single_worker = create_oracle_fixture(&single_worker_backend, case);
        let mut four_workers = create_oracle_fixture(&four_worker_backend, case);

        apply_fixture(&single_worker_backend, &mut single_worker).unwrap();
        apply_fixture(&four_worker_backend, &mut four_workers).unwrap();

        assert_eq!(
            single_worker.west_east_output,
            four_workers.west_east_output
        );
        assert_eq!(
            single_worker.south_north_output,
            four_workers.south_north_output
        );
        assert_eq!(single_worker.vertical_output, four_workers.vertical_output);
    }

    #[test]
    fn validates_every_output_role() {
        let backend = CpuBackend::try_with_worker_count(1).unwrap();
        let expected = GridShape::try_new(8, 7, 7).unwrap();
        let actual = GridShape::try_new(7, 7, 7).unwrap();
        let field = backend.create_field(actual, 0.0).unwrap();

        for role in [
            MoistureCoefficientField::WestEastOutput,
            MoistureCoefficientField::SouthNorthOutput,
            MoistureCoefficientField::VerticalOutput,
        ] {
            assert_eq!(
                validate_output_shape(&field, role, expected),
                Err(MoistureCoefficientError::OutputShapeMismatch {
                    field: role,
                    expected,
                    actual,
                })
            );
        }
    }

    #[test]
    fn rejects_output_and_species_shape_mismatches_before_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = oracle_cases()[3];
        let mut fixture = create_oracle_fixture(&backend, case);
        let wrong_shape = GridShape::try_new(7, 7, 7).unwrap();
        fixture.south_north_output = backend
            .create_field(wrong_shape, SENTINEL_SOUTH_NORTH)
            .unwrap();

        assert_eq!(
            apply_fixture(&backend, &mut fixture),
            Err(MoistureCoefficientError::OutputShapeMismatch {
                field: MoistureCoefficientField::SouthNorthOutput,
                expected: fixture.region.shape(),
                actual: wrong_shape,
            })
        );
        assert_all_outputs_retain_sentinels(&fixture);

        let mut fixture = create_oracle_fixture(&backend, case);
        fixture.active_species[2] = backend.create_field(wrong_shape, 0.0).unwrap();
        assert_eq!(
            apply_fixture(&backend, &mut fixture),
            Err(MoistureCoefficientError::SpeciesShapeMismatch {
                active_species_index: 2,
                expected: fixture.region.shape(),
                actual: wrong_shape,
            })
        );
        assert_all_outputs_retain_sentinels(&fixture);
    }

    #[test]
    fn dry_defaults_do_not_require_unread_horizontal_neighbors() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let shape = GridShape::try_new(4, 4, 4).unwrap();
        let region =
            MoistureCoefficientRegion::try_new(shape, 0..3, 0..3, 0..3, 0..4, 0..4, 0..4).unwrap();
        let mut west_east = backend.create_field(shape, -1.0).unwrap();
        let mut south_north = backend.create_field(shape, -1.0).unwrap();
        let mut vertical = backend.create_field(shape, -1.0).unwrap();

        backend
            .calculate_moisture_coefficients(
                MoistureCoefficientOutputs::new(&mut west_east, &mut south_north, &mut vertical),
                MoistureSpecies::new(&[]),
                &region,
            )
            .unwrap();

        assert_eq!(west_east.values()[0], 1.0);
        assert_eq!(south_north.values()[0], 1.0);
        assert_eq!(vertical.values()[volume_index(0, 1, 0, shape)], 0.0);
    }

    #[test]
    fn active_species_reject_missing_neighbors_before_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let shape = GridShape::try_new(4, 4, 4).unwrap();
        let mut fixture = OracleFixture {
            west_east_output: backend.create_field(shape, SENTINEL_WEST_EAST).unwrap(),
            south_north_output: backend.create_field(shape, SENTINEL_SOUTH_NORTH).unwrap(),
            vertical_output: backend.create_field(shape, SENTINEL_VERTICAL).unwrap(),
            active_species: vec![backend.create_field(shape, 0.01).unwrap()],
            region: MoistureCoefficientRegion::try_new(shape, 0..3, 1..3, 0..3, 0..4, 1..4, 0..4)
                .unwrap(),
        };

        assert_eq!(
            apply_fixture(&backend, &mut fixture),
            Err(MoistureCoefficientError::MissingLowerNeighbor {
                axis: crate::MoistureCoefficientAxis::WestEast,
                tile_start: 0,
            })
        );
        assert_all_outputs_retain_sentinels(&fixture);
    }

    #[derive(Clone, Copy)]
    struct OracleCase {
        name: &'static str,
        west_east_tile: (i32, i32),
        south_north_tile: (i32, i32),
        bottom_top_tile: (i32, i32),
        active_species_count: usize,
        exceptional: bool,
    }

    fn oracle_cases() -> [OracleCase; 7] {
        [
            OracleCase {
                name: "interior",
                west_east_tile: (0, 2),
                south_north_tile: (5, 6),
                bottom_top_tile: (1, 3),
                active_species_count: 3,
                exceptional: false,
            },
            OracleCase {
                name: "x_upper",
                west_east_tile: (-1, 4),
                south_north_tile: (5, 6),
                bottom_top_tile: (1, 3),
                active_species_count: 3,
                exceptional: false,
            },
            OracleCase {
                name: "y_upper",
                west_east_tile: (0, 2),
                south_north_tile: (4, 8),
                bottom_top_tile: (1, 3),
                active_species_count: 3,
                exceptional: false,
            },
            OracleCase {
                name: "all_upper",
                west_east_tile: (-1, 4),
                south_north_tile: (4, 8),
                bottom_top_tile: (1, 4),
                active_species_count: 3,
                exceptional: false,
            },
            OracleCase {
                name: "no_active_species",
                west_east_tile: (-1, 4),
                south_north_tile: (4, 8),
                bottom_top_tile: (1, 4),
                active_species_count: 0,
                exceptional: false,
            },
            OracleCase {
                name: "one_active_species",
                west_east_tile: (-1, 4),
                south_north_tile: (4, 8),
                bottom_top_tile: (1, 4),
                active_species_count: 1,
                exceptional: false,
            },
            OracleCase {
                name: "exceptional",
                west_east_tile: (-1, 4),
                south_north_tile: (4, 8),
                bottom_top_tile: (1, 4),
                active_species_count: 3,
                exceptional: true,
            },
        ]
    }

    struct OracleFixture {
        west_east_output: CpuField<f32>,
        south_north_output: CpuField<f32>,
        vertical_output: CpuField<f32>,
        active_species: Vec<CpuField<f32>>,
        region: MoistureCoefficientRegion,
    }

    fn create_oracle_fixture(backend: &CpuBackend, case: OracleCase) -> OracleFixture {
        let shape = GridShape::try_new(8, 7, 7).unwrap();
        let mut active_species = Vec::with_capacity(case.active_species_count);
        for active_species_index in 0..case.active_species_count {
            let fortran_species = (active_species_index + 2) as f32;
            let mut field = backend.create_field(shape, 0.0).unwrap();
            for south_north in 0..shape.south_north_points() {
                let logical_south_north = south_north as i32 + MEMORY_SOUTH_NORTH_START;
                for bottom_top in 0..shape.bottom_top_points() {
                    let logical_bottom_top = bottom_top as i32 + MEMORY_BOTTOM_TOP_START;
                    for west_east in 0..shape.west_east_points() {
                        let logical_west_east = west_east as i32 + MEMORY_WEST_EAST_START;
                        let value = ((fortran_species * 0.001 + logical_west_east as f32 * 0.0001)
                            - logical_bottom_top as f32 * 0.0002)
                            + logical_south_north as f32 * 0.00005;
                        field.values_mut()
                            [volume_index(west_east, bottom_top, south_north, shape)] = value;
                    }
                }
            }
            active_species.push(field);
        }
        if case.exceptional {
            set_exceptional_values(&mut active_species, shape);
        }

        OracleFixture {
            west_east_output: backend.create_field(shape, SENTINEL_WEST_EAST).unwrap(),
            south_north_output: backend.create_field(shape, SENTINEL_SOUTH_NORTH).unwrap(),
            vertical_output: backend.create_field(shape, SENTINEL_VERTICAL).unwrap(),
            active_species,
            region: MoistureCoefficientRegion::try_new(
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

    fn set_exceptional_values(active_species: &mut [CpuField<f32>], shape: GridShape) {
        set_logical_value(&mut active_species[0], -1, 1, 4, f32::MAX, shape);
        set_logical_value(&mut active_species[0], -2, 1, 4, f32::MAX, shape);
        set_logical_value(&mut active_species[1], -1, 1, 3, -f32::MAX, shape);
        set_logical_value(&mut active_species[2], -1, 2, 4, f32::MAX, shape);
        set_logical_value(&mut active_species[2], -1, 1, 5, -f32::MAX, shape);
        set_logical_value(&mut active_species[0], 0, 1, 4, -0.0, shape);
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

    fn apply_fixture(
        backend: &CpuBackend,
        fixture: &mut OracleFixture,
    ) -> MoistureCoefficientResult<()> {
        backend.calculate_moisture_coefficients(
            MoistureCoefficientOutputs::new(
                &mut fixture.west_east_output,
                &mut fixture.south_north_output,
                &mut fixture.vertical_output,
            ),
            MoistureSpecies::new(&fixture.active_species),
            &fixture.region,
        )
    }

    fn assert_all_outputs_retain_sentinels(fixture: &OracleFixture) {
        assert!(
            fixture
                .west_east_output
                .values()
                .iter()
                .all(|value| *value == SENTINEL_WEST_EAST)
        );
        assert!(
            fixture
                .south_north_output
                .values()
                .iter()
                .all(|value| *value == SENTINEL_SOUTH_NORTH)
        );
        assert!(
            fixture
                .vertical_output
                .values()
                .iter()
                .all(|value| *value == SENTINEL_VERTICAL)
        );
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

    struct ExpectedMoistureOutput<'a> {
        lines: std::str::Lines<'a>,
    }

    impl<'a> ExpectedMoistureOutput<'a> {
        fn new(output: &'a str) -> Self {
            Self {
                lines: output.lines(),
            }
        }

        fn assert_case(&mut self, case_name: &str, fixture: &OracleFixture) {
            for (field_name, field) in [
                ("cqu", &fixture.west_east_output),
                ("cqv", &fixture.south_north_output),
                ("cqw", &fixture.vertical_output),
            ] {
                for south_north in 0..7 {
                    for bottom_top in 0..7 {
                        for west_east in 0..8 {
                            let logical_west_east = west_east as i32 + MEMORY_WEST_EAST_START;
                            let logical_bottom_top = bottom_top as i32 + MEMORY_BOTTOM_TOP_START;
                            let logical_south_north = south_north as i32 + MEMORY_SOUTH_NORTH_START;
                            let actual = field.values()
                                [volume_index(west_east, bottom_top, south_north, field.shape())];
                            self.assert_next(
                                case_name,
                                field_name,
                                logical_west_east,
                                logical_bottom_top,
                                logical_south_north,
                                actual,
                            );
                        }
                    }
                }
            }
        }

        #[allow(clippy::too_many_arguments)]
        fn assert_next(
            &mut self,
            case_name: &str,
            field_name: &str,
            west_east: i32,
            bottom_top: i32,
            south_north: i32,
            actual: f32,
        ) {
            let line = self
                .lines
                .next()
                .unwrap_or_else(|| panic!("Fortran output ended before {case_name} {field_name}"));
            let columns: Vec<_> = line.split_whitespace().collect();
            assert_eq!(columns.len(), 6, "unexpected Fortran output: {line}");
            assert_eq!(columns[0], case_name, "unexpected case at {line}");
            assert_eq!(columns[1], field_name, "unexpected field at {line}");
            assert_eq!(columns[2].parse::<i32>().unwrap(), west_east);
            assert_eq!(columns[3].parse::<i32>().unwrap(), bottom_top);
            assert_eq!(columns[4].parse::<i32>().unwrap(), south_north);
            let expected_bits = u32::from_str_radix(columns[5], 16).unwrap();
            if f32::from_bits(expected_bits).is_nan() {
                assert!(actual.is_nan(), "expected NaN at {line}, got {actual:?}");
            } else {
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
}
