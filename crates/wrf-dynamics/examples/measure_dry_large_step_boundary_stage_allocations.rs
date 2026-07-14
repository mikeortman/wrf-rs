//! Measures warmed allocations in the WRF dry large-step boundary stage.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    DryBoundaryRelaxationMassCoefficients, DryBoundaryRelaxationState,
    DryBoundaryRelaxationWorkspace, DryBoundaryTendencyBoundaryFields,
    DryLargeStepBoundaryStageControls, DryLargeStepBoundaryStageInputs,
    DryLargeStepBoundaryStageKernels, DryLargeStepBoundaryStageMode,
    DryLargeStepBoundaryStageRegions, DryLargeStepNestedVertical,
    DryLargeStepRelaxationBoundaryValues, DryLargeStepRelaxationInputs,
    DryLargeStepSavedTendencies, DryTendencyAssemblyCoefficients,
    DryTendencyAssemblyForwardTendencies, DryTendencyAssemblyMapFactors,
    DryTendencyAssemblyRungeKuttaTendencies, DryTendencyAssemblyThermodynamics,
    SpecifiedBoundaryRelaxationBoundaryValues, SpecifiedBoundaryRelaxationCoefficients,
    SpecifiedBoundaryRelaxationParameters, SpecifiedBoundaryTendencies,
    SpecifiedBoundaryTendencyParameters, SpecifiedBoundaryWestEastPeriodicity,
};

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const ACTIVE_WEST_EAST_POINTS: usize = 34;
const ACTIVE_SOUTH_NORTH_POINTS: usize = 34;
const ACTIVE_BOTTOM_TOP_POINTS: usize = 12;
const BOUNDARY_WIDTH: usize = 5;
const DISPATCHES: usize = 100;

fn main() {
    println!("kernel,phase,workers,dispatches,allocations,reallocations,bytes_allocated");
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = create_fixture(&backend);
        run_dispatches(&backend, &mut fixture);
        let first_region = Region::new(GLOBAL_ALLOCATOR);
        run_dispatches(&backend, &mut fixture);
        let first = first_region.change();
        let settled_region = Region::new(GLOBAL_ALLOCATOR);
        run_dispatches(&backend, &mut fixture);
        let settled = settled_region.change();
        print_stats("first", workers, first);
        print_stats("settled", workers, settled);
        assert_budget(first);
        assert_budget(settled);
    }
}

fn worker_counts() -> Vec<usize> {
    let available = std::thread::available_parallelism().map_or(1, |count| count.get());
    let mut counts = vec![1, available.min(4), available];
    counts.sort_unstable();
    counts.dedup();
    counts
}

struct BoundarySet {
    value_fields: [CpuField<f32>; 4],
    tendency_fields: [CpuField<f32>; 4],
}

impl BoundarySet {
    fn new(backend: &CpuBackend, horizontal: bool) -> Self {
        let levels = if horizontal {
            1
        } else {
            ACTIVE_BOTTOM_TOP_POINTS + 1
        };
        let shape =
            GridShape::try_new(ACTIVE_WEST_EAST_POINTS + 2, BOUNDARY_WIDTH, levels).unwrap();
        Self {
            value_fields: [
                backend.create_field(shape, 2.1).unwrap(),
                backend.create_field(shape, 2.2).unwrap(),
                backend.create_field(shape, 2.3).unwrap(),
                backend.create_field(shape, 2.4).unwrap(),
            ],
            tendency_fields: [
                backend.create_field(shape, 0.1).unwrap(),
                backend.create_field(shape, 0.2).unwrap(),
                backend.create_field(shape, 0.3).unwrap(),
                backend.create_field(shape, 0.4).unwrap(),
            ],
        }
    }

