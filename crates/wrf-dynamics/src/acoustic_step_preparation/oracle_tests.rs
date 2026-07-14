use std::str::Lines;

use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};

use crate::{
    AcousticStepPreparationCoefficient, AcousticStepPreparationCoefficients,
    AcousticStepPreparationColumnMassTimeLevels, AcousticStepPreparationDiagnosticInputs,
    AcousticStepPreparationError, AcousticStepPreparationKernels,
    AcousticStepPreparationMapFactors, AcousticStepPreparationMassInputs,
    AcousticStepPreparationMassOutputs, AcousticStepPreparationPhase,
    AcousticStepPreparationRegion, AcousticStepPreparationSavedOutputs,
    AcousticStepPreparationVolumeTimeLevels,
};

#[test]
fn validation_failure_is_atomic_across_all_twenty_four_mutable_fields() {
    let backend = CpuBackend::try_with_worker_count(3).unwrap();
    let case = oracle_cases().into_iter().next().unwrap();
    let mut fixture = create_fixture(&backend, &case);
    fixture.coefficients[0].pop();
    let before = mutable_output_bits(&fixture);

    let error = call_fixture(&backend, &mut fixture).unwrap_err();

    assert_eq!(
        error,
        AcousticStepPreparationError::CoefficientLengthMismatch {
            coefficient: AcousticStepPreparationCoefficient::HalfLevelMassMultiplier,
            expected: 5,
            actual: 4,
        }
    );
    assert_eq!(mutable_output_bits(&fixture), before);
}

#[test]
fn one_and_four_workers_produce_identical_bits() {
    let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
    let four_workers = CpuBackend::try_with_worker_count(4).unwrap();
    let case = oracle_cases().into_iter().next().unwrap();
    let mut serial = create_fixture(&one_worker, &case);
    let mut parallel = create_fixture(&four_workers, &case);

    apply_fixture(&one_worker, &mut serial);
    apply_fixture(&four_workers, &mut parallel);

    assert_eq!(mutable_output_bits(&serial), mutable_output_bits(&parallel));
}

const VOLUME_OUTPUT_NAMES: [&str; 17] = [
    "u1", "u2", "v1", "v2", "w1", "w2", "t1", "t2", "ph1", "ph2", "us", "vs", "ws", "ts", "phs",
    "wws", "c2a",
];
const HORIZONTAL_OUTPUT_NAMES: [&str; 7] = ["mu1", "mu2", "muus", "muvs", "muts", "mudf", "mus"];

#[test]
fn matches_upstream_fortran_for_phases_staggers_sentinels_and_ieee_values() {
    let backend = CpuBackend::try_with_worker_count(3).unwrap();
    let mut expected = ExpectedOutput::new(include_str!(
        "../../test-data/acoustic_step_preparation.out.correct"
    ));
    for case in oracle_cases() {
        let mut fixture = create_fixture(&backend, &case);
        apply_fixture(&backend, &mut fixture);
        for (name, field) in VOLUME_OUTPUT_NAMES
            .into_iter()
            .zip(fixture.volume_outputs())
        {
            expected.assert_volume(case.name, name, field);
        }
        for (name, field) in HORIZONTAL_OUTPUT_NAMES
            .into_iter()
            .zip(fixture.horizontal_outputs())
        {
            expected.assert_horizontal(case.name, name, field);
        }
    }
    expected.finish();
}

struct OracleCase {
    name: &'static str,
    phase: AcousticStepPreparationPhase,
    west_east_tile: std::ops::Range<usize>,
    south_north_tile: std::ops::Range<usize>,
    exceptional: bool,
}

fn oracle_cases() -> [OracleCase; 3] {
    [
        OracleCase {
            name: "first",
            phase: AcousticStepPreparationPhase::FirstSubstep,
            west_east_tile: 1..6,
            south_north_tile: 1..6,
            exceptional: false,
        },
        OracleCase {
            name: "later",
            phase: AcousticStepPreparationPhase::LaterSubstep,
            west_east_tile: 2..4,
            south_north_tile: 2..4,
            exceptional: false,
        },
        OracleCase {
            name: "exceptional",
            phase: AcousticStepPreparationPhase::FirstSubstep,
            west_east_tile: 1..6,
            south_north_tile: 1..6,
            exceptional: true,
        },
    ]
}

struct Fixture {
    previous: [CpuField<f32>; 5],
    current: [CpuField<f32>; 5],
    saved_volume: [CpuField<f32>; 5],
    saved_omega: CpuField<f32>,
    pressure_coefficient: CpuField<f32>,
    diagnostics: [CpuField<f32>; 4],
    previous_mass: CpuField<f32>,
    current_mass: CpuField<f32>,
    mass_inputs: [CpuField<f32>; 4],
    mass_outputs: [CpuField<f32>; 4],
    saved_mass: CpuField<f32>,
    maps: [CpuField<f32>; 3],
    coefficients: [Vec<f32>; 4],
    phase: AcousticStepPreparationPhase,
    region: AcousticStepPreparationRegion,
}

