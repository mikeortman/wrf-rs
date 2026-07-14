use std::str::Lines;

use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};

use crate::{
    DryTendencyAssemblyCoefficient, DryTendencyAssemblyCoefficients, DryTendencyAssemblyError,
    DryTendencyAssemblyForwardTendencies, DryTendencyAssemblyKernels,
    DryTendencyAssemblyMapFactors, DryTendencyAssemblyPhase, DryTendencyAssemblyRegion,
    DryTendencyAssemblyRungeKuttaTendencies, DryTendencyAssemblySavedTendencies,
    DryTendencyAssemblyThermodynamics,
};

const VOLUME_FIELD_NAMES: [&str; 10] = [
    "ru_tend", "rv_tend", "rw_tend", "ph_tend", "t_tend", "ru_tendf", "rv_tendf", "rw_tendf",
    "ph_tendf", "t_tendf",
];

#[test]
fn matches_upstream_fortran_for_substeps_staggers_sentinels_and_ieee_values() {
    let backend = CpuBackend::try_with_worker_count(3).unwrap();
    let mut expected = ExpectedOutput::new(include_str!(
        "../../test-data/dry_tendency_assembly.out.correct"
    ));
    for case in oracle_cases() {
        let mut fixture = create_fixture(&backend, &case);
        apply_fixture(&backend, &mut fixture);
        for (field_name, field) in VOLUME_FIELD_NAMES.into_iter().zip(fixture.volume_outputs()) {
            expected.assert_volume(case.name, field_name, field);
        }
        expected.assert_horizontal(case.name, "mu_tend", &fixture.runge_kutta_column_mass);
        expected.assert_horizontal(case.name, "mu_tendf", &fixture.forward_column_mass);
    }
    expected.finish();
}

