use std::ops::Range;

use wrf_compute::{ComputeBackend, CpuBackend, CpuField, FieldStorage, GridShape};

use crate::{
    DryBoundaryTendencyBoundaryFields, DryBoundaryTendencyError, DryBoundaryTendencyTarget,
    DryTendencyAssemblyCoefficients, DryTendencyAssemblyForwardTendencies,
    DryTendencyAssemblyMapFactors, DryTendencyAssemblyPhase,
    DryTendencyAssemblyRungeKuttaTendencies, DryTendencyAssemblySavedTendencies,
    DryTendencyAssemblyThermodynamics, SpecifiedBoundaryTendencies, SpecifiedBoundaryTendencyError,
    SpecifiedBoundaryTendencyParameters, SpecifiedBoundaryWestEastPeriodicity,
};

use super::*;

const BOUNDARY_WIDTH: usize = 2;
const VOLUME_NAMES: [&str; 10] = [
    "ru_tend", "rv_tend", "rw_tend", "ph_tend", "t_tend", "ru_tendf", "rv_tendf", "rw_tendf",
    "ph_tendf", "t_tendf",
];

#[test]
fn matches_direct_pinned_fortran_for_complete_coupled_outputs() {
    let backend = CpuBackend::try_with_worker_count(3).unwrap();
    let mut expected = ExpectedOutput::new(include_str!(
        "../../test-data/dry_tendency_boundary_stage.out.correct"
    ));

    for case in oracle_cases() {
        let mut fixture = create_fixture(&backend, &case);
        call_fixture(&backend, &mut fixture).unwrap();
        for (name, field) in VOLUME_NAMES.into_iter().zip(fixture.volume_outputs()) {
            expected.assert_field(case.name, name, field);
        }
        expected.assert_field(case.name, "mu_tend", &fixture.runge_kutta_column_mass);
        expected.assert_field(case.name, "mu_tendf", &fixture.forward_column_mass);
    }
    expected.finish();
}

#[test]
fn late_boundary_validation_failure_is_atomic_across_both_routines() {
    let backend = CpuBackend::try_with_worker_count(2).unwrap();
    let mut fixture = create_fixture(&backend, &oracle_cases()[1]);
    fixture.boundaries[4].fields[3] = backend
        .create_field(GridShape::try_new(2, 2, 2).unwrap(), 0.0)
        .unwrap();
    let before = mutable_output_bits(&fixture);

    let error = call_fixture(&backend, &mut fixture).unwrap_err();

    assert!(matches!(
        error,
        DryTendencyBoundaryStageError::BoundaryAssignment(
            DryBoundaryTendencyError::SpecifiedTendency {
                target: DryBoundaryTendencyTarget::VerticalMomentum,
                source: SpecifiedBoundaryTendencyError::BoundaryShapeMismatch { .. },
            }
        )
    ));
    assert_eq!(mutable_output_bits(&fixture), before);
}

#[test]
fn complete_stage_is_bitwise_deterministic_across_worker_counts() {
    let serial_backend = CpuBackend::try_with_worker_count(1).unwrap();
    let parallel_backend = CpuBackend::try_with_worker_count(4).unwrap();

    for case in oracle_cases() {
        let mut serial = create_fixture(&serial_backend, &case);
        let mut parallel = create_fixture(&parallel_backend, &case);
        call_fixture(&serial_backend, &mut serial).unwrap();
        call_fixture(&parallel_backend, &mut parallel).unwrap();
        assert_eq!(
            mutable_output_bits(&serial),
            mutable_output_bits(&parallel),
            "{}",
            case.name
        );
    }
}

#[derive(Clone)]
struct OracleCase {
    name: &'static str,
    phase: DryTendencyAssemblyPhase,
    nested: bool,
    periodicity: SpecifiedBoundaryWestEastPeriodicity,
    west_east_tile: Range<usize>,
    south_north_tile: Range<usize>,
    bottom_top_tile: Range<usize>,
    exceptional: bool,
}