impl Fixture {
    fn volume_outputs(&self) -> [&CpuField<f32>; 17] {
        [
            &self.previous[0],
            &self.current[0],
            &self.previous[1],
            &self.current[1],
            &self.previous[2],
            &self.current[2],
            &self.previous[3],
            &self.current[3],
            &self.previous[4],
            &self.current[4],
            &self.saved_volume[0],
            &self.saved_volume[1],
            &self.saved_volume[2],
            &self.saved_volume[3],
            &self.saved_volume[4],
            &self.saved_omega,
            &self.pressure_coefficient,
        ]
    }

    fn horizontal_outputs(&self) -> [&CpuField<f32>; 7] {
        [
            &self.previous_mass,
            &self.current_mass,
            &self.mass_outputs[0],
            &self.mass_outputs[1],
            &self.mass_outputs[2],
            &self.mass_outputs[3],
            &self.saved_mass,
        ]
    }
}

fn create_fixture(backend: &CpuBackend, case: &OracleCase) -> Fixture {
    let shape = GridShape::try_new(6, 6, 5).unwrap();
    let horizontal_shape = shape.horizontal_shape();
    let volume = |value| backend.create_field(shape, value).unwrap();
    let horizontal = |value| backend.create_field(horizontal_shape, value).unwrap();
    let sentinel = -999.0;
    let mut fixture = Fixture {
        previous: [
            volume(0.0),
            volume(0.0),
            volume(0.0),
            volume(0.0),
            volume(0.0),
        ],
        current: [
            volume(0.0),
            volume(0.0),
            volume(0.0),
            volume(0.0),
            volume(0.0),
        ],
        saved_volume: [
            volume(sentinel),
            volume(sentinel),
            volume(sentinel),
            volume(sentinel),
            volume(sentinel),
        ],
        saved_omega: volume(sentinel),
        pressure_coefficient: volume(sentinel),
        diagnostics: [volume(0.0), volume(0.0), volume(0.0), volume(0.0)],
        previous_mass: horizontal(0.0),
        current_mass: horizontal(0.0),
        mass_inputs: [
            horizontal(0.0),
            horizontal(0.0),
            horizontal(0.0),
            horizontal(0.0),
        ],
        mass_outputs: [
            horizontal(sentinel),
            horizontal(sentinel),
            horizontal(sentinel),
            horizontal(sentinel),
        ],
        saved_mass: horizontal(sentinel),
        maps: [horizontal(0.0), horizontal(0.0), horizontal(0.0)],
        coefficients: [
            (0..5).map(|k| 0.2 + k as f32 * 0.03).collect(),
            (0..5).map(|k| 0.4 - k as f32 * 0.02).collect(),
            (0..5).map(|k| 0.3 + k as f32 * 0.025).collect(),
            (0..5).map(|k| 0.5 - k as f32 * 0.015).collect(),
        ],
        phase: case.phase,
        region: AcousticStepPreparationRegion::try_new(
            shape,
            1..5,
            1..5,
            1..4,
            case.west_east_tile.clone(),
            case.south_north_tile.clone(),
            1..5,
        )
        .unwrap(),
    };
    for y in 0..6 {
        let j = y as f32;
        for x in 0..6 {
            let i = x as f32;
            let horizontal_index = y * 6 + x;
            fixture.mass_inputs[0].values_mut()[horizontal_index] = 40.0 + i * 1.3 + j * 0.7;
            fixture.previous_mass.values_mut()[horizontal_index] = 1.0 + i * 0.11 - j * 0.04;
            fixture.current_mass.values_mut()[horizontal_index] = -0.5 + i * 0.07 + j * 0.03;
            fixture.mass_inputs[1].values_mut()[horizontal_index] = 42.0 + i * 1.1 + j * 0.6;
            fixture.mass_inputs[2].values_mut()[horizontal_index] = 43.0 + i * 0.9 + j * 0.8;
            fixture.mass_inputs[3].values_mut()[horizontal_index] = 44.0 + i * 1.2 + j * 0.5;
            fixture.maps[0].values_mut()[horizontal_index] = 1.0 + i * 0.02 + j * 0.01;
            let v_map = 0.9 + i * 0.015 - j * 0.005;
            fixture.maps[1].values_mut()[horizontal_index] = 1.0 / v_map;
            fixture.maps[2].values_mut()[horizontal_index] = 1.1 + i * 0.01 + j * 0.02;
            for z in 0..5 {
                let k = z as f32;
                let index = (y * 5 + z) * 6 + x;
                fixture.previous[0].values_mut()[index] = 1.0 + i * 0.11 + k * 0.07 - j * 0.03;
                fixture.current[0].values_mut()[index] = 0.8 + i * 0.09 - k * 0.05 + j * 0.02;
                fixture.previous[1].values_mut()[index] = 2.0 - i * 0.05 + k * 0.09 + j * 0.02;
                fixture.current[1].values_mut()[index] = 1.6 + i * 0.04 + k * 0.03 - j * 0.01;
                fixture.previous[2].values_mut()[index] = -1.0 + i * 0.04 - k * 0.08 + j * 0.06;
                fixture.current[2].values_mut()[index] = -0.7 - i * 0.02 + k * 0.06 + j * 0.03;
                fixture.previous[3].values_mut()[index] = 300.0 + i * 0.2 + k * 0.6 + j * 0.1;
                fixture.current[3].values_mut()[index] = 299.0 - i * 0.1 + k * 0.4 + j * 0.2;
                fixture.previous[4].values_mut()[index] = 1000.0 + i * 3.0 + k * 5.0 - j * 4.0;
                fixture.current[4].values_mut()[index] = 900.0 - i * 2.0 + k * 4.0 + j * 3.0;
                fixture.diagnostics[0].values_mut()[index] =
                    80_000.0 + i * 11.0 + k * 17.0 + j * 13.0;
                fixture.diagnostics[1].values_mut()[index] = 500.0 + i * 3.0 - k * 2.0 + j * 4.0;
                fixture.diagnostics[2].values_mut()[index] = 0.8 + i * 0.01 + k * 0.02 + j * 0.015;
                fixture.diagnostics[3].values_mut()[index] =
                    0.3 + i * 0.013 - k * 0.017 + j * 0.019;
            }
        }
    }
    if case.exceptional {
        fixture.diagnostics[2].values_mut()[(5 + 1) * 6 + 1] = 0.0;
        fixture.maps[0].values_mut()[8] = -0.0;
        fixture.maps[1].values_mut()[13] = f32::INFINITY;
        fixture.maps[2].values_mut()[14] = 0.0;
        fixture.diagnostics[1].values_mut()[(5 + 1) * 6 + 3] = f32::MAX;
        fixture.diagnostics[0].values_mut()[(5 + 1) * 6 + 3] = f32::MAX;
    }
    fixture
}

