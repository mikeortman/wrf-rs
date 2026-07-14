use std::ops::Range;

use wrf_compute::{ComputeBackend, CpuBackend, CpuField, FieldStorage, GridShape};

use crate::{
    DryBoundaryRelaxationError, DryBoundaryRelaxationMassCoefficients, DryBoundaryRelaxationState,
    DryBoundaryRelaxationTarget, DryBoundaryRelaxationWorkspace, DryBoundaryTendencyBoundaryFields,
    DryBoundaryTendencyError, DryBoundaryTendencyTarget, DryTendencyAssemblyCoefficients,
    DryTendencyAssemblyForwardTendencies, DryTendencyAssemblyMapFactors,
    DryTendencyAssemblyRungeKuttaTendencies, DryTendencyAssemblyThermodynamics,
    SpecifiedBoundaryRelaxationBoundaryValues, SpecifiedBoundaryRelaxationCoefficients,
    SpecifiedBoundaryRelaxationParameters, SpecifiedBoundaryTendencies,
    SpecifiedBoundaryTendencyParameters, SpecifiedBoundaryWestEastPeriodicity,
};

use super::*;

const BOUNDARY_WIDTH: usize = 4;
const BOUNDARY_TIME_OFFSET: f32 = 0.25;
const VOLUME_NAMES: [&str; 15] = [
    "ru_tend", "rv_tend", "rw_tend", "ph_tend", "t_tend", "ru_tendf", "rv_tendf", "rw_tendf",
    "ph_tendf", "t_tendf", "u_save", "v_save", "w_save", "ph_save", "t_save",
];

