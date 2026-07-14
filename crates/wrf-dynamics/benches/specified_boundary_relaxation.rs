//! Criterion benchmark for WRF specified-boundary relaxation forcing.
#![allow(missing_docs)]

use std::alloc::System;
use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, FieldStorage, GridShape};
use wrf_dynamics::{
    SpecifiedBoundaryFieldLocation, SpecifiedBoundaryRelaxationBoundaryValues,
    SpecifiedBoundaryRelaxationCoefficients, SpecifiedBoundaryRelaxationField,
    SpecifiedBoundaryRelaxationInputs, SpecifiedBoundaryRelaxationKernels,
    SpecifiedBoundaryRelaxationParameters, SpecifiedBoundaryTendencies,
    SpecifiedBoundaryUpdateRegion, SpecifiedBoundaryWestEastPeriodicity,
};

const HORIZONTAL_MASS_POINTS: usize = 256;
const HALF_LEVELS: usize = 40;
const BOUNDARY_WIDTH: usize = 8;
const UPDATED_POINTS: u64 = 238_080;

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn benchmark(criterion: &mut Criterion) {
    report_settled_allocations();
    let mut group = criterion.benchmark_group("specified_boundary_relaxation");
    group.throughput(Throughput::Elements(UPDATED_POINTS));
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
    field: CpuField<f32>,
    tendency: CpuField<f32>,
    boundaries: [CpuField<f32>; 8],
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
            field: backend.create_field(shape, 1.2).unwrap(),
            tendency: backend.create_field(shape, 0.1).unwrap(),
            boundaries: [
                boundary(2.1),
                boundary(2.2),
                boundary(2.3),
                boundary(2.4),
                boundary(0.01),
                boundary(0.02),
                boundary(0.03),
                boundary(0.04),
            ],
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
        let shape = self.field.shape();
        backend
            .add_specified_boundary_relaxation_tendencies(
                &mut self.tendency,
                SpecifiedBoundaryRelaxationInputs::new(
                    SpecifiedBoundaryRelaxationField::new(
                        &self.field,
                        0..shape.west_east_points(),
                        0..shape.south_north_points(),
                        0..shape.bottom_top_points(),
                    ),
                    SpecifiedBoundaryRelaxationBoundaryValues::new(
                        &self.boundaries[0],
                        &self.boundaries[1],
                        &self.boundaries[2],
                        &self.boundaries[3],
                    ),
                    SpecifiedBoundaryTendencies::new(
                        &self.boundaries[4],
                        &self.boundaries[5],
                        &self.boundaries[6],
                        &self.boundaries[7],
                    ),
                    SpecifiedBoundaryRelaxationCoefficients::new(
                        &[0.0, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.0],
                        &[0.0, 0.08, 0.07, 0.06, 0.05, 0.04, 0.03, 0.0],
                    ),
                ),
                SpecifiedBoundaryRelaxationParameters::new(BOUNDARY_WIDTH, 1, 7, 0.25),
                SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                &self.region,
            )
            .unwrap();
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
