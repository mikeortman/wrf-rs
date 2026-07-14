//! Criterion benchmark for the coupled Kessler precipitation trajectory.
#![allow(missing_docs)]

use std::hint::black_box;
use std::time::{Duration, Instant};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::CpuBackend;

#[path = "support/kessler_precipitation_trajectory_fixture.rs"]
mod fixture;

use fixture::KesslerPrecipitationTrajectoryFixture;

fn benchmark(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("kessler_precipitation_trajectory");
    group.throughput(Throughput::Elements(
        KesslerPrecipitationTrajectoryFixture::updated_value_count() as u64,
    ));
    for worker_count in [1, 4, 16] {
        let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
        let mut fixture = KesslerPrecipitationTrajectoryFixture::new(&backend);
        group.bench_with_input(
            BenchmarkId::new("three_step_warm_rain", worker_count),
            &worker_count,
            |bencher, _| {
                bencher.iter_custom(|iteration_count| {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iteration_count {
                        fixture.reset();
                        let started = Instant::now();
                        fixture.apply_three_steps(black_box(&backend));
                        elapsed += started.elapsed();
                    }
                    elapsed
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