#[test]
fn matches_direct_pinned_fortran_for_complete_three_stage_outputs() {
    let backend = CpuBackend::try_with_worker_count(3).unwrap();
    let mut expected = ExpectedOutput::new(include_str!(
        "../../test-data/dry_large_step_boundary_stage.out.correct"
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
fn late_nested_boundary_failure_is_atomic_across_all_three_stages() {
    let backend = CpuBackend::try_with_worker_count(2).unwrap();
    let mut fixture = create_fixture(&backend, &oracle_cases()[1]);
    fixture.boundaries[4].fields[7] = backend
        .create_field(GridShape::try_new(2, 2, 2).unwrap(), 0.0)
        .unwrap();
    let before = mutable_output_bits(&fixture);

    let error = call_fixture(&backend, &mut fixture).unwrap_err();

    assert!(matches!(
        error,
        DryLargeStepBoundaryStageError::Relaxation(
            DryBoundaryRelaxationError::SpecifiedRelaxation {
                target: DryBoundaryRelaxationTarget::VerticalMomentum,
                ..
            }
        )
    ));
    assert_eq!(mutable_output_bits(&fixture), before);
}

#[test]
fn later_substep_boundary_failure_is_atomic_without_relaxation() {
    let backend = CpuBackend::try_with_worker_count(2).unwrap();
    let mut fixture = create_fixture(&backend, &oracle_cases()[2]);
    fixture.boundaries[4].fields[7] = backend
        .create_field(GridShape::try_new(2, 2, 2).unwrap(), 0.0)
        .unwrap();
    let before = mutable_output_bits(&fixture);

    let error = call_fixture(&backend, &mut fixture).unwrap_err();

    assert!(matches!(
        error,
        DryLargeStepBoundaryStageError::BoundaryAssignment(
            DryBoundaryTendencyError::SpecifiedTendency {
                target: DryBoundaryTendencyTarget::VerticalMomentum,
                ..
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

#[test]
fn inactive_and_empty_relaxation_bands_leave_the_workspace_untouched() {
    let backend = CpuBackend::try_with_worker_count(2).unwrap();
    for case in [oracle_cases()[6].clone(), oracle_cases()[7].clone()] {
        let name = case.name;
        let mut fixture = create_fixture(&backend, &case);
        let workspace_before = fixture.workspace.clone();

        call_fixture(&backend, &mut fixture).unwrap();

        assert_eq!(fixture.workspace, workspace_before, "{name}");
    }
}

#[derive(Clone)]
struct OracleCase {
    name: &'static str,
    first_substep: bool,
    nested: bool,
    periodicity: SpecifiedBoundaryWestEastPeriodicity,
    west_east_tile: Range<usize>,
    south_north_tile: Range<usize>,
    bottom_top_tile: Range<usize>,
    specified_zone_width: usize,
    relaxation_zone_width: usize,
    exceptional: bool,
}

fn oracle_cases() -> [OracleCase; 9] {
    [
        oracle_case(
            "first_global",
            true,
            false,
            false,
            1..9,
            1..9,
            1..6,
            1,
            3,
            false,
        ),
        oracle_case(
            "first_nested",
            true,
            true,
            false,
            1..9,
            1..9,
            1..6,
            1,
            3,
            false,
        ),
        oracle_case(
            "later_nested",
            false,
            true,
            false,
            1..9,
            1..9,
            1..5,
            1,
            3,
            false,
        ),
        oracle_case(
            "periodic_nested",
            true,
            true,
            true,
            1..9,
            1..9,
            1..6,
            1,
            3,
            false,
        ),
        oracle_case(
            "south_west",
            true,
            true,
            false,
            1..6,
            1..6,
            1..6,
            1,
            3,
            false,
        ),
        oracle_case(
            "north_east",
            true,
            true,
            false,
            4..9,
            4..9,
            1..6,
            1,
            3,
            false,
        ),
        oracle_case("inactive", true, true, false, 4..6, 4..6, 1..6, 1, 3, false),
        oracle_case(
            "empty_band",
            true,
            true,
            false,
            1..9,
            1..9,
            1..6,
            2,
            2,
            false,
        ),
        oracle_case(
            "exceptional",
            true,
            true,
            false,
            1..9,
            1..9,
            1..6,
            1,
            3,
            true,
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn oracle_case(
    name: &'static str,
    first_substep: bool,
    nested: bool,
    periodic: bool,
    west_east_tile: Range<usize>,
    south_north_tile: Range<usize>,
    bottom_top_tile: Range<usize>,
    specified_zone_width: usize,
    relaxation_zone_width: usize,
    exceptional: bool,
) -> OracleCase {
    OracleCase {
        name,
        first_substep,
        nested,
        periodicity: if periodic {
            SpecifiedBoundaryWestEastPeriodicity::Periodic
        } else {
            SpecifiedBoundaryWestEastPeriodicity::Nonperiodic
        },
        west_east_tile,
        south_north_tile,
        bottom_top_tile,
        specified_zone_width,
        relaxation_zone_width,
        exceptional,
    }
}

struct BoundarySet {
    fields: [CpuField<f32>; 8],
}

impl BoundarySet {
    fn values(&self) -> SpecifiedBoundaryRelaxationBoundaryValues<'_, CpuField<f32>> {
        SpecifiedBoundaryRelaxationBoundaryValues::new(
            &self.fields[0],
            &self.fields[1],
            &self.fields[2],
            &self.fields[3],
        )
    }

    fn tendencies(&self) -> SpecifiedBoundaryTendencies<'_, CpuField<f32>> {
        SpecifiedBoundaryTendencies::new(
            &self.fields[4],
            &self.fields[5],
            &self.fields[6],
            &self.fields[7],
        )
    }
}

struct Fixture {
    case: OracleCase,
    runge_kutta_volume: [CpuField<f32>; 5],
    forward_volume: [CpuField<f32>; 5],
    saved_volume: [CpuField<f32>; 5],
    state_volume: [CpuField<f32>; 5],
    diabatic_heating: CpuField<f32>,
    runge_kutta_column_mass: CpuField<f32>,
    forward_column_mass: CpuField<f32>,
    perturbation_column_mass: CpuField<f32>,
    full_column_mass: CpuField<f32>,
    map_factors: [CpuField<f32>; 4],
    half_level_multiplier: Vec<f32>,
    half_level_offset: Vec<f32>,
    full_level_multiplier: Vec<f32>,
    full_level_offset: Vec<f32>,
    forcing: Vec<f32>,
    smoothing: Vec<f32>,
    boundaries: [BoundarySet; 6],
    workspace: CpuField<f32>,
    regions: DryLargeStepBoundaryStageRegions,
}

impl Fixture {
    fn volume_outputs(&self) -> [&CpuField<f32>; 15] {
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
            &self.saved_volume[0],
            &self.saved_volume[1],
            &self.saved_volume[2],
            &self.saved_volume[3],
            &self.saved_volume[4],
        ]
    }
}

fn create_fixture(backend: &CpuBackend, case: &OracleCase) -> Fixture {
    let shape = GridShape::try_new(10, 10, 6).unwrap();
    let horizontal_shape = shape.horizontal_shape();
    let volume = |value| backend.create_field(shape, value).unwrap();
    let horizontal = |value| backend.create_field(horizontal_shape, value).unwrap();
    let regions = DryLargeStepBoundaryStageRegions::try_new(
        shape,
        1..9,
        1..9,
        1..5,
        case.west_east_tile.clone(),
        case.south_north_tile.clone(),
        case.bottom_top_tile.clone(),
    )
    .unwrap();
    let mut fixture = Fixture {
        case: case.clone(),
        runge_kutta_volume: std::array::from_fn(|_| volume(0.0)),
        forward_volume: std::array::from_fn(|_| volume(0.0)),
        saved_volume: std::array::from_fn(|_| volume(0.0)),
        state_volume: std::array::from_fn(|_| volume(0.0)),
        diabatic_heating: volume(0.0),
        runge_kutta_column_mass: horizontal(0.0),
        forward_column_mass: horizontal(0.0),
        perturbation_column_mass: horizontal(0.0),
        full_column_mass: horizontal(0.0),
        map_factors: std::array::from_fn(|_| horizontal(0.0)),
        half_level_multiplier: (0..6).map(|k| 0.2 + k as f32 * 0.03).collect(),
        half_level_offset: (0..6).map(|k| 0.4 - k as f32 * 0.02).collect(),
        full_level_multiplier: (0..6).map(|k| 0.55 + k as f32 * 0.015625).collect(),
        full_level_offset: (0..6).map(|k| 0.45 - k as f32 * 0.0078125).collect(),
        forcing: vec![0.0, 0.7, 0.4, 0.0],
        smoothing: vec![0.0, 0.1, 0.05, 0.0],
        boundaries: [
            boundary_set(backend, 100.0, false),
            boundary_set(backend, 200.0, false),
            boundary_set(backend, 300.0, false),
            boundary_set(backend, 400.0, false),
            boundary_set(backend, 500.0, false),
            boundary_set(backend, 600.0, true),
        ],
        workspace: backend
            .create_field(regions.relaxation_workspace_shape(), 0.0)
            .unwrap(),
        regions,
    };

    initialize_prognostic_fields(&mut fixture);
    if case.exceptional {
        set_boundary_bits(&mut fixture.boundaries[2].fields[2], 4, 0, 1, 0x7f80_0000);
        set_boundary_bits(&mut fixture.boundaries[2].fields[6], 4, 0, 1, 0xff80_0000);
        set_boundary_bits(&mut fixture.boundaries[3].fields[3], 4, 0, 1, 0x8000_0000);
        set_boundary_bits(&mut fixture.boundaries[4].fields[4], 4, 0, 1, 0x0000_0001);
        set_boundary_bits(&mut fixture.boundaries[3].fields[6], 3, 0, 0, 0x7fc1_2345);
        set_boundary_bits(&mut fixture.boundaries[5].fields[7], 2, 0, 0, 0x7f7f_ffff);
        fixture.full_column_mass.values_mut()[14] = f32::from_bits(0x7f7f_ffff);
        fixture.map_factors[3].values_mut()[11] = 0.0;
        fixture.map_factors[0].values_mut()[12] = -0.0;
        fixture.map_factors[2].values_mut()[21] = f32::INFINITY;
        fixture.diabatic_heating.values_mut()[132] = f32::INFINITY;
        fixture.forward_volume[0].values_mut()[72] = -0.0;
        fixture.forward_volume[3].values_mut()[71] = f32::MAX;
    }
    fixture
}

fn initialize_prognostic_fields(fixture: &mut Fixture) {
    for south_north in 0..10 {
        let j = south_north as f32;
        for west_east in 0..10 {
            let i = west_east as f32;
            let horizontal_index = south_north * 10 + west_east;
            fixture.runge_kutta_column_mass.values_mut()[horizontal_index] =
                0.6 + i * 0.07 - j * 0.03;
            fixture.forward_column_mass.values_mut()[horizontal_index] = -0.2 + i * 0.02 + j * 0.04;
            fixture.perturbation_column_mass.values_mut()[horizontal_index] =
                60.0 + i * 0.25 - j * 0.125;
            fixture.full_column_mass.values_mut()[horizontal_index] = 50.0 + i * 2.0 + j * 3.0;
            fixture.map_factors[0].values_mut()[horizontal_index] = 1.0 + i * 0.02 + j * 0.01;
            let factor = 0.9 + i * 0.015 - j * 0.005;
            fixture.map_factors[1].values_mut()[horizontal_index] = factor;
            fixture.map_factors[2].values_mut()[horizontal_index] = 1.0 / factor;
            fixture.map_factors[3].values_mut()[horizontal_index] = 1.1 + i * 0.01 + j * 0.02;
            for bottom_top in 0..6 {
                let k = bottom_top as f32;
                let index = (south_north * 6 + bottom_top) * 10 + west_east;
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
                fixture.state_volume[0].values_mut()[index] =
                    ((10.0 + i * 0.5) + k * 0.25) - j * 0.125;
                fixture.state_volume[1].values_mut()[index] =
                    ((20.0 - i * 0.25) + k * 0.5) + j * 0.0625;
                fixture.state_volume[2].values_mut()[index] =
                    ((30.0 + i * 0.125) + k * 0.75) - j * 0.25;
                fixture.state_volume[3].values_mut()[index] =
                    ((40.0 - i * 0.0625) + k * 0.375) + j * 0.5;
                fixture.state_volume[4].values_mut()[index] =
                    ((50.0 + i * 0.375) - k * 0.125) + j * 0.25;
            }
        }
    }
}

fn boundary_set(backend: &CpuBackend, base: f32, horizontal: bool) -> BoundarySet {
    let vertical_points = if horizontal { 1 } else { 5 };
    let shape = GridShape::try_new(10, BOUNDARY_WIDTH, vertical_points).unwrap();
    let fields = std::array::from_fn(|role| {
        initialized_boundary_field(backend, shape, role, base, horizontal)
    });
    BoundarySet { fields }
}

fn initialized_boundary_field(
    backend: &CpuBackend,
    shape: GridShape,
    role: usize,
    base: f32,
    horizontal: bool,
) -> CpuField<f32> {
    let mut field = backend.create_field(shape, 0.0).unwrap();
    for distance in 0..BOUNDARY_WIDTH {
        let source_distance = (distance + 1) as f32;
        for vertical in 0..shape.bottom_top_points() {
            let source_vertical = if horizontal {
                0.0
            } else {
                (vertical + 1) as f32
            };
            for line in 0..10 {
                let line = line as f32;
                let value = match role {
                    0 => {
                        base + 10.0
                            + line * 0.5
                            + source_vertical * 0.25
                            + source_distance * 0.03125
                    }
                    1 => {
                        base + 20.0
                            + line * 0.5
                            + source_vertical * 0.25
                            + source_distance * 0.03125
                    }
                    2 => {
                        base + 30.0
                            + line * 0.5
                            + source_vertical * 0.25
                            + source_distance * 0.03125
                    }
                    3 => {
                        base + 40.0
                            + line * 0.5
                            + source_vertical * 0.25
                            + source_distance * 0.03125
                    }
                    4 => {
                        -base * 0.01
                            + line * 0.125
                            + source_vertical * 0.0625
                            + source_distance * 0.015625
                    }
                    5 => {
                        base * 0.01
                            + line * 0.125
                            + source_vertical * 0.0625
                            + source_distance * 0.015625
                    }
                    6 => {
                        -base * 0.02
                            + line * 0.125
                            + source_vertical * 0.0625
                            + source_distance * 0.015625
                    }
                    7 => {
                        base * 0.02
                            + line * 0.125
                            + source_vertical * 0.0625
                            + source_distance * 0.015625
                    }
                    _ => unreachable!(),
                };
                let index = line as usize
                    + shape.west_east_points() * (vertical + shape.bottom_top_points() * distance);
                field.values_mut()[index] = value;
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

fn call_fixture(
    backend: &CpuBackend,
    fixture: &mut Fixture,
) -> DryLargeStepBoundaryStageResult<()> {
    let [ru, rv, rw, ph, t] = &mut fixture.runge_kutta_volume;
    let [ruf, rvf, rwf, phf, tf] = &mut fixture.forward_volume;
    let [us, vs, ws, phs, ts] = &mut fixture.saved_volume;
    let boundaries = DryBoundaryTendencyBoundaryFields::new(
        fixture.boundaries[0].tendencies(),
        fixture.boundaries[1].tendencies(),
        fixture.boundaries[2].tendencies(),
        fixture.boundaries[3].tendencies(),
        fixture.boundaries[5].tendencies(),
    );
    let mode = if fixture.case.first_substep {
        let relaxation = DryLargeStepRelaxationInputs::new(
            DryBoundaryRelaxationState::new(
                &fixture.state_volume[0],
                &fixture.state_volume[1],
                &fixture.state_volume[2],
                &fixture.state_volume[3],
                &fixture.perturbation_column_mass,
                &fixture.full_column_mass,
            ),
            DryLargeStepRelaxationBoundaryValues::new(
                fixture.boundaries[0].values(),
                fixture.boundaries[1].values(),
                fixture.boundaries[2].values(),
                fixture.boundaries[3].values(),
                fixture.boundaries[5].values(),
            ),
            DryBoundaryRelaxationWorkspace::new(&mut fixture.workspace),
            DryBoundaryRelaxationMassCoefficients::new(
                &fixture.half_level_multiplier,
                &fixture.half_level_offset,
                &fixture.full_level_multiplier,
                &fixture.full_level_offset,
            ),
            SpecifiedBoundaryRelaxationCoefficients::new(&fixture.forcing, &fixture.smoothing),
            SpecifiedBoundaryRelaxationParameters::new(
                BOUNDARY_WIDTH,
                fixture.case.specified_zone_width,
                fixture.case.relaxation_zone_width,
                BOUNDARY_TIME_OFFSET,
            ),
        );
        if fixture.case.nested {
            DryLargeStepBoundaryStageMode::FirstSubstepNested {
                relaxation,
                vertical: DryLargeStepNestedVertical::new(
                    &fixture.state_volume[4],
                    fixture.boundaries[4].values(),
                    fixture.boundaries[4].tendencies(),
                ),
            }
        } else {
            DryLargeStepBoundaryStageMode::FirstSubstepGlobal { relaxation }
        }
    } else if fixture.case.nested {
        DryLargeStepBoundaryStageMode::LaterSubstepNested {
            vertical_boundaries: fixture.boundaries[4].tendencies(),
        }
    } else {
        DryLargeStepBoundaryStageMode::LaterSubstepGlobal
    };
    backend.advance_dry_large_step_boundary_stage(
        DryTendencyAssemblyRungeKuttaTendencies::new(
            ru,
            rv,
            rw,
            ph,
            t,
            &mut fixture.runge_kutta_column_mass,
        ),
        DryLargeStepSavedTendencies::new(us, vs, ws, phs, ts),
        DryLargeStepBoundaryStageInputs::new(
            DryTendencyAssemblyForwardTendencies::new(
                ruf,
                rvf,
                rwf,
                phf,
                tf,
                &fixture.forward_column_mass,
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
                &fixture.half_level_multiplier,
                &fixture.half_level_offset,
            ),
            boundaries,
        ),
        mode,
        DryLargeStepBoundaryStageControls::new(
            SpecifiedBoundaryTendencyParameters::new(
                BOUNDARY_WIDTH,
                fixture.case.specified_zone_width,
            ),
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
            &fixture.workspace,
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
