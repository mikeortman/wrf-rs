//! Criterion benchmark for WRF specified-boundary state finalization.
#![allow(missing_docs)]

use std::alloc::System;
use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    SpecifiedBoundaryFinalizationBoundaryFields, SpecifiedBoundaryFinalizationFieldLocation,
    SpecifiedBoundaryFinalizationInputs, SpecifiedBoundaryFinalizationKernels,
    SpecifiedBoundaryFinalizationParameters, SpecifiedBoundaryFinalizationRegion,
    SpecifiedBoundaryWestEastPeriodicity,
};

const HORIZONTAL_MASS_POINTS: usize = 256;
const HALF_LEVELS: usize = 40;
const BOUNDARY_WIDTH: usize = 8;
const UPDATED_POINTS: u64 = 205_820;

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn benchmark(criterion: &mut Criterion) {
    report_settled_allocations();
    let mut group = criterion.benchmark_group("specified_boundary_finalization");
    group.throughput(Throughput::Elements(UPDATED_POINTS));
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = Fixture::new(&backend);
        group.bench_with_input(
            BenchmarkId::new("vertical_momentum_full_tile", workers),
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
    column_mass: CpuField<f32>,
    map_factor: CpuField<f32>,
    column_mass_multiplier: Vec<f32>,
    column_mass_offset: Vec<f32>,
    boundary_values: [CpuField<f32>; 4],
    boundary_tendencies: [CpuField<f32>; 4],
    region: SpecifiedBoundaryFinalizationRegion,
}

impl Fixture {
    fn new(backend: &CpuBackend) -> Self {
        let shape = GridShape::try_new(
            HORIZONTAL_MASS_POINTS + 2,
            HORIZONTAL_MASS_POINTS + 2,
            HALF_LEVELS + 2,
        )
        .unwrap();
        let horizontal = |value| {
            backend
                .create_field(shape.horizontal_shape(), value)
                .unwrap()
        };
        let boundary_shape =
            GridShape::try_new(HORIZONTAL_MASS_POINTS + 2, BOUNDARY_WIDTH, HALF_LEVELS + 1)
                .unwrap();
        let boundary = |value| backend.create_field(boundary_shape, value).unwrap();
        Self {
            field: backend.create_field(shape, -30.0).unwrap(),
            column_mass: horizontal(5.0),
            map_factor: horizontal(0.9),
            column_mass_multiplier: vec![0.4; shape.bottom_top_points()],
            column_mass_offset: vec![1.3; shape.bottom_top_points()],
            boundary_values: [
                boundary(10.0),
                boundary(-8.0),
                boundary(4.0),
                boundary(-2.0),
            ],
            boundary_tendencies: [
                boundary(0.03),
                boundary(-0.02),
                boundary(0.01),
                boundary(-0.015),
            ],
            region: SpecifiedBoundaryFinalizationRegion::try_new(
                shape,
                SpecifiedBoundaryFinalizationFieldLocation::VerticalMomentum,
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
        let values = SpecifiedBoundaryFinalizationBoundaryFields::new(
            &self.boundary_values[0],
            &self.boundary_values[1],
            &self.boundary_values[2],
            &self.boundary_values[3],
        );
        let tendencies = SpecifiedBoundaryFinalizationBoundaryFields::new(
            &self.boundary_tendencies[0],
            &self.boundary_tendencies[1],
            &self.boundary_tendencies[2],
            &self.boundary_tendencies[3],
        );
        backend
            .finalize_specified_boundary(
                &mut self.field,
                SpecifiedBoundaryFinalizationInputs::new(
                    &self.column_mass,
                    &self.map_factor,
                    &self.column_mass_multiplier,
                    &self.column_mass_offset,
                    values,
                    tendencies,
                ),
                SpecifiedBoundaryFinalizationParameters::new(0.25, BOUNDARY_WIDTH, 5),
                SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                &self.region,
            )
            .unwrap();
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
