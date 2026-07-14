//! Criterion benchmark for WRF physical boundary-zone assignment.
#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    PhysicalBoundaryConditions, PhysicalBoundaryKernels, PhysicalBoundaryRegion,
    PhysicalBoundaryVariable,
};

const ACTIVE_HORIZONTAL_POINTS: usize = 256;
const HALF_LEVELS: usize = 40;
const ASSIGNED_VALUES: usize = 126_960;

fn benchmark(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("physical_boundary");
    group.throughput(Throughput::Elements(ASSIGNED_VALUES as u64));
    for workers in [1, 4, 16] {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = Fixture::new(&backend);
        group.bench_with_input(
            BenchmarkId::new("specified_mass_full_tile", workers),
            &workers,
            |bencher, _| bencher.iter(|| fixture.apply(black_box(&backend))),
        );
    }
    group.finish();
}

struct Fixture {
    field: CpuField<f32>,
    region: PhysicalBoundaryRegion,
}

impl Fixture {
    fn new(backend: &CpuBackend) -> Self {
        let shape = GridShape::try_new(
            ACTIVE_HORIZONTAL_POINTS + 9,
            ACTIVE_HORIZONTAL_POINTS + 9,
            HALF_LEVELS + 2,
        )
        .unwrap();
        Self {
            field: backend.create_field(shape, 1.2).unwrap(),
            region: PhysicalBoundaryRegion::try_new(
                shape,
                4..ACTIVE_HORIZONTAL_POINTS + 4,
                4..ACTIVE_HORIZONTAL_POINTS + 4,
                1..HALF_LEVELS + 1,
                4..ACTIVE_HORIZONTAL_POINTS + 5,
                4..ACTIVE_HORIZONTAL_POINTS + 5,
                1..HALF_LEVELS + 2,
            )
            .unwrap(),
        }
    }

    fn apply(&mut self, backend: &CpuBackend) {
        backend
            .apply_physical_boundary(
                &mut self.field,
                PhysicalBoundaryVariable::MassHalfLevel,
                PhysicalBoundaryConditions::specified_lateral(),
                &self.region,
            )
            .unwrap();
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
