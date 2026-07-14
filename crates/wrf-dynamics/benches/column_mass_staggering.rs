//! Criterion throughput benchmark for column-mass staggering.

// Criterion 0.7 generates a public harness function without a doc comment.
// Rust 1.85 prevents adopting Criterion 0.8, which fixes that warning.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, FieldStorage, GridShape};
use wrf_dynamics::{ColumnMassStaggeringKernels, ColumnMassStaggeringRegion};

const ACTIVE_WEST_EAST_MASS_POINTS: usize = 1_024;
const ACTIVE_SOUTH_NORTH_MASS_POINTS: usize = 1_024;

fn benchmark_column_mass_staggering(criterion: &mut Criterion) {
    let output_count = (ACTIVE_WEST_EAST_MASS_POINTS + 1) * ACTIVE_SOUTH_NORTH_MASS_POINTS
        + ACTIVE_WEST_EAST_MASS_POINTS * (ACTIVE_SOUTH_NORTH_MASS_POINTS + 1);
    let mut group = criterion.benchmark_group("column_mass_staggering");
    group.throughput(Throughput::Elements(output_count as u64));

    for worker_count in benchmark_worker_counts() {
        let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
        let mut fixture = create_fixture(&backend);
        group.bench_with_input(
            BenchmarkId::new("both_boundaries", worker_count),
            &worker_count,
            |bencher, _| {
                bencher.iter(|| {
                    backend
                        .stagger_column_mass(
                            black_box(&fixture.perturbation_mass),
                            black_box(&fixture.base_mass),
                            black_box(&mut fixture.west_east_momentum_mass),
                            black_box(&mut fixture.south_north_momentum_mass),
                            black_box(&fixture.region),
                        )
                        .unwrap();
                });
            },
        );
    }

    group.finish();
}

fn benchmark_worker_counts() -> Vec<usize> {
    let available_workers = thread::available_parallelism().map_or(1, |count| count.get());
    let mut worker_counts = vec![1, available_workers.min(4), available_workers];
    worker_counts.sort_unstable();
    worker_counts.dedup();
    worker_counts
}

struct ColumnMassStaggeringBenchmarkFixture {
    perturbation_mass: CpuField<f32>,
    base_mass: CpuField<f32>,
    west_east_momentum_mass: CpuField<f32>,
    south_north_momentum_mass: CpuField<f32>,
    region: ColumnMassStaggeringRegion,
}

fn create_fixture(backend: &CpuBackend) -> ColumnMassStaggeringBenchmarkFixture {
    let west_east_points = ACTIVE_WEST_EAST_MASS_POINTS + 2;
    let south_north_points = ACTIVE_SOUTH_NORTH_MASS_POINTS + 2;
    let shape = GridShape::try_new(west_east_points, south_north_points, 1).unwrap();
    let mut fixture = ColumnMassStaggeringBenchmarkFixture {
        perturbation_mass: backend.create_field(shape, 0.0_f32).unwrap(),
        base_mass: backend.create_field(shape, 0.0_f32).unwrap(),
        west_east_momentum_mass: backend.create_field(shape, -999.0_f32).unwrap(),
        south_north_momentum_mass: backend.create_field(shape, -999.0_f32).unwrap(),
        region: ColumnMassStaggeringRegion::try_new(
            shape,
            1..(ACTIVE_WEST_EAST_MASS_POINTS + 1),
            1..(ACTIVE_SOUTH_NORTH_MASS_POINTS + 1),
            1..(ACTIVE_WEST_EAST_MASS_POINTS + 2),
            1..(ACTIVE_SOUTH_NORTH_MASS_POINTS + 2),
        )
        .unwrap(),
    };

    initialize_mass_fields(&mut fixture.perturbation_mass, &mut fixture.base_mass);
    fixture
}

fn initialize_mass_fields(perturbation_mass: &mut CpuField<f32>, base_mass: &mut CpuField<f32>) {
    let shape = perturbation_mass.shape();
    for south_north_index in 0..shape.south_north_points() {
        for west_east_index in 0..shape.west_east_points() {
            let index = south_north_index * shape.west_east_points() + west_east_index;
            perturbation_mass.values_mut()[index] =
                west_east_index as f32 * 0.25 + south_north_index as f32 * 1.5 - 0.3;
            base_mass.values_mut()[index] =
                100.0 + west_east_index as f32 * 0.5 - south_north_index as f32 * 0.75;
        }
    }
}

criterion_group!(benches, benchmark_column_mass_staggering);
criterion_main!(benches);
