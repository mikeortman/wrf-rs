//! Criterion throughput benchmark for WRF moisture momentum coefficients.

// Criterion 0.7 generates a public harness function without a doc comment.
// Rust 1.85 prevents adopting Criterion 0.8, which fixes that warning.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    MoistureCoefficientKernels, MoistureCoefficientOutputs, MoistureCoefficientRegion,
    MoistureSpecies,
};

const ACTIVE_WEST_EAST_MASS_POINTS: usize = 256;
const ACTIVE_SOUTH_NORTH_MASS_POINTS: usize = 256;
const ACTIVE_HALF_LEVELS: usize = 40;
const ACTIVE_SPECIES_COUNT: usize = 6;

fn benchmark_moisture_coefficients(criterion: &mut Criterion) {
    let horizontal_staggered_outputs = (ACTIVE_WEST_EAST_MASS_POINTS + 1)
        * ACTIVE_SOUTH_NORTH_MASS_POINTS
        + ACTIVE_WEST_EAST_MASS_POINTS * (ACTIVE_SOUTH_NORTH_MASS_POINTS + 1);
    let output_count = horizontal_staggered_outputs * ACTIVE_HALF_LEVELS
        + ACTIVE_WEST_EAST_MASS_POINTS * ACTIVE_SOUTH_NORTH_MASS_POINTS * (ACTIVE_HALF_LEVELS - 1);
    let mut group = criterion.benchmark_group("moisture_coefficients");
    group.throughput(Throughput::Elements(output_count as u64));

    for worker_count in benchmark_worker_counts() {
        let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
        let mut fixture = create_fixture(&backend);
        group.bench_with_input(
            BenchmarkId::new("six_active_species", worker_count),
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

struct MoistureCoefficientBenchmarkFixture {
    west_east_output: CpuField<f32>,
    south_north_output: CpuField<f32>,
    vertical_output: CpuField<f32>,
    active_species: Vec<CpuField<f32>>,
    region: MoistureCoefficientRegion,
}

fn create_fixture(backend: &CpuBackend) -> MoistureCoefficientBenchmarkFixture {
    let shape = GridShape::try_new(
        ACTIVE_WEST_EAST_MASS_POINTS + 2,
        ACTIVE_SOUTH_NORTH_MASS_POINTS + 2,
        ACTIVE_HALF_LEVELS + 2,
    )
    .unwrap();
    let mut active_species = Vec::with_capacity(ACTIVE_SPECIES_COUNT);
    for active_species_index in 0..ACTIVE_SPECIES_COUNT {
        let mut field = backend.create_field(shape, 0.0).unwrap();
        for south_north in 0..shape.south_north_points() {
            for bottom_top in 0..shape.bottom_top_points() {
                for west_east in 0..shape.west_east_points() {
                    let index = volume_index(west_east, bottom_top, south_north, shape);
                    field.values_mut()[index] = 0.000_5 * (active_species_index + 1) as f32
                        + 0.000_001 * west_east as f32
                        - 0.000_000_5 * bottom_top as f32
                        + 0.000_000_25 * south_north as f32;
                }
            }
        }
        active_species.push(field);
    }

    MoistureCoefficientBenchmarkFixture {
        west_east_output: backend.create_field(shape, -999.0).unwrap(),
        south_north_output: backend.create_field(shape, -999.0).unwrap(),
        vertical_output: backend.create_field(shape, -999.0).unwrap(),
        active_species,
        region: MoistureCoefficientRegion::try_new(
            shape,
            1..(ACTIVE_WEST_EAST_MASS_POINTS + 1),
            1..(ACTIVE_SOUTH_NORTH_MASS_POINTS + 1),
            1..(ACTIVE_HALF_LEVELS + 1),
            1..(ACTIVE_WEST_EAST_MASS_POINTS + 2),
            1..(ACTIVE_SOUTH_NORTH_MASS_POINTS + 2),
            1..(ACTIVE_HALF_LEVELS + 2),
        )
        .unwrap(),
    }
}

fn apply_fixture(backend: &CpuBackend, fixture: &mut MoistureCoefficientBenchmarkFixture) {
    backend
        .calculate_moisture_coefficients(
            MoistureCoefficientOutputs::new(
                &mut fixture.west_east_output,
                &mut fixture.south_north_output,
                &mut fixture.vertical_output,
            ),
            MoistureSpecies::new(&fixture.active_species),
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

criterion_group!(benches, benchmark_moisture_coefficients);
criterion_main!(benches);