fn oracle_cases() -> [OracleCase; 5] {
    [
        oracle_case(
            "first_global",
            DryTendencyAssemblyPhase::FirstSubstep,
            false,
            false,
            1..6,
            1..6,
            1..5,
            false,
        ),
        oracle_case(
            "first_nested",
            DryTendencyAssemblyPhase::FirstSubstep,
            true,
            false,
            1..6,
            1..6,
            1..5,
            false,
        ),
        oracle_case(
            "later_partial",
            DryTendencyAssemblyPhase::LaterSubstep,
            true,
            false,
            2..5,
            2..5,
            1..4,
            false,
        ),
        oracle_case(
            "periodic_nested",
            DryTendencyAssemblyPhase::FirstSubstep,
            true,
            true,
            1..6,
            1..6,
            1..5,
            false,
        ),
        oracle_case(
            "exceptional",
            DryTendencyAssemblyPhase::FirstSubstep,
            true,
            false,
            1..6,
            1..6,
            1..5,
            true,
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn oracle_case(
    name: &'static str,
    phase: DryTendencyAssemblyPhase,
    nested: bool,
    periodic: bool,
    west_east_tile: Range<usize>,
    south_north_tile: Range<usize>,
    bottom_top_tile: Range<usize>,
    exceptional: bool,
) -> OracleCase {
    OracleCase {
        name,
        phase,
        nested,
        periodicity: if periodic {
            SpecifiedBoundaryWestEastPeriodicity::Periodic
        } else {
            SpecifiedBoundaryWestEastPeriodicity::Nonperiodic
        },
        west_east_tile,
        south_north_tile,
        bottom_top_tile,
        exceptional,
    }
}

struct BoundarySet {
    fields: [CpuField<f32>; 4],
}

impl BoundarySet {
    fn references(&self) -> SpecifiedBoundaryTendencies<'_, CpuField<f32>> {
        SpecifiedBoundaryTendencies::new(
            &self.fields[0],
            &self.fields[1],
            &self.fields[2],
            &self.fields[3],
        )
    }
}

struct Fixture {
    case: OracleCase,
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
    boundaries: [BoundarySet; 6],
    regions: DryTendencyBoundaryStageRegions,
}

impl Fixture {
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

fn create_fixture(backend: &CpuBackend, case: &OracleCase) -> Fixture {
    let shape = GridShape::try_new(6, 6, 5).unwrap();
    let horizontal_shape = shape.horizontal_shape();
    let volume = |value| backend.create_field(shape, value).unwrap();
    let horizontal = |value| backend.create_field(horizontal_shape, value).unwrap();
    let mut fixture = Fixture {
        case: case.clone(),
        runge_kutta_volume: std::array::from_fn(|_| volume(0.0)),
        forward_volume: std::array::from_fn(|_| volume(0.0)),
        saved_volume: std::array::from_fn(|_| volume(0.0)),
        diabatic_heating: volume(0.0),
        runge_kutta_column_mass: horizontal(0.0),
        forward_column_mass: horizontal(0.0),
        full_column_mass: horizontal(0.0),
        map_factors: std::array::from_fn(|_| horizontal(0.0)),
        full_mass_multiplier: (0..5).map(|k| 0.2 + k as f32 * 0.03).collect(),
        vertical_offset: (0..5).map(|k| 0.4 - k as f32 * 0.02).collect(),
        boundaries: [
            boundary_set(backend, 1, false),
            boundary_set(backend, 2, false),
            boundary_set(backend, 3, false),
            boundary_set(backend, 4, false),
            boundary_set(backend, 5, false),
            boundary_set(backend, 6, true),
        ],
        regions: DryTendencyBoundaryStageRegions::try_new(
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

    initialize_assembly_fields(&mut fixture);
    if case.exceptional {
        fixture.map_factors[3].values_mut()[7] = 0.0;
        fixture.map_factors[0].values_mut()[8] = -0.0;
        fixture.map_factors[2].values_mut()[13] = f32::INFINITY;
        fixture.diabatic_heating.values_mut()[(2 * 5 + 1) * 6 + 2] = f32::INFINITY;
        fixture.forward_volume[0].values_mut()[(5 + 1) * 6 + 2] = -0.0;
        fixture.forward_volume[3].values_mut()[(5 + 1) * 6 + 1] = f32::MAX;
        set_boundary_bits(&mut fixture.boundaries[0].fields[2], 2, 0, 0, 0x8000_0000);
        set_boundary_bits(&mut fixture.boundaries[1].fields[3], 2, 0, 0, 0x7f80_0000);
        set_boundary_bits(&mut fixture.boundaries[2].fields[0], 2, 0, 0, 0xff80_0000);
        set_boundary_bits(&mut fixture.boundaries[3].fields[1], 2, 0, 0, 0x0000_0001);
        set_boundary_bits(&mut fixture.boundaries[4].fields[2], 3, 0, 0, 0x7fc1_2345);
        set_boundary_bits(&mut fixture.boundaries[5].fields[3], 2, 0, 0, 0x7f7f_ffff);
    }
    fixture
}

fn initialize_assembly_fields(fixture: &mut Fixture) {
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
            let factor = 0.9 + i * 0.015 - j * 0.005;
            fixture.map_factors[1].values_mut()[horizontal_index] = factor;
            fixture.map_factors[2].values_mut()[horizontal_index] = 1.0 / factor;
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
}

fn boundary_set(backend: &CpuBackend, field: usize, horizontal: bool) -> BoundarySet {
    let vertical_points = if horizontal { 1 } else { 4 };
    let shape = GridShape::try_new(6, BOUNDARY_WIDTH, vertical_points).unwrap();
    BoundarySet {
        fields: std::array::from_fn(|side| {
            initialized_field(backend, shape, |line, vertical, distance| {
                field as f32 * 1000.0
                    + (side + 1) as f32 * 100.0
                    + line * 10.0
                    + if horizontal { 0.0 } else { vertical + 1.0 }
                    + (distance + 1.0) * 0.01
            })
        }),
    }
}

fn initialized_field(
    backend: &CpuBackend,
    shape: GridShape,
    value: impl Fn(f32, f32, f32) -> f32,
) -> CpuField<f32> {
    let mut field = backend.create_field(shape, 0.0).unwrap();
    for distance in 0..shape.south_north_points() {
        for vertical in 0..shape.bottom_top_points() {
            for line in 0..shape.west_east_points() {
                let index = line
                    + shape.west_east_points() * (vertical + shape.bottom_top_points() * distance);
                field.values_mut()[index] = value(line as f32, vertical as f32, distance as f32);
            }
        }
    }
    field
}

fn set_boundary_bits(
    field: &mut CpuField<f32>,
    line: usize,
    vertical: usize,
    distance: usize,
    bits: u32,
) {
    let shape = field.shape();
    let index = line + shape.west_east_points() * (vertical + shape.bottom_top_points() * distance);
    field.values_mut()[index] = f32::from_bits(bits);
}

fn call_fixture(backend: &CpuBackend, fixture: &mut Fixture) -> DryTendencyBoundaryStageResult<()> {
    let [ru, rv, rw, ph, t] = &mut fixture.runge_kutta_volume;
    let [ruf, rvf, rwf, phf, tf] = &mut fixture.forward_volume;
    let boundaries = DryBoundaryTendencyBoundaryFields::new(
        fixture.boundaries[0].references(),
        fixture.boundaries[1].references(),
        fixture.boundaries[2].references(),
        fixture.boundaries[3].references(),
        fixture.boundaries[5].references(),
    );
    let vertical = if fixture.case.nested {
        DryTendencyBoundaryStageVertical::Nested {
            boundaries: fixture.boundaries[4].references(),
        }
    } else {
        DryTendencyBoundaryStageVertical::Global
    };
    backend.advance_dry_tendency_boundary_stage(
        DryTendencyAssemblyRungeKuttaTendencies::new(
            ru,
            rv,
            rw,
            ph,
            t,
            &mut fixture.runge_kutta_column_mass,
        ),
        DryTendencyBoundaryStageInputs::new(
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
            boundaries,
        ),
        vertical,
        DryTendencyBoundaryStageControls::new(
            fixture.case.phase,
            SpecifiedBoundaryTendencyParameters::new(BOUNDARY_WIDTH, 1),
            fixture.case.periodicity,
        ),
        &fixture.regions,
    )
}

fn mutable_output_bits(fixture: &Fixture) -> Vec<Vec<u32>> {
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
    lines: std::str::Lines<'a>,
}

impl<'a> ExpectedOutput<'a> {
    fn new(output: &'a str) -> Self {
        Self {
            lines: output.lines(),
        }
    }

    fn assert_field(&mut self, case: &str, name: &str, field: &CpuField<f32>) {
        for (index, actual) in field.values().iter().copied().enumerate() {
            let line = self.lines.next().expect("missing oracle output");
            let mut parts = line.split_whitespace();
            assert_eq!(parts.next(), Some(case), "line {index}");
            assert_eq!(parts.next(), Some(name), "line {index}");
            let expected = parts.next().unwrap();
            assert!(parts.next().is_none());
            if expected == "NAN" {
                assert!(actual.is_nan(), "{case} {name} index {index}");
            } else {
                assert_eq!(
                    actual.to_bits(),
                    u32::from_str_radix(expected, 16).unwrap(),
                    "{case} {name} index {index}"
                );
            }
        }
    }

    fn finish(mut self) {
        assert!(self.lines.next().is_none(), "unconsumed oracle output");
    }
}