fn apply_fixture(backend: &CpuBackend, fixture: &mut Fixture) {
    call_fixture(backend, fixture).unwrap();
}

fn call_fixture(
    backend: &CpuBackend,
    fixture: &mut Fixture,
) -> crate::AcousticStepPreparationResult<()> {
    let [u1, v1, w1, t1, ph1] = &mut fixture.previous;
    let [u2, v2, w2, t2, ph2] = &mut fixture.current;
    let [us, vs, ws, ts, phs] = &mut fixture.saved_volume;
    let [mub, muu, muv, mutf] = &fixture.mass_inputs;
    let [muus, muvs, muts, mudf] = &mut fixture.mass_outputs;
    let [pb, p, alt, ww] = &fixture.diagnostics;
    let [msfuy, msfvxi, msfty] = &fixture.maps;
    backend.prepare_acoustic_step(
        AcousticStepPreparationVolumeTimeLevels::new(u1, u2, v1, v2, w1, w2, t1, t2, ph1, ph2),
        AcousticStepPreparationColumnMassTimeLevels::new(
            &mut fixture.previous_mass,
            &mut fixture.current_mass,
        ),
        AcousticStepPreparationSavedOutputs::new(
            us,
            vs,
            ws,
            ts,
            phs,
            &mut fixture.saved_mass,
            &mut fixture.saved_omega,
            &mut fixture.pressure_coefficient,
        ),
        AcousticStepPreparationMassOutputs::new(muus, muvs, muts, mudf),
        AcousticStepPreparationMassInputs::new(mub, muu, muv, mutf),
        AcousticStepPreparationDiagnosticInputs::new(pb, p, alt, ww),
        AcousticStepPreparationMapFactors::new(msfuy, msfvxi, msfty),
        AcousticStepPreparationCoefficients::new(
            &fixture.coefficients[0],
            &fixture.coefficients[1],
            &fixture.coefficients[2],
            &fixture.coefficients[3],
        ),
        fixture.phase,
        &fixture.region,
    )
}

fn mutable_output_bits(fixture: &Fixture) -> Vec<Vec<u32>> {
    fixture
        .volume_outputs()
        .into_iter()
        .chain(fixture.horizontal_outputs())
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
            for z in 0..5 {
                for x in 0..6 {
                    self.assert_next(case, name, &[x, z, y], field.values()[(y * 5 + z) * 6 + x]);
                }
            }
        }
    }
    fn assert_horizontal(&mut self, case: &str, name: &str, field: &CpuField<f32>) {
        for y in 0..6 {
            for x in 0..6 {
                self.assert_next(case, name, &[x, y], field.values()[y * 6 + x]);
            }
        }
    }
    fn assert_next(&mut self, case: &str, name: &str, coordinates: &[usize], actual: f32) {
        let line = self
            .lines
            .next()
            .unwrap_or_else(|| panic!("Fortran output ended before {case} {name}"));
        let columns: Vec<_> = line.split_whitespace().collect();
        assert_eq!(columns[0], case, "unexpected case at {line}");
        assert_eq!(columns[1], name, "unexpected field at {line}");
        for (column, coordinate) in columns[2..2 + coordinates.len()].iter().zip(coordinates) {
            assert_eq!(
                column.parse::<usize>().unwrap(),
                *coordinate,
                "unexpected coordinate at {line}"
            );
        }
        let expected = columns.last().unwrap();
        if *expected == "NAN" {
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
