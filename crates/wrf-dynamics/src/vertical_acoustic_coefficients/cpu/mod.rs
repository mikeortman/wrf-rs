mod forward_elimination;
mod lower_diagonal;
mod validation;

use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, ParallelExecutionError};

use crate::{
    VerticalAcousticCoefficientError, VerticalAcousticCoefficientInputs,
    VerticalAcousticCoefficientKernels, VerticalAcousticCoefficientParameters,
    VerticalAcousticCoefficientRegion, VerticalAcousticCoefficientResult,
    VerticalAcousticMassCoefficients, VerticalAcousticMetrics, VerticalAcousticSolveCoefficients,
};

use self::forward_elimination::ForwardElimination;
use self::lower_diagonal::LowerDiagonalCalculation;

impl VerticalAcousticCoefficientKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn calculate_vertical_acoustic_coefficients(
        &self,
        outputs: VerticalAcousticSolveCoefficients<'_, Self::Field>,
        inputs: VerticalAcousticCoefficientInputs<'_, Self::Field>,
        mass_coefficients: VerticalAcousticMassCoefficients<'_>,
        metrics: VerticalAcousticMetrics<'_>,
        parameters: VerticalAcousticCoefficientParameters,
        region: &VerticalAcousticCoefficientRegion,
    ) -> VerticalAcousticCoefficientResult<()> {
        validation::validate_operation(&outputs, inputs, mass_coefficients, metrics, region)?;
        LowerDiagonalCalculation::new(inputs, mass_coefficients, metrics, parameters, region)
            .calculate(self, outputs.lower_diagonal)?;
        ForwardElimination::new(
            &*outputs.lower_diagonal,
            inputs,
            mass_coefficients,
            metrics,
            parameters,
            region,
        )
        .calculate(
            self,
            outputs.inverse_eliminated_diagonal,
            outputs.upper_elimination_factor,
        )
    }
}

pub(crate) fn validate_vertical_acoustic_coefficients(
    outputs: &VerticalAcousticSolveCoefficients<'_, CpuField<f32>>,
    inputs: VerticalAcousticCoefficientInputs<'_, CpuField<f32>>,
    mass_coefficients: VerticalAcousticMassCoefficients<'_>,
    metrics: VerticalAcousticMetrics<'_>,
    region: &VerticalAcousticCoefficientRegion,
) -> VerticalAcousticCoefficientResult<()> {
    validation::validate_operation(outputs, inputs, mass_coefficients, metrics, region)
}

pub(super) fn map_parallel_error(
    error: ParallelExecutionError<Infallible>,
) -> VerticalAcousticCoefficientError {
    match error {
        ParallelExecutionError::WorkerPanicked => VerticalAcousticCoefficientError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. }
        | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
            VerticalAcousticCoefficientError::SchedulerContractViolated
        }
        ParallelExecutionError::Kernel(unreachable) => match unreachable {},
    }
}

#[cfg(test)]
mod tests {
    use std::str::Lines;

    use wrf_compute::{ComputeBackend, FieldStorage, GridShape};

    use super::*;
    use crate::{
        VerticalAcousticCoefficient, VerticalAcousticMassCoefficients, VerticalAcousticMetrics,
        VerticalAcousticTopBoundary,
    };

    const MEMORY_WEST_EAST_START: i32 = -1;
    const MEMORY_BOTTOM_TOP_START: i32 = -1;
    const MEMORY_SOUTH_NORTH_START: i32 = 0;

    #[test]
    fn matches_fortran_for_boundaries_partial_tiles_sentinels_and_ieee_values() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let mut expected = ExpectedOutput::new(include_str!(
            "../../../test-data/vertical_acoustic_coefficients.out.correct"
        ));