    fn values(&self) -> SpecifiedBoundaryRelaxationBoundaryValues<'_, CpuField<f32>> {
        SpecifiedBoundaryRelaxationBoundaryValues::new(
            &self.value_fields[0],
            &self.value_fields[1],
            &self.value_fields[2],
            &self.value_fields[3],
        )
    }

    fn tendencies(&self) -> SpecifiedBoundaryTendencies<'_, CpuField<f32>> {
        SpecifiedBoundaryTendencies::new(
            &self.tendency_fields[0],
            &self.tendency_fields[1],
            &self.tendency_fields[2],
            &self.tendency_fields[3],
        )
    }
}

struct Fixture {
    runge_kutta: [CpuField<f32>; 5],
    forward: [CpuField<f32>; 5],
    saved: [CpuField<f32>; 5],
    state: [CpuField<f32>; 5],
    heating: CpuField<f32>,
    runge_kutta_mass: CpuField<f32>,
    forward_mass: CpuField<f32>,
    perturbation_mass: CpuField<f32>,
    full_mass: CpuField<f32>,
    maps: [CpuField<f32>; 4],
    full_mass_multiplier: Vec<f32>,
    vertical_offset: Vec<f32>,
    half_multiplier: Vec<f32>,
    half_offset: Vec<f32>,
    full_multiplier: Vec<f32>,
    full_offset: Vec<f32>,
    boundaries: [BoundarySet; 6],
    workspace: CpuField<f32>,
    regions: DryLargeStepBoundaryStageRegions,
}

fn create_fixture(backend: &CpuBackend) -> Fixture {
    let shape = GridShape::try_new(
        ACTIVE_WEST_EAST_POINTS + 2,
        ACTIVE_SOUTH_NORTH_POINTS + 2,
        ACTIVE_BOTTOM_TOP_POINTS + 2,
    )
    .unwrap();
    let horizontal_shape = shape.horizontal_shape();
    let volume = |value| backend.create_field(shape, value).unwrap();
    let horizontal = |value| backend.create_field(horizontal_shape, value).unwrap();
    let regions = DryLargeStepBoundaryStageRegions::try_new(
        shape,
        1..(ACTIVE_WEST_EAST_POINTS + 1),
        1..(ACTIVE_SOUTH_NORTH_POINTS + 1),
        1..(ACTIVE_BOTTOM_TOP_POINTS + 1),
        1..(ACTIVE_WEST_EAST_POINTS + 2),
        1..(ACTIVE_SOUTH_NORTH_POINTS + 2),
        1..(ACTIVE_BOTTOM_TOP_POINTS + 2),
    )
    .unwrap();
    let workspace = backend
        .create_field(regions.relaxation_workspace_shape(), 0.0)
        .unwrap();
    Fixture {
        runge_kutta: [
            volume(1.0),
            volume(2.0),
            volume(-1.0),
            volume(3.0),
            volume(-2.0),
        ],
        forward: [
            volume(0.3),
            volume(-0.4),
            volume(0.5),
            volume(-0.6),
            volume(0.7),
        ],
        saved: [
            volume(0.09),
            volume(-0.08),
            volume(0.07),
            volume(-0.06),
            volume(0.05),
        ],
        state: [
            volume(1.2),
            volume(1.3),
            volume(1.4),
            volume(1.5),
            volume(1.6),
        ],
        heating: volume(0.001),
        runge_kutta_mass: horizontal(0.6),
        forward_mass: horizontal(-0.2),
        perturbation_mass: horizontal(1.7),
        full_mass: horizontal(50.0),
        maps: [
            horizontal(1.03),
            horizontal(0.97),
            horizontal(1.0 / 0.97),
            horizontal(1.12),
        ],
        full_mass_multiplier: vec![0.2; shape.bottom_top_points()],
        vertical_offset: vec![0.4; shape.bottom_top_points()],
        half_multiplier: vec![0.60; shape.bottom_top_points()],
        half_offset: vec![0.40; shape.bottom_top_points()],
        full_multiplier: vec![0.55; shape.bottom_top_points()],
        full_offset: vec![0.45; shape.bottom_top_points()],
        boundaries: [
            BoundarySet::new(backend, false),
            BoundarySet::new(backend, false),
            BoundarySet::new(backend, false),
            BoundarySet::new(backend, false),
            BoundarySet::new(backend, false),
            BoundarySet::new(backend, true),
        ],
        workspace,
        regions,
    }
}

