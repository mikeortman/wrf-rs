//! Criterion benchmark for WRF acoustic pressure diagnosis.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    AcousticPressureCoefficients, AcousticPressureDampingPhase, AcousticPressureKernels,
    AcousticPressureMasses, AcousticPressureMode, AcousticPressureParameters,
    AcousticPressureRegion, AcousticPressureState, AcousticPressureThermodynamics,
    AcousticPressureVerticalMetrics,
};

const NX: usize = 256;
const NY: usize = 256;
const NZ: usize = 40;

fn benchmark(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("acoustic_pressure");
    group.throughput(Throughput::Elements((NX * NY * NZ) as u64));
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        for mode in [
            AcousticPressureMode::Nonhydrostatic,
            AcousticPressureMode::Hydrostatic,
        ] {
            let mut fixture = create_fixture(&backend);
            group.bench_with_input(
                BenchmarkId::new(mode_name(mode), workers),
                &workers,
                |bencher, _| {
                    bencher.iter(|| {
                        apply_fixture(
                            black_box(&backend),
                            black_box(&mut fixture),
                            black_box(mode),
                        );
                    });
                },
            );
        }
    }
    group.finish();
}

fn mode_name(mode: AcousticPressureMode) -> &'static str {
    match mode {
        AcousticPressureMode::Nonhydrostatic => "nonhydrostatic",
        AcousticPressureMode::Hydrostatic => "hydrostatic",
    }
}

fn worker_counts() -> Vec<usize> {
    let available = thread::available_parallelism().map_or(1, |count| count.get());
    let mut counts = vec![1, available.min(4), available];
    counts.sort_unstable();
    counts.dedup();
    counts
}

struct Fixture {
    state: [CpuField<f32>; 4],
    thermodynamics: [CpuField<f32>; 4],
    masses: [CpuField<f32>; 2],
    coefficients: [Vec<f32>; 3],
    vertical_metrics: [Vec<f32>; 2],
    region: AcousticPressureRegion,
}

fn create_fixture(backend: &CpuBackend) -> Fixture {
    let shape = GridShape::try_new(NX + 2, NY + 2, NZ + 2).unwrap();
    let horizontal_shape = shape.horizontal_shape();
    let volume = |value| backend.create_field(shape, value).unwrap();
    let horizontal = |value| backend.create_field(horizontal_shape, value).unwrap();
    Fixture {
        state: [volume(-999.0), volume(-999.0), volume(900.0), volume(500.0)],
        thermodynamics: [volume(0.8), volume(2.0), volume(1.0), volume(140_000.0)],
        masses: [horizontal(1.0), horizontal(40.0)],
        coefficients: [vec![0.2; NZ + 2], vec![0.4; NZ + 2], vec![1.1; NZ + 2]],
        vertical_metrics: [vec![1.3; NZ + 2], vec![0.7; NZ + 2]],
        region: AcousticPressureRegion::try_new(
            shape,
            1..(NX + 1),
            1..(NY + 1),
            1..(NZ + 1),
            1..(NX + 2),
            1..(NY + 2),
            1..(NZ + 2),
        )
        .unwrap(),
    }
}

fn apply_fixture(backend: &CpuBackend, fixture: &mut Fixture, mode: AcousticPressureMode) {
    let [inverse_density, pressure, geopotential, previous_pressure] = &mut fixture.state;
    backend
        .update_acoustic_pressure(
            AcousticPressureState::new(inverse_density, pressure, geopotential, previous_pressure),
            AcousticPressureThermodynamics::new(
                &fixture.thermodynamics[0],
                &fixture.thermodynamics[1],
                &fixture.thermodynamics[2],
                &fixture.thermodynamics[3],
            ),
            AcousticPressureMasses::new(&fixture.masses[0], &fixture.masses[1]),
            AcousticPressureCoefficients::new(
                &fixture.coefficients[0],
                &fixture.coefficients[1],
                &fixture.coefficients[2],
            ),
            AcousticPressureVerticalMetrics::new(
                &fixture.vertical_metrics[0],
                &fixture.vertical_metrics[1],
            ),
            AcousticPressureParameters::new(300.0, 0.17),
            mode,
            AcousticPressureDampingPhase::Initialize,
            &fixture.region,
        )
        .unwrap();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