        for case in oracle_cases() {
            let mut fixture = create_fixture(&backend, case);
            apply_fixture(&backend, &mut fixture);
            expected.assert_case(case.name, &fixture.outputs);
        }
        expected.finish();
    }

    #[test]
    fn coefficient_failure_is_atomic_across_all_outputs() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let mut fixture = create_fixture(&backend, oracle_cases()[0]);
        fixture.mass_coefficients[0].pop();
        let before = output_bits(&fixture.outputs);

        let error = call_fixture(&backend, &mut fixture).unwrap_err();

        assert_eq!(
            error,
            VerticalAcousticCoefficientError::CoefficientLengthMismatch {
                coefficient: VerticalAcousticCoefficient::HalfLevelMassMultiplier,
                expected: 7,
                actual: 6,
            }
        );
        assert_eq!(output_bits(&fixture.outputs), before);
    }

    #[test]
    fn one_and_four_workers_produce_identical_bits() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();

        for case in oracle_cases() {
            let mut serial = create_fixture(&one_worker, case);
            let mut parallel = create_fixture(&four_workers, case);
            apply_fixture(&one_worker, &mut serial);
            apply_fixture(&four_workers, &mut parallel);
            assert_eq!(output_bits(&serial.outputs), output_bits(&parallel.outputs));
        }
    }

    #[derive(Clone, Copy)]
    struct OracleCase {
        name: &'static str,
        top_boundary: VerticalAcousticTopBoundary,
        west_east_tile: (i32, i32),
        south_north_tile: (i32, i32),
        exceptional: bool,
    }

    fn oracle_cases() -> [OracleCase; 4] {
        [
            OracleCase {
                name: "open_full",
                top_boundary: VerticalAcousticTopBoundary::Nonrigid,
                west_east_tile: (0, 4),
                south_north_tile: (1, 5),
                exceptional: false,
            },
            OracleCase {
                name: "rigid_full",
                top_boundary: VerticalAcousticTopBoundary::RigidLid,
                west_east_tile: (0, 4),
                south_north_tile: (1, 5),
                exceptional: false,
            },
            OracleCase {
                name: "open_partial",
                top_boundary: VerticalAcousticTopBoundary::Nonrigid,
                west_east_tile: (1, 2),
                south_north_tile: (2, 3),
                exceptional: true,
            },
            OracleCase {
                name: "rigid_partial",
                top_boundary: VerticalAcousticTopBoundary::RigidLid,
                west_east_tile: (1, 2),
                south_north_tile: (2, 3),
                exceptional: true,
            },
        ]
    }

    struct Fixture {
        outputs: [CpuField<f32>; 3],
        full_column_mass: CpuField<f32>,
        vertical_moisture_coefficient: CpuField<f32>,
        pressure_coefficient: CpuField<f32>,
        mass_coefficients: [Vec<f32>; 4],
        metrics: [Vec<f32>; 2],
        parameters: VerticalAcousticCoefficientParameters,
        region: VerticalAcousticCoefficientRegion,
    }

    fn create_fixture(backend: &CpuBackend, case: OracleCase) -> Fixture {
        let shape = GridShape::try_new(6, 6, 7).unwrap();
        let volume = |value| backend.create_field(shape, value).unwrap();
        let mut fixture = Fixture {
            outputs: [volume(-901.0), volume(-902.0), volume(-903.0)],
            full_column_mass: backend.create_field(shape.horizontal_shape(), 0.0).unwrap(),
            vertical_moisture_coefficient: volume(0.0),
            pressure_coefficient: volume(0.0),
            mass_coefficients: [
                coefficient_values(|level| 0.2 + level * 0.03),
                coefficient_values(|level| 0.4 - level * 0.02),
                coefficient_values(|level| 0.25 + level * 0.015),
                coefficient_values(|level| 0.35 - level * 0.01),
            ],
            metrics: [
                coefficient_values(|level| 1.1 + level * 0.04),
                coefficient_values(|level| 1.3 + level * 0.05),
            ],
            parameters: VerticalAcousticCoefficientParameters::new(
                2.5,
                9.81,
                0.1,
                case.top_boundary,
            ),
            region: VerticalAcousticCoefficientRegion::try_new(
                shape,
                logical_range(0, 3, MEMORY_WEST_EAST_START),
                logical_range(1, 4, MEMORY_SOUTH_NORTH_START),
                logical_range(1, 3, MEMORY_BOTTOM_TOP_START),
                logical_range(
                    case.west_east_tile.0,
                    case.west_east_tile.1,
                    MEMORY_WEST_EAST_START,
                ),
                logical_range(
                    case.south_north_tile.0,
                    case.south_north_tile.1,
                    MEMORY_SOUTH_NORTH_START,
                ),
            )
            .unwrap(),
        };
        initialize_fields(&mut fixture, shape);
        if case.exceptional {
            set_exceptional_values(&mut fixture, shape);
        }
        fixture
    }

    fn coefficient_values(operation: impl Fn(f32) -> f32) -> Vec<f32> {
        (0..7)
            .map(|position| operation((position + MEMORY_BOTTOM_TOP_START) as f32))
            .collect()
    }

    fn initialize_fields(fixture: &mut Fixture, shape: GridShape) {
        for south_north in 0..shape.south_north_points() {
            let logical_south_north = (south_north as i32 + MEMORY_SOUTH_NORTH_START) as f32;
            for west_east in 0..shape.west_east_points() {
                let logical_west_east = (west_east as i32 + MEMORY_WEST_EAST_START) as f32;
                fixture.full_column_mass.values_mut()
                    [south_north * shape.west_east_points() + west_east] =
                    40.0 + logical_west_east * 1.3 + logical_south_north * 0.7;
                for bottom_top in 0..shape.bottom_top_points() {
                    let logical_bottom_top = (bottom_top as i32 + MEMORY_BOTTOM_TOP_START) as f32;
                    let index = volume_index(west_east, bottom_top, south_north, shape);
                    fixture.vertical_moisture_coefficient.values_mut()[index] =
                        0.9 + logical_west_east * 0.01 - logical_bottom_top * 0.02
                            + logical_south_north * 0.015;
                    fixture.pressure_coefficient.values_mut()[index] = 140_000.0
                        + logical_west_east * 13.0
                        + logical_bottom_top * 17.0
                        + logical_south_north * 11.0;
                }
            }
        }
    }

    fn set_exceptional_values(fixture: &mut Fixture, shape: GridShape) {
        fixture.mass_coefficients[0][logical_position(2, MEMORY_BOTTOM_TOP_START)] = 0.0;
        fixture.mass_coefficients[1][logical_position(2, MEMORY_BOTTOM_TOP_START)] = 0.0;
        fixture.mass_coefficients[2][logical_position(3, MEMORY_BOTTOM_TOP_START)] = 0.0;
        fixture.mass_coefficients[3][logical_position(3, MEMORY_BOTTOM_TOP_START)] = 0.0;
        set_logical_value(
            &mut fixture.vertical_moisture_coefficient,
            1,
            2,
            2,
            -0.0,
            shape,
        );
        set_logical_value(&mut fixture.pressure_coefficient, 2, 2, 2, 0.0, shape);
        set_logical_value(
            &mut fixture.pressure_coefficient,
            1,
            3,
            2,
            f32::INFINITY,
            shape,
        );
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
            logical_position(west_east, MEMORY_WEST_EAST_START),
            logical_position(bottom_top, MEMORY_BOTTOM_TOP_START),
            logical_position(south_north, MEMORY_SOUTH_NORTH_START),
            shape,
        );
        field.values_mut()[index] = value;
    }

    fn apply_fixture(backend: &CpuBackend, fixture: &mut Fixture) {
        call_fixture(backend, fixture).unwrap();
    }

    fn call_fixture(
        backend: &CpuBackend,
        fixture: &mut Fixture,
    ) -> VerticalAcousticCoefficientResult<()> {
        let [lower, inverse_diagonal, upper] = &mut fixture.outputs;
        backend.calculate_vertical_acoustic_coefficients(
            VerticalAcousticSolveCoefficients::new(lower, inverse_diagonal, upper),
            VerticalAcousticCoefficientInputs::new(
                &fixture.full_column_mass,
                &fixture.vertical_moisture_coefficient,
                &fixture.pressure_coefficient,
            ),
            VerticalAcousticMassCoefficients::new(
                &fixture.mass_coefficients[0],
                &fixture.mass_coefficients[1],
                &fixture.mass_coefficients[2],
                &fixture.mass_coefficients[3],
            ),
            VerticalAcousticMetrics::new(&fixture.metrics[0], &fixture.metrics[1]),
            fixture.parameters,
            &fixture.region,
        )
    }

    fn logical_range(start: i32, inclusive_end: i32, memory_start: i32) -> std::ops::Range<usize> {
        logical_position(start, memory_start)..logical_position(inclusive_end, memory_start) + 1
    }

    fn logical_position(logical_index: i32, memory_start: i32) -> usize {
        (logical_index - memory_start) as usize
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

    fn output_bits(outputs: &[CpuField<f32>; 3]) -> Vec<Vec<u32>> {
        outputs
            .iter()
            .map(|field| field.values().iter().map(|value| value.to_bits()).collect())
            .collect()
    }

    struct ExpectedOutput<'a> {
        lines: Lines<'a>,
    }

    impl<'a> ExpectedOutput<'a> {
        fn new(output: &'a str) -> Self {
            Self {
                lines: output.lines(),
            }
        }

        fn assert_case(&mut self, case_name: &str, outputs: &[CpuField<f32>; 3]) {
            for (field_name, field) in ["a", "alpha", "gamma"].into_iter().zip(outputs) {
                let shape = field.shape();
                for south_north in 0..shape.south_north_points() {
                    for bottom_top in 0..shape.bottom_top_points() {
                        for west_east in 0..shape.west_east_points() {
                            let actual = field.values()
                                [volume_index(west_east, bottom_top, south_north, shape)];
                            self.assert_next(
                                case_name,
                                field_name,
                                west_east,
                                bottom_top,
                                south_north,
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
            west_east: usize,
            bottom_top: usize,
            south_north: usize,
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
            assert_eq!(columns[2].parse::<usize>().unwrap(), west_east);
            assert_eq!(columns[3].parse::<usize>().unwrap(), bottom_top);
            assert_eq!(columns[4].parse::<usize>().unwrap(), south_north);
            if columns[5] == "NAN" {
                assert!(actual.is_nan(), "expected NaN at {line}, got {actual:?}");
            } else {
                let expected_bits = u32::from_str_radix(columns[5], 16).unwrap();
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
