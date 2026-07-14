//! Criterion throughput benchmark for WRF full inverse density.

// Criterion 0.7 generates a public harness function without a doc comment.
// Rust 1.85 prevents adopting Criterion 0.8, which fixes that warning.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{InverseDensityKernels, InverseDensityRegion};

const ACTIVE_WEST_EAST_POINTS: usize = 256;
const ACTIVE_SOUTH_NORTH_POINTS: usize = 256;
const ACTIVE_BOTTOM_TOP_POINTS: usize = 40;

fn benchmark_inverse_density(criterion: &mut Criterion) {
    let output_count =
        ACTIVE_WEST_EAST_POINTS * ACTIVE_SOUTH_NORTH_POINTS * ACTIVE_BOTTOM_TOP_POINTS;
    let mut group = criterion.benchmark_group("inverse_density");
    group.throughput(Throughput::Elements(output_count as u64));

    for worker_count in benchmark_worker_counts() {
        let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
        let mut fixture = create_fixture(&backend);
        group.bench_with_input(
            BenchmarkId::new("full_mass_tile", worker_count),
            &worker_count,
            |bencher, _| {
                bencher.iter(|| apply_fixture(black_box(&backend), black_box(&mut fixture)));
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

struct InverseDensityBenchmarkFixture {
    full_inverse_density: CpuField<f32>,
    perturbation_inverse_density: CpuField<f32>,
    base_state_inverse_density: CpuField<f32>,
    region: InverseDensityRegion,
}

fn create_fixture(backend: &CpuBackend) -> InverseDensityBenchmarkFixture {
    let shape = GridShape::try_new(
        ACTIVE_WEST_EAST_POINTS + 2,
        ACTIVE_SOUTH_NORTH_POINTS + 2,
        ACTIVE_BOTTOM_TOP_POINTS + 2,
    )
    .unwrap();
    let mut perturbation_inverse_density = backend.create_field(shape, 0.0).unwrap();
    let mut base_state_inverse_density = backend.create_field(shape, 0.0).unwrap();
    for south_north in 0..shape.south_north_points() {
        for bottom_top in 0..shape.bottom_top_points() {
            for west_east in 0..shape.west_east_points() {
                let index = volume_index(west_east, bottom_top, south_north, shape);
                perturbation_inverse_density.values_mut()[index] =
                    ((0.125 + west_east as f32 * 0.000_031) - bottom_top as f32 * 0.000_017)
                        + south_north as f32 * 0.000_009;
                base_state_inverse_density.values_mut()[index] =
                    ((0.875 - west_east as f32 * 0.000_023) + bottom_top as f32 * 0.000_011)
                        - south_north as f32 * 0.000_007;
            }
        }
    }

    InverseDensityBenchmarkFixture {
        full_inverse_density: backend.create_field(shape, -999.0).unwrap(),
        perturbation_inverse_density,
        base_state_inverse_density,
        region: InverseDensityRegion::try_new(
            shape,
            1..(ACTIVE_WEST_EAST_POINTS + 1),
            1..(ACTIVE_SOUTH_NORTH_POINTS + 1),
            1..(ACTIVE_BOTTOM_TOP_POINTS + 1),
            1..(ACTIVE_WEST_EAST_POINTS + 2),
            1..(ACTIVE_SOUTH_NORTH_POINTS + 2),
            1..(ACTIVE_BOTTOM_TOP_POINTS + 2),
        )
        .unwrap(),
    }
}

fn apply_fixture(backend: &CpuBackend, fixture: &mut InverseDensityBenchmarkFixture) {
    backend
        .calculate_full_inverse_density(
            &mut fixture.full_inverse_density,
            &fixture.perturbation_inverse_density,
            &fixture.base_state_inverse_density,
            &fixture.region,
        )
        .unwrap();
}

fn volume_index(
    west_east: usize,
    bottom_top: usize,
    south_north: usize,
    shape: GridShape,
) -> usize {
    (south_north * shape.bottom_top_points() + bottom_top) * shape.west_east_points() + west_east
}

criterion_group!(benches, benchmark_inverse_density);
criterion_main!(benches);
