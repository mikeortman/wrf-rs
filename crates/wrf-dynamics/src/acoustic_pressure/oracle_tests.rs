use std::str::Lines;

use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};

use crate::{
    AcousticPressureCoefficient, AcousticPressureCoefficients, AcousticPressureDampingPhase,
    AcousticPressureError, AcousticPressureKernels, AcousticPressureMasses, AcousticPressureMode,
    AcousticPressureParameters, AcousticPressureRegion, AcousticPressureState,
    AcousticPressureThermodynamics, AcousticPressureVerticalMetrics,
};

#[test]
fn matches_upstream_fortran_for_modes_damping_recurrence_sentinels_and_ieee_values() {
    let backend = CpuBackend::try_with_worker_count(3).unwrap();
    let mut expected = ExpectedOutput::new(include_str!(
        "../../test-data/acoustic_pressure.out.correct"
    ));
    for case in oracle_cases() {
        let mut fixture = create_fixture(&backend, &case);
        apply_fixture(&backend, &mut fixture);
        for (name, field) in ["al", "p", "ph", "pm1"]
            .into_iter()
            .zip(fixture.mutable_fields())
        {
            expected.assert_volume(case.name, name, field);
        }
    }
    expected.finish();
}

#[test]
fn validation_failure_is_atomic_across_all_mutable_fields() {
    let backend = CpuBackend::try_with_worker_count(3).unwrap();
    let case = oracle_cases().into_iter().next().unwrap();
    let mut fixture = create_fixture(&backend, &case);
    fixture.coefficients[0].pop();
    let before = mutable_bits(&fixture);

    let error = call_fixture(&backend, &mut fixture).unwrap_err();

    assert_eq!(
        error,
        AcousticPressureError::CoefficientLengthMismatch {
            coefficient: AcousticPressureCoefficient::HalfLevelMassMultiplier,
            expected: 6,
            actual: 5,
        }
    );
    assert_eq!(mutable_bits(&fixture), before);
}

#[test]
fn one_and_four_workers_produce_identical_bits_for_both_modes() {
    let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
    let four_workers = CpuBackend::try_with_worker_count(4).unwrap();
    for case in oracle_cases() {
        let mut serial = create_fixture(&one_worker, &case);
        let mut parallel = create_fixture(&four_workers, &case);
        apply_fixture(&one_worker, &mut serial);
        apply_fixture(&four_workers, &mut parallel);
        assert_eq!(mutable_bits(&serial), mutable_bits(&parallel));
    }
}

struct OracleCase {
    name: &'static str,
    mode: AcousticPressureMode,
    damping_phase: AcousticPressureDampingPhase,
    west_east_tile: std::ops::Range<usize>,
    south_north_tile: std::ops::Range<usize>,
    half_level_tile: std::ops::Range<usize>,
    exceptional: bool,
}

fn oracle_cases() -> [OracleCase; 4] {
    [
        OracleCase {
            name: "nonhydro_init",
            mode: AcousticPressureMode::Nonhydrostatic,
            damping_phase: AcousticPressureDampingPhase::Initialize,
            west_east_tile: 1..6,
            south_north_tile: 1..6,
            half_level_tile: 1..6,
            exceptional: false,
        },
        OracleCase {
            name: "nonhydro_advance",
            mode: AcousticPressureMode::Nonhydrostatic,
            damping_phase: AcousticPressureDampingPhase::Advance,
            west_east_tile: 2..4,
            south_north_tile: 2..4,
            half_level_tile: 2..4,
            exceptional: true,
        },
        OracleCase {
            name: "hydro_init",
            mode: AcousticPressureMode::Hydrostatic,
            damping_phase: AcousticPressureDampingPhase::Initialize,
            west_east_tile: 1..6,
            south_north_tile: 1..6,
            half_level_tile: 1..6,
            exceptional: false,
        },
        OracleCase {
            name: "hydro_advance",
            mode: AcousticPressureMode::Hydrostatic,
            damping_phase: AcousticPressureDampingPhase::Advance,
            west_east_tile: 2..4,
            south_north_tile: 2..4,
            half_level_tile: 2..4,
            exceptional: true,
        },
    ]
}

struct Fixture {
    state: [CpuField<f32>; 4],
    thermodynamics: [CpuField<f32>; 4],
    masses: [CpuField<f32>; 2],
    coefficients: [Vec<f32>; 3],
    vertical_metrics: [Vec<f32>; 2],
    mode: AcousticPressureMode,
    damping_phase: AcousticPressureDampingPhase,
    region: AcousticPressureRegion,
}

impl Fixture {
    fn mutable_fields(&self) -> [&CpuField<f32>; 4] {
        [
            &self.state[0],
            &self.state[1],
            &self.state[2],
            &self.state[3],
        ]
    }
}

