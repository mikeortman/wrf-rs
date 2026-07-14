//! Criterion benchmark for the complete dry large-step boundary stage.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
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

const HORIZONTAL_POINTS: usize = 256;
const HALF_LEVELS: usize = 40;
const BOUNDARY_WIDTH: usize = 5;

fn benchmark(criterion: &mut Criterion) {
    let volume = HORIZONTAL_POINTS * HORIZONTAL_POINTS * HALF_LEVELS;
    let updated_values = volume * 15 + HORIZONTAL_POINTS * HORIZONTAL_POINTS * 2;
    let mut group = criterion.benchmark_group("dry_large_step_boundary_stage");
    group.throughput(Throughput::Elements(updated_values as u64));
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = Fixture::new(&backend);
        group.bench_with_input(
            BenchmarkId::new("first_nested", workers),
            &workers,
            |bencher, _| {
                bencher.iter(|| fixture.apply(black_box(&backend)));
            },
        );
    }
    group.finish();
}

fn worker_counts() -> Vec<usize> {
    let available = thread::available_parallelism().map_or(1, |count| count.get());
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
        let levels = if horizontal { 1 } else { HALF_LEVELS + 1 };
        let shape = GridShape::try_new(HORIZONTAL_POINTS + 2, BOUNDARY_WIDTH, levels).unwrap();
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

impl Fixture {
    fn new(backend: &CpuBackend) -> Self {
        let shape = GridShape::try_new(
            HORIZONTAL_POINTS + 2,
            HORIZONTAL_POINTS + 2,
            HALF_LEVELS + 2,
        )
        .unwrap();
        let horizontal = shape.horizontal_shape();
        let volume = |value| backend.create_field(shape, value).unwrap();
        let surface = |value| backend.create_field(horizontal, value).unwrap();
        let regions = DryLargeStepBoundaryStageRegions::try_new(
            shape,
            1..HORIZONTAL_POINTS + 1,
            1..HORIZONTAL_POINTS + 1,
            1..HALF_LEVELS + 1,
            1..HORIZONTAL_POINTS + 2,
            1..HORIZONTAL_POINTS + 2,
            1..HALF_LEVELS + 2,
        )
        .unwrap();
        let workspace = backend
            .create_field(regions.relaxation_workspace_shape(), 0.0)
            .unwrap();
        Self {
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
            runge_kutta_mass: surface(0.6),
            forward_mass: surface(-0.2),
            perturbation_mass: surface(1.7),
            full_mass: surface(50.0),
            maps: [
                surface(1.03),
                surface(0.97),
                surface(1.0 / 0.97),
                surface(1.12),
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

    fn apply(&mut self, backend: &CpuBackend) {
        let [u, v, w, ph, t] = &mut self.runge_kutta;
        let [uf, vf, wf, phf, tf] = &mut self.forward;
        let [us, vs, ws, phs, ts] = &mut self.saved;
        let boundary_fields = DryBoundaryTendencyBoundaryFields::new(
            self.boundaries[0].tendencies(),
            self.boundaries[1].tendencies(),
            self.boundaries[2].tendencies(),
            self.boundaries[3].tendencies(),
            self.boundaries[5].tendencies(),
        );
        backend
            .advance_dry_large_step_boundary_stage(
                DryTendencyAssemblyRungeKuttaTendencies::new(
                    u,
                    v,
                    w,
                    ph,
                    t,
                    &mut self.runge_kutta_mass,
                ),
                DryLargeStepSavedTendencies::new(us, vs, ws, phs, ts),
                DryLargeStepBoundaryStageInputs::new(
                    DryTendencyAssemblyForwardTendencies::new(
                        uf,
                        vf,
                        wf,
                        phf,
                        tf,
                        &self.forward_mass,
                    ),
                    DryTendencyAssemblyThermodynamics::new(&self.heating, &self.full_mass),
                    DryTendencyAssemblyMapFactors::new(
                        &self.maps[0],
                        &self.maps[1],
                        &self.maps[2],
                        &self.maps[3],
                    ),
                    DryTendencyAssemblyCoefficients::new(
                        &self.full_mass_multiplier,
                        &self.vertical_offset,
                    ),
                    boundary_fields,
                ),
                DryLargeStepBoundaryStageMode::FirstSubstepNested {
                    relaxation: DryLargeStepRelaxationInputs::new(
                        DryBoundaryRelaxationState::new(
                            &self.state[0],
                            &self.state[1],
                            &self.state[2],
                            &self.state[3],
                            &self.perturbation_mass,
                            &self.full_mass,
                        ),
                        DryLargeStepRelaxationBoundaryValues::new(
                            self.boundaries[0].values(),
                            self.boundaries[1].values(),
                            self.boundaries[2].values(),
                            self.boundaries[3].values(),
                            self.boundaries[5].values(),
                        ),
                        DryBoundaryRelaxationWorkspace::new(&mut self.workspace),
                        DryBoundaryRelaxationMassCoefficients::new(
                            &self.half_multiplier,
                            &self.half_offset,
                            &self.full_multiplier,
                            &self.full_offset,
                        ),
                        SpecifiedBoundaryRelaxationCoefficients::new(
                            &[0.0, 0.7, 0.5, 0.3, 0.0],
                            &[0.0, 0.1, 0.07, 0.04, 0.0],
                        ),
                        SpecifiedBoundaryRelaxationParameters::new(BOUNDARY_WIDTH, 1, 4, 0.25),
                    ),
                    vertical: DryLargeStepNestedVertical::new(
                        &self.state[4],
                        self.boundaries[4].values(),
                        self.boundaries[4].tendencies(),
                    ),
                },
                DryLargeStepBoundaryStageControls::new(
                    SpecifiedBoundaryTendencyParameters::new(BOUNDARY_WIDTH, 1),
                    SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                ),
                &self.regions,
            )
            .unwrap();
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
