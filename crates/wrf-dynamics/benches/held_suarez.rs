//! Criterion throughput benchmarks for Held-Suarez momentum damping.

// Criterion 0.7 generates a public harness function without a doc comment.
// Rust 1.85 prevents adopting Criterion 0.8, which fixes that warning.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{HeldSuarezDampingFields, HeldSuarezDampingKernels, HeldSuarezDampingRegion};

const ACTIVE_WEST_EAST_POINTS: usize = 256;
const ACTIVE_BOTTOM_TOP_POINTS: usize = 64;
const ACTIVE_SOUTH_NORTH_POINTS: usize = 64;

fn benchmark_held_suarez(c: &mut Criterion) {
    let momentum_update_count =
        2 * ACTIVE_WEST_EAST_POINTS * ACTIVE_BOTTOM_TOP_POINTS * ACTIVE_SOUTH_NORTH_POINTS;
    let mut group = c.benchmark_group("held_suarez");
    group.throughput(Throughput::Elements(momentum_update_count as u64));

    for worker_count in benchmark_worker_counts() {
        let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
        let fixture = create_fixture(&backend);
        group.bench_with_input(
            BenchmarkId::new("damping", worker_count),
            &worker_count,
            |bencher, _| {
                bencher.iter_batched_ref(
                    || {
                        (
                            fixture.west_east_momentum_tendency.clone(),
                            fixture.south_north_momentum_tendency.clone(),
                        )
                    },
                    |(west_east_momentum_tendency, south_north_momentum_tendency)| {
                        backend
                            .apply_held_suarez_damping(
                                HeldSuarezDampingFields::new(
                                    black_box(west_east_momentum_tendency),
                                    black_box(south_north_momentum_tendency),
                                    black_box(&fixture.west_east_momentum),
                                    black_box(&fixture.south_north_momentum),
                                    black_box(&fixture.perturbation_pressure),
                                    black_box(&fixture.base_pressure),
                                ),
                                black_box(&fixture.region),
                            )
                            .unwrap();
                    },
                    BatchSize::LargeInput,
                );
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

struct HeldSuarezBenchmarkFixture {
    west_east_momentum_tendency: CpuField<f32>,
    south_north_momentum_tendency: CpuField<f32>,
    west_east_momentum: CpuField<f32>,
    south_north_momentum: CpuField<f32>,
    perturbation_pressure: CpuField<f32>,
    base_pressure: CpuField<f32>,
    region: HeldSuarezDampingRegion,
}

fn create_fixture(backend: &CpuBackend) -> HeldSuarezBenchmarkFixture {
    let west_east_points = ACTIVE_WEST_EAST_POINTS + 2;
    let bottom_top_points = ACTIVE_BOTTOM_TOP_POINTS + 1;
    let south_north_points = ACTIVE_SOUTH_NORTH_POINTS + 2;
    let shape =
        GridShape::try_new(west_east_points, south_north_points, bottom_top_points).unwrap();
    let mut fixture = HeldSuarezBenchmarkFixture {
        west_east_momentum_tendency: backend.create_field(shape, 0.0).unwrap(),
        south_north_momentum_tendency: backend.create_field(shape, 0.0).unwrap(),
        west_east_momentum: backend.create_field(shape, 0.0).unwrap(),
        south_north_momentum: backend.create_field(shape, 0.0).unwrap(),
        perturbation_pressure: backend.create_field(shape, 0.0).unwrap(),
        base_pressure: backend.create_field(shape, 0.0).unwrap(),
        region: HeldSuarezDampingRegion::try_new(
            shape,
            1..(ACTIVE_WEST_EAST_POINTS + 1),
            1..(ACTIVE_BOTTOM_TOP_POINTS + 1),
            1..(ACTIVE_SOUTH_NORTH_POINTS + 1),
            1..(ACTIVE_SOUTH_NORTH_POINTS + 1),
            1,
        )
        .unwrap(),
    };

    for south_north_index in 0..south_north_points {
        for bottom_top_index in 0..bottom_top_points {
            for west_east_index in 0..west_east_points {
                let index = (south_north_index * bottom_top_points + bottom_top_index)
                    * west_east_points
                    + west_east_index;
                let horizontal_variation =
                    (west_east_index as f32 * 0.125) + (south_north_index as f32 * 0.25);
                fixture.perturbation_pressure.values_mut()[index] = horizontal_variation;
                fixture.base_pressure.values_mut()[index] =
                    100_500.0 - bottom_top_index as f32 * 500.0;
                fixture.west_east_momentum.values_mut()[index] = 10.0 + horizontal_variation * 0.01;
                fixture.south_north_momentum.values_mut()[index] =
                    -7.0 + horizontal_variation * 0.02;
                fixture.west_east_momentum_tendency.values_mut()[index] = 0.001;
                fixture.south_north_momentum_tendency.values_mut()[index] = -0.002;
            }
        }
    }

    fixture
}

criterion_group!(benches, benchmark_held_suarez);
criterion_main!(benches);