fn run_dispatches(backend: &CpuBackend, fixture: &mut Fixture) {
    for _ in 0..DISPATCHES {
        let [u, v, w, ph, t] = &mut fixture.runge_kutta;
        let [uf, vf, wf, phf, tf] = &mut fixture.forward;
        let [us, vs, ws, phs, ts] = &mut fixture.saved;
        let boundary_fields = DryBoundaryTendencyBoundaryFields::new(
            fixture.boundaries[0].tendencies(),
            fixture.boundaries[1].tendencies(),
            fixture.boundaries[2].tendencies(),
            fixture.boundaries[3].tendencies(),
            fixture.boundaries[5].tendencies(),
        );
        backend
            .advance_dry_large_step_boundary_stage(
                DryTendencyAssemblyRungeKuttaTendencies::new(
                    u,
                    v,
                    w,
                    ph,
                    t,
                    &mut fixture.runge_kutta_mass,
                ),
                DryLargeStepSavedTendencies::new(us, vs, ws, phs, ts),
                DryLargeStepBoundaryStageInputs::new(
                    DryTendencyAssemblyForwardTendencies::new(
                        uf,
                        vf,
                        wf,
                        phf,
                        tf,
                        &fixture.forward_mass,
                    ),
                    DryTendencyAssemblyThermodynamics::new(&fixture.heating, &fixture.full_mass),
                    DryTendencyAssemblyMapFactors::new(
                        &fixture.maps[0],
                        &fixture.maps[1],
                        &fixture.maps[2],
                        &fixture.maps[3],
                    ),
                    DryTendencyAssemblyCoefficients::new(
                        &fixture.full_mass_multiplier,
                        &fixture.vertical_offset,
                    ),
                    boundary_fields,
                ),
                DryLargeStepBoundaryStageMode::FirstSubstepNested {
                    relaxation: DryLargeStepRelaxationInputs::new(
                        DryBoundaryRelaxationState::new(
                            &fixture.state[0],
                            &fixture.state[1],
                            &fixture.state[2],
                            &fixture.state[3],
                            &fixture.perturbation_mass,
                            &fixture.full_mass,
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
                            &fixture.half_multiplier,
                            &fixture.half_offset,
                            &fixture.full_multiplier,
                            &fixture.full_offset,
                        ),
                        SpecifiedBoundaryRelaxationCoefficients::new(
                            &[0.0, 0.7, 0.5, 0.3, 0.0],
                            &[0.0, 0.1, 0.07, 0.04, 0.0],
                        ),
                        SpecifiedBoundaryRelaxationParameters::new(BOUNDARY_WIDTH, 1, 4, 0.25),
                    ),
                    vertical: DryLargeStepNestedVertical::new(
                        &fixture.state[4],
                        fixture.boundaries[4].values(),
                        fixture.boundaries[4].tendencies(),
                    ),
                },
                DryLargeStepBoundaryStageControls::new(
                    SpecifiedBoundaryTendencyParameters::new(BOUNDARY_WIDTH, 1),
                    SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                ),
                &fixture.regions,
            )
            .unwrap();
    }
}

fn print_stats(phase: &str, workers: usize, stats: Stats) {
    println!(
        "dry_large_step_boundary_stage,{phase},{workers},{DISPATCHES},{},{},{}",
        stats.allocations, stats.reallocations, stats.bytes_allocated
    );
}

fn assert_budget(stats: Stats) {
    assert_eq!(stats.reallocations, 0);
    assert!(stats.allocations <= DISPATCHES * 7);
    assert!(stats.bytes_allocated <= 2 * 1_024 * 1_024);
}
