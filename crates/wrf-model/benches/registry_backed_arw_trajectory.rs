//! Criterion benchmark for the Registry-backed accepted ARW trajectory.
#![allow(missing_docs)]

use std::hint::black_box;
use std::time::{Duration, Instant};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::CpuBackend;
use wrf_model::ArwMassField;

mod support;

use support::registry_backed_arw_trajectory_fixture::RegistryBackedArwTrajectoryFixture;

fn registry_backed_arw_trajectory(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("registry_backed_arw_trajectory");
    group.throughput(Throughput::Elements(62 * 62 * 38));

    for worker_count in [1, 4] {
        let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
        let mut fixture = RegistryBackedArwTrajectoryFixture::new(&backend);
        fixture
            .model
            .advance_short_trajectory(&backend, &mut fixture.state, &mut fixture.workspace)
            .unwrap();

        group.bench_with_input(
            BenchmarkId::new("canonical_continuous", worker_count),
            &worker_count,
            |bencher, _| {
                bencher.iter_custom(|iteration_count| {
                    let mut measured = Duration::ZERO;
                    for _ in 0..iteration_count {
                        fixture.reset();
                        let started = Instant::now();
                        fixture
                            .model
                            .advance_short_trajectory(
                                &backend,
                                &mut fixture.state,
                                &mut fixture.workspace,
                            )
                            .unwrap();
                        measured += started.elapsed();
                        black_box(
                            fixture
                                .state
                                .mass_field_values(ArwMassField::CurrentPotentialTemperature),
                        );
                    }
                    measured
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, registry_backed_arw_trajectory);
criterion_main!(benches);
