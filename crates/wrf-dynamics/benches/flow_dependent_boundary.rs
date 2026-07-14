//! Criterion benchmark for WRF flow-dependent specified boundaries.
#![allow(missing_docs)]

use std::alloc::System;
use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    SpecifiedBoundaryFlowInputs, SpecifiedBoundaryFlowKernels, SpecifiedBoundaryFlowParameters,
    SpecifiedBoundaryFlowRegion, SpecifiedBoundaryWestEastPeriodicity,
};

const HORIZONTAL_MASS_POINTS: usize = 256;
const HALF_LEVELS: usize = 40;
const UPDATED_POINTS: u64 = 200_800;

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn benchmark(criterion: &mut Criterion) {
    report_settled_allocations();
    let mut group = criterion.benchmark_group("flow_dependent_boundary");
    group.throughput(Throughput::Elements(UPDATED_POINTS));
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = Fixture::new(&backend);
        group.bench_with_input(
            BenchmarkId::new("mixed_flow_full_tile", workers),
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
    scalar: CpuField<f32>,
    west_east_velocity: CpuField<f32>,
    south_north_velocity: CpuField<f32>,
    region: SpecifiedBoundaryFlowRegion,
}

impl Fixture {
    fn new(backend: &CpuBackend) -> Self {
        let shape = GridShape::try_new(
            HORIZONTAL_MASS_POINTS + 2,
            HORIZONTAL_MASS_POINTS + 2,
            HALF_LEVELS + 2,
        )
        .unwrap();
        let mut scalar = backend.create_field(shape, 0.0).unwrap();
        let mut west_east_velocity = backend.create_field(shape, 0.0).unwrap();
        let mut south_north_velocity = backend.create_field(shape, 0.0).unwrap();
        for index in 0..shape.point_count() {
            scalar.values_mut()[index] = index as f32 * 0.000_001 - 30.0;
            west_east_velocity.values_mut()[index] = if index % 2 == 0 { -1.0 } else { 1.0 };
            south_north_velocity.values_mut()[index] = if index % 3 == 0 { -1.0 } else { 1.0 };
        }
        Self {
            scalar,
            west_east_velocity,
            south_north_velocity,
            region: SpecifiedBoundaryFlowRegion::try_new(
                shape,
                1..HORIZONTAL_MASS_POINTS + 1,
                1..HORIZONTAL_MASS_POINTS + 1,
                1..HALF_LEVELS + 1,
                1..HORIZONTAL_MASS_POINTS + 2,
                1..HORIZONTAL_MASS_POINTS + 2,
                1..HALF_LEVELS + 1,
            )
            .unwrap(),
        }
    }

    fn apply(&mut self, backend: &CpuBackend) {
        backend
            .apply_flow_dependent_specified_boundary(
                &mut self.scalar,
                SpecifiedBoundaryFlowInputs::new(
                    &self.west_east_velocity,
                    &self.south_north_velocity,
                ),
                SpecifiedBoundaryFlowParameters::new(5),
                SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                &self.region,
            )
            .unwrap();
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
