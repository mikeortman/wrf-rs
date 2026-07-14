//! Criterion benchmark for WRF specified-boundary tendency assignment.
#![allow(missing_docs)]

use std::alloc::System;
use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    SpecifiedBoundaryFieldLocation, SpecifiedBoundaryTendencies, SpecifiedBoundaryTendencyKernels,
    SpecifiedBoundaryTendencyParameters, SpecifiedBoundaryUpdateRegion,
    SpecifiedBoundaryWestEastPeriodicity,
};

const HORIZONTAL_MASS_POINTS: usize = 256;
const HALF_LEVELS: usize = 40;
const BOUNDARY_WIDTH: usize = 8;
const ASSIGNED_POINTS: u64 = 200_800;

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn benchmark(criterion: &mut Criterion) {
    report_settled_allocations();
    let mut group = criterion.benchmark_group("specified_boundary_tendencies");
    group.throughput(Throughput::Elements(ASSIGNED_POINTS));
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = Fixture::new(&backend);
        group.bench_with_input(
            BenchmarkId::new("mass_half_level_full_tile", workers),
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

struct Fixture {
    tendency: CpuField<f32>,
    boundary_tendencies: [CpuField<f32>; 4],
    region: SpecifiedBoundaryUpdateRegion,
}

impl Fixture {
    fn new(backend: &CpuBackend) -> Self {
        let shape = GridShape::try_new(
            HORIZONTAL_MASS_POINTS + 2,
            HORIZONTAL_MASS_POINTS + 2,
            HALF_LEVELS + 2,
        )
        .unwrap();
        let boundary_shape =
            GridShape::try_new(HORIZONTAL_MASS_POINTS + 2, BOUNDARY_WIDTH, HALF_LEVELS + 1)
                .unwrap();
        let boundary = |value| backend.create_field(boundary_shape, value).unwrap();
        Self {
            tendency: backend.create_field(shape, 1.2).unwrap(),
            boundary_tendencies: [boundary(0.1), boundary(0.2), boundary(0.3), boundary(0.4)],
            region: SpecifiedBoundaryUpdateRegion::try_new(
                shape,
                SpecifiedBoundaryFieldLocation::MassHalfLevel,
                1..HORIZONTAL_MASS_POINTS + 1,
                1..HORIZONTAL_MASS_POINTS + 1,
                1..HALF_LEVELS + 1,
                1..HORIZONTAL_MASS_POINTS + 1,
                1..HORIZONTAL_MASS_POINTS + 1,
                1..HALF_LEVELS + 1,
            )
            .unwrap(),
        }
    }

    fn apply(&mut self, backend: &CpuBackend) {
        let boundaries = SpecifiedBoundaryTendencies::new(
            &self.boundary_tendencies[0],
            &self.boundary_tendencies[1],
            &self.boundary_tendencies[2],
            &self.boundary_tendencies[3],
        );
        backend
            .assign_specified_boundary_tendencies(
                &mut self.tendency,
                boundaries,
                SpecifiedBoundaryTendencyParameters::new(BOUNDARY_WIDTH, 5),
                SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                &self.region,
            )
            .unwrap();
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
