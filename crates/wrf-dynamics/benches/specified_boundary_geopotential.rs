//! Criterion benchmark for mass-normalized geopotential boundary updates.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    SpecifiedBoundaryFieldLocation, SpecifiedBoundaryGeopotentialInputs,
    SpecifiedBoundaryGeopotentialKernels, SpecifiedBoundaryUpdateParameters,
    SpecifiedBoundaryUpdateRegion, SpecifiedBoundaryWestEastPeriodicity,
};

const HORIZONTAL_MASS_POINTS: usize = 256;
const HALF_LEVELS: usize = 40;
const UPDATED_POINTS: u64 = 205_820;

fn benchmark(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("specified_boundary_geopotential");
    group.throughput(Throughput::Elements(UPDATED_POINTS));
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = Fixture::new(&backend);
        group.bench_with_input(
            BenchmarkId::new("full_level_full_tile", workers),
            &workers,
            |bencher, _| bencher.iter(|| fixture.apply(black_box(&backend))),
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
    geopotential: CpuField<f32>,
    volumes: [CpuField<f32>; 2],
    horizontal: [CpuField<f32>; 2],
    coefficients: [Vec<f32>; 2],
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
        let volume = |value| backend.create_field(shape, value).unwrap();
        let horizontal = |value| {
            backend
                .create_field(shape.horizontal_shape(), value)
                .unwrap()
        };
        Self {
            geopotential: volume(-200.0),
            volumes: [volume(100.0), volume(1.5)],
            horizontal: [horizontal(0.3), horizontal(10.0)],
            coefficients: [
                vec![0.4; shape.bottom_top_points()],
                vec![2.0; shape.bottom_top_points()],
            ],
            region: SpecifiedBoundaryUpdateRegion::try_new(
                shape,
                SpecifiedBoundaryFieldLocation::FullLevel,
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
        let inputs = SpecifiedBoundaryGeopotentialInputs::new(
            &self.volumes[0],
            &self.volumes[1],
            &self.horizontal[0],
            &self.horizontal[1],
            &self.coefficients[0],
            &self.coefficients[1],
        );
        backend
            .update_specified_boundary_geopotential(
                &mut self.geopotential,
                inputs,
                SpecifiedBoundaryUpdateParameters::new(0.25, 5),
                SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                &self.region,
            )
            .unwrap();
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