fn create_fixture(backend: &CpuBackend, case: &OracleCase) -> Fixture {
    let shape = GridShape::try_new(6, 6, 6).unwrap();
    let horizontal_shape = shape.horizontal_shape();
    let volume = |value| backend.create_field(shape, value).unwrap();
    let horizontal = |value| backend.create_field(horizontal_shape, value).unwrap();
    let mut fixture = Fixture {
        state: [volume(-999.0), volume(-999.0), volume(0.0), volume(0.0)],
        thermodynamics: [volume(0.0), volume(0.0), volume(0.0), volume(0.0)],
        masses: [horizontal(0.0), horizontal(0.0)],
        coefficients: [
            (0..6).map(|k| 0.2 + k as f32 * 0.03).collect(),
            (0..6).map(|k| 0.4 - k as f32 * 0.02).collect(),
            (0..6).map(|k| 1.1 + k as f32 * 0.04).collect(),
        ],
        vertical_metrics: [
            (0..6).map(|k| 1.3 + k as f32 * 0.05).collect(),
            (0..6).map(|k| 0.7 - k as f32 * 0.025).collect(),
        ],
        mode: case.mode,
        damping_phase: case.damping_phase,
        region: AcousticPressureRegion::try_new(
            shape,
            1..5,
            1..5,
            1..5,
            case.west_east_tile.clone(),
            case.south_north_tile.clone(),
            case.half_level_tile.clone(),
        )
        .unwrap(),
    };
    for y in 0..6 {
        let j = y as f32;
        for x in 0..6 {
            let i = x as f32;
            let horizontal_index = y * 6 + x;
            fixture.masses[0].values_mut()[horizontal_index] = 1.0 + i * 0.11 - j * 0.04;
            fixture.masses[1].values_mut()[horizontal_index] = 40.0 + i * 1.3 + j * 0.7;
            for z in 0..6 {
                let k = z as f32;
                let index = (y * 6 + z) * 6 + x;
                fixture.state[2].values_mut()[index] = 900.0 + i * 3.0 + k * 5.0 - j * 4.0;
                fixture.thermodynamics[0].values_mut()[index] =
                    0.8 + i * 0.01 + k * 0.02 + j * 0.015;
                fixture.thermodynamics[1].values_mut()[index] =
                    2.0 + i * 0.09 - k * 0.05 + j * 0.02;
                fixture.thermodynamics[2].values_mut()[index] =
                    1.0 + i * 0.07 + k * 0.03 - j * 0.01;
                fixture.thermodynamics[3].values_mut()[index] =
                    140_000.0 + i * 13.0 + k * 17.0 + j * 11.0;
                fixture.state[3].values_mut()[index] = 500.0 + i * 3.0 - k * 2.0 + j * 4.0;
            }
        }
    }
    if case.exceptional {
        fixture.coefficients[0][2] = 0.0;
        fixture.coefficients[1][2] = 0.0;
        fixture.thermodynamics[2].values_mut()[volume_index(3, 2, 2)] = -300.0;
        fixture.thermodynamics[3].values_mut()[volume_index(2, 2, 2)] = 0.0;
        fixture.state[2].values_mut()[volume_index(2, 3, 2)] = f32::INFINITY;
        fixture.state[3].values_mut()[volume_index(3, 3, 3)] = -f32::MAX;
    }
    fixture
}

const fn volume_index(x: usize, k: usize, y: usize) -> usize {
    (y * 6 + k) * 6 + x
}

fn apply_fixture(backend: &CpuBackend, fixture: &mut Fixture) {
    call_fixture(backend, fixture).unwrap();
}

fn call_fixture(backend: &CpuBackend, fixture: &mut Fixture) -> crate::AcousticPressureResult<()> {
    let [inverse_density, pressure, geopotential, previous_pressure] = &mut fixture.state;
    backend.update_acoustic_pressure(
        AcousticPressureState::new(inverse_density, pressure, geopotential, previous_pressure),
        AcousticPressureThermodynamics::new(
            &fixture.thermodynamics[0],
            &fixture.thermodynamics[1],
            &fixture.thermodynamics[2],
            &fixture.thermodynamics[3],
        ),
        AcousticPressureMasses::new(&fixture.masses[0], &fixture.masses[1]),
        AcousticPressureCoefficients::new(
            &fixture.coefficients[0],
            &fixture.coefficients[1],
            &fixture.coefficients[2],
        ),
        AcousticPressureVerticalMetrics::new(
            &fixture.vertical_metrics[0],
            &fixture.vertical_metrics[1],
        ),
        AcousticPressureParameters::new(300.0, 0.17),
        fixture.mode,
        fixture.damping_phase,
        &fixture.region,
    )
}

fn mutable_bits(fixture: &Fixture) -> Vec<Vec<u32>> {
    fixture
        .mutable_fields()
        .into_iter()
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

    fn assert_volume(&mut self, case: &str, name: &str, field: &CpuField<f32>) {
        for y in 0..6 {
            for k in 0..6 {
                for x in 0..6 {
                    self.assert_next(case, name, x, k, y, field.values()[volume_index(x, k, y)]);
                }
            }
        }
    }

    fn assert_next(&mut self, case: &str, name: &str, x: usize, k: usize, y: usize, actual: f32) {
        let line = self
            .lines
            .next()
            .unwrap_or_else(|| panic!("Fortran output ended before {case} {name}"));
        let columns: Vec<_> = line.split_whitespace().collect();
        assert_eq!(columns[0], case, "unexpected case at {line}");
        assert_eq!(columns[1], name, "unexpected field at {line}");
        assert_eq!(
            columns[2].parse::<usize>().unwrap(),
            x,
            "unexpected x at {line}"
        );
        assert_eq!(
            columns[3].parse::<usize>().unwrap(),
            k,
            "unexpected k at {line}"
        );
        assert_eq!(
            columns[4].parse::<usize>().unwrap(),
            y,
            "unexpected y at {line}"
        );
        let expected = columns[5];
        if expected == "NAN" {
            assert!(actual.is_nan(), "expected NaN at {line}, got {actual:?}");
        } else {
            assert_eq!(
                actual.to_bits(),
                u32::from_str_radix(expected, 16).unwrap(),
                "value mismatch at {line}: got {:08X}",
                actual.to_bits()
            );
        }
    }

    fn finish(mut self) {
        assert!(self.lines.next().is_none(), "unused Fortran output remains");
    }
}
