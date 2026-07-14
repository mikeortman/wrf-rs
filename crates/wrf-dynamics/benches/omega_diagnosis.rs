//! Criterion throughput benchmark for WRF dry-air omega diagnosis.

// Criterion 0.7 generates a public harness function without a doc comment.
// Rust 1.85 prevents adopting Criterion 0.8, which fixes that warning.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, FieldStorage, GridShape};
use wrf_dynamics::{
    OmegaDiagnosisCoefficients, OmegaDiagnosisGridMetrics, OmegaDiagnosisKernels,
    OmegaDiagnosisMapFactors, OmegaDiagnosisMasses, OmegaDiagnosisRegion, OmegaDiagnosisVelocities,
};

const ACTIVE_WEST_EAST_MASS_POINTS: usize = 256;
const ACTIVE_SOUTH_NORTH_MASS_POINTS: usize = 256;
const ACTIVE_HALF_LEVELS: usize = 40;

fn benchmark_omega_diagnosis(criterion: &mut Criterion) {
    let output_count =
        ACTIVE_WEST_EAST_MASS_POINTS * ACTIVE_SOUTH_NORTH_MASS_POINTS * (ACTIVE_HALF_LEVELS + 1);
    let mut group = criterion.benchmark_group("omega_diagnosis");
    group.throughput(Throughput::Elements(output_count as u64));

    for worker_count in benchmark_worker_counts() {
        let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
        let mut fixture = create_fixture(&backend);
        group.bench_with_input(
            BenchmarkId::new("complete_column", worker_count),
            &worker_count,
            |bencher, _| {
                bencher.iter(|| {
                    apply_fixture(black_box(&backend), black_box(&mut fixture));
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

struct OmegaDiagnosisBenchmarkFixture {
    omega: CpuField<f32>,
    west_east_velocity: CpuField<f32>,
    south_north_velocity: CpuField<f32>,
    perturbation_mass: CpuField<f32>,
    base_mass: CpuField<f32>,
    mass_point_west_east_map_factor: CpuField<f32>,
    west_east_momentum_south_north_map_factor: CpuField<f32>,
    inverse_south_north_momentum_west_east_map_factor: CpuField<f32>,
    half_level_mass_multiplier: Vec<f32>,
    half_level_mass_offset: Vec<f32>,
    eta_layer_thickness: Vec<f32>,
    region: OmegaDiagnosisRegion,
}

fn create_fixture(backend: &CpuBackend) -> OmegaDiagnosisBenchmarkFixture {
    let shape = GridShape::try_new(
        ACTIVE_WEST_EAST_MASS_POINTS + 2,
        ACTIVE_SOUTH_NORTH_MASS_POINTS + 2,
        ACTIVE_HALF_LEVELS + 2,
    )
    .unwrap();
    let horizontal_shape = shape.horizontal_shape();
    let mut fixture = OmegaDiagnosisBenchmarkFixture {
        omega: backend.create_field(shape, -999.0).unwrap(),
        west_east_velocity: backend.create_field(shape, 0.0).unwrap(),
        south_north_velocity: backend.create_field(shape, 0.0).unwrap(),
        perturbation_mass: backend.create_field(horizontal_shape, 0.0).unwrap(),
        base_mass: backend.create_field(horizontal_shape, 0.0).unwrap(),
        mass_point_west_east_map_factor: backend.create_field(horizontal_shape, 0.0).unwrap(),
        west_east_momentum_south_north_map_factor: backend
            .create_field(horizontal_shape, 0.0)
            .unwrap(),
        inverse_south_north_momentum_west_east_map_factor: backend
            .create_field(horizontal_shape, 0.0)
            .unwrap(),
        half_level_mass_multiplier: vec![0.0; shape.bottom_top_points()],
        half_level_mass_offset: vec![0.0; shape.bottom_top_points()],
        eta_layer_thickness: vec![0.0; shape.bottom_top_points()],
        region: OmegaDiagnosisRegion::try_new(
            shape,
            1..(ACTIVE_WEST_EAST_MASS_POINTS + 1),
            1..(ACTIVE_SOUTH_NORTH_MASS_POINTS + 1),
            1..(ACTIVE_HALF_LEVELS + 1),
            1..(ACTIVE_WEST_EAST_MASS_POINTS + 2),
            1..(ACTIVE_SOUTH_NORTH_MASS_POINTS + 2),
            1..(ACTIVE_HALF_LEVELS + 2),
        )
        .unwrap(),
    };
    initialize_fixture(&mut fixture);
    fixture
}

fn initialize_fixture(fixture: &mut OmegaDiagnosisBenchmarkFixture) {
    let shape = fixture.west_east_velocity.shape();
    for bottom_top in 0..shape.bottom_top_points() {
        fixture.half_level_mass_multiplier[bottom_top] = 0.65 + bottom_top as f32 * 0.003;
        fixture.half_level_mass_offset[bottom_top] = 1.4 - bottom_top as f32 * 0.001;
        fixture.eta_layer_thickness[bottom_top] = -0.025 - bottom_top as f32 * 0.000_01;
    }
    for south_north in 0..shape.south_north_points() {
        for west_east in 0..shape.west_east_points() {
            let horizontal_index = south_north * shape.west_east_points() + west_east;
            fixture.perturbation_mass.values_mut()[horizontal_index] =
                -4.0 + west_east as f32 * 0.002 - south_north as f32 * 0.001;
            fixture.base_mass.values_mut()[horizontal_index] =
                95.0 + west_east as f32 * 0.003 + south_north as f32 * 0.005;
            fixture.mass_point_west_east_map_factor.values_mut()[horizontal_index] =
                0.9 + west_east as f32 * 0.000_06 + south_north as f32 * 0.000_03;
            fixture
                .west_east_momentum_south_north_map_factor
                .values_mut()[horizontal_index] =
                1.0 + west_east as f32 * 0.000_1 + south_north as f32 * 0.000_05;
            fixture
                .inverse_south_north_momentum_west_east_map_factor
                .values_mut()[horizontal_index] =
                1.0 / (1.1 + west_east as f32 * 0.000_08 + south_north as f32 * 0.000_04);

            for bottom_top in 0..shape.bottom_top_points() {
                let volume_index = ((south_north * shape.bottom_top_points() + bottom_top)
                    * shape.west_east_points())
                    + west_east;
                fixture.west_east_velocity.values_mut()[volume_index] = -3.0
                    + west_east as f32 * 0.002
                    + bottom_top as f32 * 0.003
                    + south_north as f32 * 0.001;
                fixture.south_north_velocity.values_mut()[volume_index] =
                    2.0 - west_east as f32 * 0.0015 + bottom_top as f32 * 0.0025
                        - south_north as f32 * 0.0005;
            }
        }
    }
}

fn apply_fixture(backend: &CpuBackend, fixture: &mut OmegaDiagnosisBenchmarkFixture) {
    backend
        .diagnose_omega(
            &mut fixture.omega,
            OmegaDiagnosisVelocities::new(
                &fixture.west_east_velocity,
                &fixture.south_north_velocity,
            ),
            OmegaDiagnosisMasses::new(&fixture.perturbation_mass, &fixture.base_mass),
            OmegaDiagnosisMapFactors::new(
                &fixture.mass_point_west_east_map_factor,
                &fixture.west_east_momentum_south_north_map_factor,
                &fixture.inverse_south_north_momentum_west_east_map_factor,
            ),
            OmegaDiagnosisCoefficients::new(
                &fixture.half_level_mass_multiplier,
                &fixture.half_level_mass_offset,
                &fixture.eta_layer_thickness,
            ),
            OmegaDiagnosisGridMetrics::new(0.125, 0.2),
            &fixture.region,
        )
        .unwrap();
}

criterion_group!(benches, benchmark_omega_diagnosis);
criterion_main!(benches);
