//! Criterion benchmark for the coupled dry-tendency boundary stage.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    DryBoundaryTendencyBoundaryFields, DryTendencyAssemblyCoefficients,
    DryTendencyAssemblyForwardTendencies, DryTendencyAssemblyMapFactors, DryTendencyAssemblyPhase,
    DryTendencyAssemblyRungeKuttaTendencies, DryTendencyAssemblySavedTendencies,
    DryTendencyAssemblyThermodynamics, DryTendencyBoundaryStageControls,
    DryTendencyBoundaryStageInputs, DryTendencyBoundaryStageKernels,
    DryTendencyBoundaryStageRegions, DryTendencyBoundaryStageVertical, SpecifiedBoundaryTendencies,
    SpecifiedBoundaryTendencyParameters, SpecifiedBoundaryWestEastPeriodicity,
};

const HORIZONTAL_POINTS: usize = 256;
const HALF_LEVELS: usize = 40;
const BOUNDARY_WIDTH: usize = 3;

fn benchmark(criterion: &mut Criterion) {
    let volume = HORIZONTAL_POINTS * HORIZONTAL_POINTS * HALF_LEVELS;
    let updated_values = volume * 10 + HORIZONTAL_POINTS * HORIZONTAL_POINTS * 5;
    let mut group = criterion.benchmark_group("dry_tendency_boundary_stage");
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
    fields: [CpuField<f32>; 4],
}

impl BoundarySet {
    fn new(backend: &CpuBackend, horizontal: bool) -> Self {
        let levels = if horizontal { 1 } else { HALF_LEVELS + 1 };
        let shape = GridShape::try_new(HORIZONTAL_POINTS + 2, BOUNDARY_WIDTH, levels).unwrap();
        Self {
            fields: [
                backend.create_field(shape, 0.1).unwrap(),
                backend.create_field(shape, 0.2).unwrap(),
                backend.create_field(shape, 0.3).unwrap(),
                backend.create_field(shape, 0.4).unwrap(),
            ],
        }
    }

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
    runge_kutta: [CpuField<f32>; 5],
    forward: [CpuField<f32>; 5],
    saved: [CpuField<f32>; 5],
    heating: CpuField<f32>,
    runge_kutta_mass: CpuField<f32>,
    forward_mass: CpuField<f32>,
    full_mass: CpuField<f32>,
    maps: [CpuField<f32>; 4],
    full_mass_multiplier: Vec<f32>,
    vertical_offset: Vec<f32>,
    boundaries: [BoundarySet; 6],
    regions: DryTendencyBoundaryStageRegions,
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
            heating: volume(0.001),
            runge_kutta_mass: surface(0.6),
            forward_mass: surface(-0.2),
            full_mass: surface(50.0),
            maps: [
                surface(1.03),
                surface(0.97),
                surface(1.0 / 0.97),
                surface(1.12),
            ],
            full_mass_multiplier: vec![0.2; shape.bottom_top_points()],
            vertical_offset: vec![0.4; shape.bottom_top_points()],
            boundaries: [
                BoundarySet::new(backend, false),
                BoundarySet::new(backend, false),
                BoundarySet::new(backend, false),
                BoundarySet::new(backend, false),
                BoundarySet::new(backend, false),
                BoundarySet::new(backend, true),
            ],
            regions: DryTendencyBoundaryStageRegions::try_new(
                shape,
                1..HORIZONTAL_POINTS + 1,
                1..HORIZONTAL_POINTS + 1,
                1..HALF_LEVELS + 1,
                1..HORIZONTAL_POINTS + 2,
                1..HORIZONTAL_POINTS + 2,
                1..HALF_LEVELS + 2,
            )
            .unwrap(),
        }
    }

    fn apply(&mut self, backend: &CpuBackend) {
        let [u, v, w, ph, t] = &mut self.runge_kutta;
        let [uf, vf, wf, phf, tf] = &mut self.forward;
        let boundary_fields = DryBoundaryTendencyBoundaryFields::new(
            self.boundaries[0].references(),
            self.boundaries[1].references(),
            self.boundaries[2].references(),
            self.boundaries[3].references(),
            self.boundaries[5].references(),
        );
        backend
            .advance_dry_tendency_boundary_stage(
                DryTendencyAssemblyRungeKuttaTendencies::new(
                    u,
                    v,
                    w,
                    ph,
                    t,
                    &mut self.runge_kutta_mass,
                ),
                DryTendencyBoundaryStageInputs::new(
                    DryTendencyAssemblyForwardTendencies::new(
                        uf,
                        vf,
                        wf,
                        phf,
                        tf,
                        &self.forward_mass,
                    ),
                    DryTendencyAssemblySavedTendencies::new(
                        &self.saved[0],
                        &self.saved[1],
                        &self.saved[2],
                        &self.saved[3],
                        &self.saved[4],
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
                DryTendencyBoundaryStageVertical::Nested {
                    boundaries: self.boundaries[4].references(),
                },
                DryTendencyBoundaryStageControls::new(
                    DryTendencyAssemblyPhase::FirstSubstep,
                    SpecifiedBoundaryTendencyParameters::new(BOUNDARY_WIDTH, 2),
                    SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                ),
                &self.regions,
            )
            .unwrap();
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