#[test]
fn validation_failure_is_atomic_across_every_mutable_output() {
    let backend = CpuBackend::try_with_worker_count(3).unwrap();
    let case = oracle_cases().into_iter().next().unwrap();
    let mut fixture = create_fixture(&backend, &case);
    fixture.full_mass_multiplier.pop();
    let before = mutable_output_bits(&fixture);

    let error = call_fixture(&backend, &mut fixture).unwrap_err();

    assert_eq!(
        error,
        DryTendencyAssemblyError::CoefficientLengthMismatch {
            coefficient: DryTendencyAssemblyCoefficient::FullMassMultiplier,
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

#[derive(Clone)]
struct OracleCase {
    name: &'static str,
    phase: DryTendencyAssemblyPhase,
    west_east_tile: std::ops::Range<usize>,
    south_north_tile: std::ops::Range<usize>,
    bottom_top_tile: std::ops::Range<usize>,
    exceptional: bool,
}

fn oracle_cases() -> [OracleCase; 3] {
    [
        OracleCase {
            name: "first",
            phase: DryTendencyAssemblyPhase::FirstSubstep,
            west_east_tile: 1..6,
            south_north_tile: 1..6,
            bottom_top_tile: 1..5,
            exceptional: false,
        },
        OracleCase {
            name: "later",
            phase: DryTendencyAssemblyPhase::LaterSubstep,
            west_east_tile: 2..4,
            south_north_tile: 2..4,
            bottom_top_tile: 1..3,
            exceptional: false,
        },
        OracleCase {
            name: "exceptional",
            phase: DryTendencyAssemblyPhase::FirstSubstep,
            west_east_tile: 1..6,
            south_north_tile: 1..6,
            bottom_top_tile: 1..5,
            exceptional: true,
        },
    ]
}

struct OracleFixture {
    runge_kutta_volume: [CpuField<f32>; 5],
    forward_volume: [CpuField<f32>; 5],
    saved_volume: [CpuField<f32>; 5],
    diabatic_heating: CpuField<f32>,
    runge_kutta_column_mass: CpuField<f32>,
    forward_column_mass: CpuField<f32>,
    full_column_mass: CpuField<f32>,
    map_factors: [CpuField<f32>; 4],
    full_mass_multiplier: Vec<f32>,
    vertical_offset: Vec<f32>,
    phase: DryTendencyAssemblyPhase,
    region: DryTendencyAssemblyRegion,
}

impl OracleFixture {
    fn volume_outputs(&self) -> [&CpuField<f32>; 10] {
        [
            &self.runge_kutta_volume[0],
            &self.runge_kutta_volume[1],
            &self.runge_kutta_volume[2],
            &self.runge_kutta_volume[3],
            &self.runge_kutta_volume[4],
            &self.forward_volume[0],
            &self.forward_volume[1],
            &self.forward_volume[2],
            &self.forward_volume[3],
            &self.forward_volume[4],
        ]
    }
}

fn create_fixture(backend: &CpuBackend, case: &OracleCase) -> OracleFixture {
    let shape = GridShape::try_new(6, 6, 5).unwrap();
    let horizontal_shape = shape.horizontal_shape();
    let volume = |value| backend.create_field(shape, value).unwrap();
    let horizontal = |value| backend.create_field(horizontal_shape, value).unwrap();
    let mut fixture = OracleFixture {
        runge_kutta_volume: [
            volume(0.0),
            volume(0.0),
            volume(0.0),
            volume(0.0),
            volume(0.0),
        ],
        forward_volume: [
            volume(0.0),
            volume(0.0),
            volume(0.0),
            volume(0.0),
            volume(0.0),
        ],
        saved_volume: [
            volume(0.0),
            volume(0.0),
            volume(0.0),
            volume(0.0),
            volume(0.0),
        ],
        diabatic_heating: volume(0.0),
        runge_kutta_column_mass: horizontal(0.0),
        forward_column_mass: horizontal(0.0),
        full_column_mass: horizontal(0.0),
        map_factors: [
            horizontal(0.0),
            horizontal(0.0),
            horizontal(0.0),
            horizontal(0.0),
        ],
        full_mass_multiplier: (0..5).map(|k| 0.2 + k as f32 * 0.03).collect(),
        vertical_offset: (0..5).map(|k| 0.4 - k as f32 * 0.02).collect(),
        phase: case.phase,
        region: DryTendencyAssemblyRegion::try_new(
            shape,
            1..5,
            1..5,
            1..4,
            case.west_east_tile.clone(),
            case.south_north_tile.clone(),
            case.bottom_top_tile.clone(),
        )
        .unwrap(),
    };

    for south_north in 0..6 {
        let j = south_north as f32;
        for west_east in 0..6 {
            let i = west_east as f32;
            let horizontal_index = south_north * 6 + west_east;
            fixture.runge_kutta_column_mass.values_mut()[horizontal_index] =
                0.6 + i * 0.07 - j * 0.03;
            fixture.forward_column_mass.values_mut()[horizontal_index] = -0.2 + i * 0.02 + j * 0.04;
            fixture.full_column_mass.values_mut()[horizontal_index] = 50.0 + i * 2.0 + j * 3.0;
            fixture.map_factors[0].values_mut()[horizontal_index] = 1.0 + i * 0.02 + j * 0.01;
            let south_north_factor = 0.9 + i * 0.015 - j * 0.005;
            fixture.map_factors[1].values_mut()[horizontal_index] = south_north_factor;
            fixture.map_factors[2].values_mut()[horizontal_index] = 1.0 / south_north_factor;
            fixture.map_factors[3].values_mut()[horizontal_index] = 1.1 + i * 0.01 + j * 0.02;
            for bottom_top in 0..5 {
                let k = bottom_top as f32;
                let index = (south_north * 5 + bottom_top) * 6 + west_east;
                fixture.runge_kutta_volume[0].values_mut()[index] =
                    1.0 + i * 0.11 + k * 0.07 - j * 0.03;
                fixture.runge_kutta_volume[1].values_mut()[index] =
                    2.0 - i * 0.05 + k * 0.09 + j * 0.02;
                fixture.runge_kutta_volume[2].values_mut()[index] =
                    -1.0 + i * 0.04 - k * 0.08 + j * 0.06;
                fixture.runge_kutta_volume[3].values_mut()[index] =
                    3.0 + i * 0.03 + k * 0.05 - j * 0.04;
                fixture.runge_kutta_volume[4].values_mut()[index] =
                    -2.0 + i * 0.02 + k * 0.06 + j * 0.01;
                fixture.forward_volume[0].values_mut()[index] =
                    0.3 + i * 0.013 - k * 0.017 + j * 0.019;
                fixture.forward_volume[1].values_mut()[index] =
                    -0.4 + i * 0.021 + k * 0.015 - j * 0.011;
                fixture.forward_volume[2].values_mut()[index] =
                    0.5 - i * 0.014 + k * 0.012 + j * 0.016;
                fixture.forward_volume[3].values_mut()[index] =
                    -0.6 + i * 0.018 - k * 0.013 + j * 0.009;
                fixture.forward_volume[4].values_mut()[index] =
                    0.7 - i * 0.012 + k * 0.014 - j * 0.008;
                fixture.saved_volume[0].values_mut()[index] =
                    0.09 + i * 0.003 + k * 0.002 - j * 0.001;
                fixture.saved_volume[1].values_mut()[index] =
                    -0.08 + i * 0.002 - k * 0.003 + j * 0.001;
                fixture.saved_volume[2].values_mut()[index] =
                    0.07 - i * 0.001 + k * 0.002 + j * 0.003;
                fixture.saved_volume[3].values_mut()[index] =
                    -0.06 + i * 0.004 - k * 0.001 + j * 0.002;
                fixture.saved_volume[4].values_mut()[index] =
                    0.05 + i * 0.002 + k * 0.003 - j * 0.004;
                fixture.diabatic_heating.values_mut()[index] =
                    0.001 + i * 0.0001 + k * 0.0002 + j * 0.0003;
            }
        }
    }
    if case.exceptional {
        fixture.map_factors[3].values_mut()[7] = 0.0;
        fixture.map_factors[0].values_mut()[8] = -0.0;
        fixture.map_factors[2].values_mut()[13] = f32::INFINITY;
        fixture.diabatic_heating.values_mut()[(2 * 5 + 1) * 6 + 2] = f32::INFINITY;
        fixture.forward_volume[0].values_mut()[(5 + 1) * 6 + 2] = -0.0;
        fixture.forward_volume[3].values_mut()[(5 + 1) * 6 + 1] = f32::MAX;
    }
    fixture
}

fn apply_fixture(backend: &CpuBackend, fixture: &mut OracleFixture) {
    call_fixture(backend, fixture).unwrap();
}

fn call_fixture(
    backend: &CpuBackend,
    fixture: &mut OracleFixture,
) -> crate::DryTendencyAssemblyResult<()> {
    let [ru, rv, rw, ph, t] = &mut fixture.runge_kutta_volume;
    let [ruf, rvf, rwf, phf, tf] = &mut fixture.forward_volume;
    backend.assemble_dry_tendencies(
        DryTendencyAssemblyRungeKuttaTendencies::new(
            ru,
            rv,
            rw,
            ph,
            t,
            &mut fixture.runge_kutta_column_mass,
        ),
        DryTendencyAssemblyForwardTendencies::new(
            ruf,
            rvf,
            rwf,
            phf,
            tf,
            &fixture.forward_column_mass,
        ),
        DryTendencyAssemblySavedTendencies::new(
            &fixture.saved_volume[0],
            &fixture.saved_volume[1],
            &fixture.saved_volume[2],
            &fixture.saved_volume[3],
            &fixture.saved_volume[4],
        ),
        DryTendencyAssemblyThermodynamics::new(
            &fixture.diabatic_heating,
            &fixture.full_column_mass,
        ),
        DryTendencyAssemblyMapFactors::new(
            &fixture.map_factors[0],
            &fixture.map_factors[1],
            &fixture.map_factors[2],
            &fixture.map_factors[3],
        ),
        DryTendencyAssemblyCoefficients::new(
            &fixture.full_mass_multiplier,
            &fixture.vertical_offset,
        ),
        fixture.phase,
        &fixture.region,
    )
}

fn mutable_output_bits(fixture: &OracleFixture) -> Vec<Vec<u32>> {
    fixture
        .volume_outputs()
        .into_iter()
        .chain([
            &fixture.runge_kutta_column_mass,
            &fixture.forward_column_mass,
        ])
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
        for south_north in 0..6 {
            for bottom_top in 0..5 {
                for west_east in 0..6 {
                    let index = (south_north * 5 + bottom_top) * 6 + west_east;
                    self.assert_next(
                        case,
                        name,
                        &[west_east, bottom_top, south_north],
                        field.values()[index],
                    );
                }
            }
        }
    }

    fn assert_horizontal(&mut self, case: &str, name: &str, field: &CpuField<f32>) {
        for south_north in 0..6 {
            for west_east in 0..6 {
                self.assert_next(
                    case,
                    name,
                    &[west_east, south_north],
                    field.values()[south_north * 6 + west_east],
                );
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
