//! Criterion benchmark for WRF vertical acoustic coefficient construction.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    VerticalAcousticCoefficientInputs, VerticalAcousticCoefficientKernels,
    VerticalAcousticCoefficientParameters, VerticalAcousticCoefficientRegion,
    VerticalAcousticMassCoefficients, VerticalAcousticMetrics, VerticalAcousticSolveCoefficients,
    VerticalAcousticTopBoundary,
};

const WEST_EAST_POINTS: usize = 256;
const SOUTH_NORTH_POINTS: usize = 256;
const HALF_LEVELS: usize = 40;

fn benchmark(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("vertical_acoustic_coefficients");
    group.throughput(Throughput::Elements(
        (WEST_EAST_POINTS * SOUTH_NORTH_POINTS * (HALF_LEVELS + 1)) as u64,
    ));
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = create_fixture(&backend);
        group.bench_with_input(
            BenchmarkId::new("nonrigid", workers),
            &workers,
            |bencher, _| {
                bencher.iter(|| {
                    apply_fixture(black_box(&backend), black_box(&mut fixture));
                });
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
    outputs: [CpuField<f32>; 3],
    full_column_mass: CpuField<f32>,
    vertical_moisture_coefficient: CpuField<f32>,
    pressure_coefficient: CpuField<f32>,
    mass_coefficients: [Vec<f32>; 4],
    metrics: [Vec<f32>; 2],
    region: VerticalAcousticCoefficientRegion,
}

fn create_fixture(backend: &CpuBackend) -> Fixture {
    let shape = GridShape::try_new(WEST_EAST_POINTS, SOUTH_NORTH_POINTS, HALF_LEVELS + 1).unwrap();
    let volume = |value| backend.create_field(shape, value).unwrap();
    Fixture {
        outputs: [volume(-901.0), volume(-902.0), volume(-903.0)],
        full_column_mass: backend
            .create_field(shape.horizontal_shape(), 40.0)
            .unwrap(),
        vertical_moisture_coefficient: volume(0.9),
        pressure_coefficient: volume(140_000.0),
        mass_coefficients: [
            vec![0.2; HALF_LEVELS + 1],
            vec![0.4; HALF_LEVELS + 1],
            vec![0.25; HALF_LEVELS + 1],
            vec![0.35; HALF_LEVELS + 1],
        ],
        metrics: [vec![1.1; HALF_LEVELS + 1], vec![1.3; HALF_LEVELS + 1]],
        region: VerticalAcousticCoefficientRegion::try_new(
            shape,
            0..WEST_EAST_POINTS,
            0..SOUTH_NORTH_POINTS,
            0..HALF_LEVELS,
            0..WEST_EAST_POINTS,
            0..SOUTH_NORTH_POINTS,
        )
        .unwrap(),
    }
}

fn apply_fixture(backend: &CpuBackend, fixture: &mut Fixture) {
    let [lower, inverse_diagonal, upper] = &mut fixture.outputs;
    backend
        .calculate_vertical_acoustic_coefficients(
            VerticalAcousticSolveCoefficients::new(lower, inverse_diagonal, upper),
            VerticalAcousticCoefficientInputs::new(
                &fixture.full_column_mass,
                &fixture.vertical_moisture_coefficient,
                &fixture.pressure_coefficient,
            ),
            VerticalAcousticMassCoefficients::new(
                &fixture.mass_coefficients[0],
                &fixture.mass_coefficients[1],
                &fixture.mass_coefficients[2],
                &fixture.mass_coefficients[3],
            ),
            VerticalAcousticMetrics::new(&fixture.metrics[0], &fixture.metrics[1]),
            VerticalAcousticCoefficientParameters::new(
                2.5,
                9.81,
                0.1,
                VerticalAcousticTopBoundary::Nonrigid,
            ),
            &fixture.region,
        )
        .unwrap();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
