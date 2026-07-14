//! Criterion benchmark for WRF complete dry boundary-tendency assignment.
#![allow(missing_docs)]

use std::alloc::System;
use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    DryBoundaryTendencies, DryBoundaryTendencyBoundaryFields, DryBoundaryTendencyKernels,
    DryBoundaryTendencyRegion, DryBoundaryVerticalTendency, SpecifiedBoundaryTendencies,
    SpecifiedBoundaryTendencyParameters, SpecifiedBoundaryWestEastPeriodicity,
};

const HORIZONTAL_MASS_POINTS: usize = 256;
const HALF_LEVELS: usize = 40;
const BOUNDARY_WIDTH: usize = 8;
const ASSIGNED_POINTS: u64 = 1_019_860;

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn benchmark(criterion: &mut Criterion) {
    report_settled_allocations();
    let mut group = criterion.benchmark_group("dry_boundary_tendencies");
    group.throughput(Throughput::Elements(ASSIGNED_POINTS));
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = Fixture::new(&backend);
        group.bench_with_input(
            BenchmarkId::new("nested_full_tile", workers),
            &workers,
            |bencher, _| bencher.iter(|| fixture.apply(black_box(&backend))),
        );
    }
    group.finish();
}

fn report_settled_allocations() {
    let workers = thread::available_parallelism().map_or(1, |count| count.get());
    let backend = CpuBackend::try_with_worker_count(workers).unwrap();
    let mut fixture = Fixture::new(&backend);
    fixture.apply(&backend);
    let allocation_region = Region::new(GLOBAL_ALLOCATOR);
    for _ in 0..100 {
        fixture.apply(&backend);
    }
    eprintln!(
        "settled allocations per 100 calls: {:?}",
        allocation_region.change()
    );
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

struct Fixture {
    outputs: [CpuField<f32>; 5],
    column_mass_output: CpuField<f32>,
    boundaries: [BoundarySet; 6],
    region: DryBoundaryTendencyRegion,
}

impl Fixture {
    fn new(backend: &CpuBackend) -> Self {
        let shape = GridShape::try_new(
            HORIZONTAL_MASS_POINTS + 2,
            HORIZONTAL_MASS_POINTS + 2,
            HALF_LEVELS + 2,
        )
        .unwrap();
        let horizontal_shape = shape.horizontal_shape();
        Self {
            outputs: std::array::from_fn(|_| backend.create_field(shape, 1.2).unwrap()),
            column_mass_output: backend.create_field(horizontal_shape, 1.2).unwrap(),
            boundaries: [
                BoundarySet::new(backend, false),
                BoundarySet::new(backend, false),
                BoundarySet::new(backend, false),
                BoundarySet::new(backend, false),
                BoundarySet::new(backend, false),
                BoundarySet::new(backend, true),
            ],
            region: DryBoundaryTendencyRegion::try_new(
                shape,
                1..HORIZONTAL_MASS_POINTS + 1,
                1..HORIZONTAL_MASS_POINTS + 1,
                1..HALF_LEVELS + 1,
                1..HORIZONTAL_MASS_POINTS + 1,
                1..HORIZONTAL_MASS_POINTS + 1,
                1..HALF_LEVELS + 2,
            )
            .unwrap(),
        }
    }

    fn apply(&mut self, backend: &CpuBackend) {
        let boundaries = DryBoundaryTendencyBoundaryFields::new(
            self.boundaries[0].references(),
            self.boundaries[1].references(),
            self.boundaries[2].references(),
            self.boundaries[3].references(),
            self.boundaries[5].references(),
        );
        let [u, v, ph, t, w] = &mut self.outputs;
        backend
            .assign_dry_boundary_tendencies(
                DryBoundaryTendencies::new(u, v, ph, t, &mut self.column_mass_output),
                boundaries,
                DryBoundaryVerticalTendency::Nested {
                    tendency: w,
                    boundaries: self.boundaries[4].references(),
                },
                SpecifiedBoundaryTendencyParameters::new(BOUNDARY_WIDTH, 5),
                SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                &self.region,
            )
            .unwrap();
    }
}

impl BoundarySet {
    fn new(backend: &CpuBackend, horizontal: bool) -> Self {
        let vertical_points = if horizontal { 1 } else { HALF_LEVELS + 1 };
        let shape = GridShape::try_new(HORIZONTAL_MASS_POINTS + 2, BOUNDARY_WIDTH, vertical_points)
            .unwrap();
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

criterion_group!(benches, benchmark);
criterion_main!(benches);
