//! Criterion benchmark for the complete WRF acoustic boundary stage.
#![allow(missing_docs)]

use std::hint::black_box;
use std::time::{Duration, Instant};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::CpuBackend;

#[path = "support/acoustic_boundary_stage_fixture.rs"]
mod fixture;

use fixture::AcousticBoundaryStageFixture;

const ACTIVE_WEST_EAST_POINTS: usize = 128;
const ACTIVE_SOUTH_NORTH_POINTS: usize = 128;
const HALF_LEVELS: usize = 40;

fn benchmark(criterion: &mut Criterion) {
    let updated_values = ACTIVE_WEST_EAST_POINTS * ACTIVE_SOUTH_NORTH_POINTS * HALF_LEVELS * 13;
    let mut group = criterion.benchmark_group("acoustic_boundary_stage");
    group.throughput(Throughput::Elements(updated_values as u64));
    for workers in [1, 4, 16] {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = AcousticBoundaryStageFixture::new(
            &backend,
            ACTIVE_WEST_EAST_POINTS,
            ACTIVE_SOUTH_NORTH_POINTS,
            HALF_LEVELS,
        );
        group.bench_with_input(
            BenchmarkId::new("specified_three_substeps", workers),
            &workers,
            |bencher, _| {
                bencher.iter_custom(|iterations| {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        fixture.reset();
                        let started = Instant::now();
                        fixture.apply(black_box(&backend));
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
