//! Criterion benchmark for WRF specified-boundary tendency updates.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    SpecifiedBoundaryFieldLocation, SpecifiedBoundaryUpdateKernels,
    SpecifiedBoundaryUpdateParameters, SpecifiedBoundaryUpdateRegion,
    SpecifiedBoundaryWestEastPeriodicity,
};

const HORIZONTAL_MASS_POINTS: usize = 256;
const HALF_LEVELS: usize = 40;
const SPECIFIED_ZONE_WIDTH: usize = 5;
const UPDATED_POINTS: u64 = 200_800;

fn benchmark(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("specified_boundary_update");
    group.throughput(Throughput::Elements(UPDATED_POINTS));
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = Fixture::new(&backend);
        group.bench_with_input(
            BenchmarkId::new("mass_full_tile", workers),
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

struct Fixture {
    field: CpuField<f32>,
    tendency: CpuField<f32>,
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
        Self {
            field: backend.create_field(shape, 1.2).unwrap(),
            tendency: backend.create_field(shape, 0.1).unwrap(),
            region: SpecifiedBoundaryUpdateRegion::try_new(
                shape,
                SpecifiedBoundaryFieldLocation::MassHalfLevel,
                1..HORIZONTAL_MASS_POINTS + 1,
                1..HORIZONTAL_MASS_POINTS + 1,
                1..HALF_LEVELS + 1,
                1..HORIZONTAL_MASS_POINTS + 2,
                1..HORIZONTAL_MASS_POINTS + 2,
                1..HALF_LEVELS + 2,
            )
            .unwrap(),
        }
    }

    fn apply(&mut self, backend: &CpuBackend) {
        backend
            .update_specified_boundary(
                &mut self.field,
                &self.tendency,
                SpecifiedBoundaryUpdateParameters::new(0.25, SPECIFIED_ZONE_WIDTH),
                SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                &self.region,
            )
            .unwrap();
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
