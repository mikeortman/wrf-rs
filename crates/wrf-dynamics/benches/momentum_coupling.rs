//! Criterion throughput benchmark for WRF momentum coupling.

// Criterion 0.7 generates a public harness function without a doc comment.
// Rust 1.85 prevents adopting Criterion 0.8, which fixes that warning.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, FieldStorage, GridShape};
use wrf_dynamics::{
    MomentumCouplingCoefficients, MomentumCouplingKernels, MomentumCouplingMapFactors,
    MomentumCouplingMasses, MomentumCouplingOutputs, MomentumCouplingRegion,
    MomentumCouplingVelocities,
};

const ACTIVE_WEST_EAST_MASS_POINTS: usize = 256;
const ACTIVE_SOUTH_NORTH_MASS_POINTS: usize = 256;
const ACTIVE_HALF_LEVELS: usize = 40;

fn benchmark_momentum_coupling(criterion: &mut Criterion) {
    let output_count = (ACTIVE_WEST_EAST_MASS_POINTS + 1)
        * ACTIVE_SOUTH_NORTH_MASS_POINTS
        * ACTIVE_HALF_LEVELS
        + ACTIVE_WEST_EAST_MASS_POINTS * (ACTIVE_SOUTH_NORTH_MASS_POINTS + 1) * ACTIVE_HALF_LEVELS
        + ACTIVE_WEST_EAST_MASS_POINTS * ACTIVE_SOUTH_NORTH_MASS_POINTS * (ACTIVE_HALF_LEVELS + 1);
    let mut group = criterion.benchmark_group("momentum_coupling");
    group.throughput(Throughput::Elements(output_count as u64));

    for worker_count in benchmark_worker_counts() {
        let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
        let mut fixture = create_fixture(&backend);
        group.bench_with_input(
            BenchmarkId::new("all_upper_staggers", worker_count),
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

struct MomentumCouplingBenchmarkFixture {
    west_east_output: CpuField<f32>,
    south_north_output: CpuField<f32>,
    vertical_output: CpuField<f32>,
    west_east_velocity: CpuField<f32>,
    south_north_velocity: CpuField<f32>,
    vertical_velocity: CpuField<f32>,
    west_east_mass: CpuField<f32>,
    south_north_mass: CpuField<f32>,
    mass_point_mass: CpuField<f32>,
    west_east_map_factor: CpuField<f32>,
    inverse_south_north_map_factor: CpuField<f32>,
    mass_point_map_factor: CpuField<f32>,
    half_level_mass_multiplier: Vec<f32>,
    half_level_offset: Vec<f32>,
    full_level_mass_multiplier: Vec<f32>,
    full_level_offset: Vec<f32>,
    region: MomentumCouplingRegion,
}

fn create_fixture(backend: &CpuBackend) -> MomentumCouplingBenchmarkFixture {
    let shape = GridShape::try_new(
        ACTIVE_WEST_EAST_MASS_POINTS + 2,
        ACTIVE_SOUTH_NORTH_MASS_POINTS + 2,
        ACTIVE_HALF_LEVELS + 2,
    )
    .unwrap();
    let horizontal_shape = shape.horizontal_shape();
    let mut fixture = MomentumCouplingBenchmarkFixture {
        west_east_output: backend.create_field(shape, -999.0).unwrap(),
        south_north_output: backend.create_field(shape, -999.0).unwrap(),
        vertical_output: backend.create_field(shape, -999.0).unwrap(),
        west_east_velocity: backend.create_field(shape, 0.0).unwrap(),
        south_north_velocity: backend.create_field(shape, 0.0).unwrap(),
        vertical_velocity: backend.create_field(shape, 0.0).unwrap(),
        west_east_mass: backend.create_field(horizontal_shape, 0.0).unwrap(),
        south_north_mass: backend.create_field(horizontal_shape, 0.0).unwrap(),
        mass_point_mass: backend.create_field(horizontal_shape, 0.0).unwrap(),
        west_east_map_factor: backend.create_field(horizontal_shape, 0.0).unwrap(),
        inverse_south_north_map_factor: backend.create_field(horizontal_shape, 0.0).unwrap(),
        mass_point_map_factor: backend.create_field(horizontal_shape, 0.0).unwrap(),
        half_level_mass_multiplier: vec![0.0; shape.bottom_top_points()],
        half_level_offset: vec![0.0; shape.bottom_top_points()],
        full_level_mass_multiplier: vec![0.0; shape.bottom_top_points()],
        full_level_offset: vec![0.0; shape.bottom_top_points()],
        region: MomentumCouplingRegion::try_new(
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

fn initialize_fixture(fixture: &mut MomentumCouplingBenchmarkFixture) {
    let shape = fixture.west_east_velocity.shape();
    for bottom_top in 0..shape.bottom_top_points() {
        fixture.half_level_mass_multiplier[bottom_top] = 0.7 + bottom_top as f32 * 0.003;
        fixture.half_level_offset[bottom_top] = 1.5 - bottom_top as f32 * 0.001;
        fixture.full_level_mass_multiplier[bottom_top] = 0.6 + bottom_top as f32 * 0.002;
        fixture.full_level_offset[bottom_top] = 2.0 + bottom_top as f32 * 0.0015;
    }
    for south_north in 0..shape.south_north_points() {
        for west_east in 0..shape.west_east_points() {
            let horizontal_index = south_north * shape.west_east_points() + west_east;
            fixture.west_east_mass.values_mut()[horizontal_index] =
                80.0 + west_east as f32 * 0.005 + south_north as f32 * 0.0125;
            fixture.south_north_mass.values_mut()[horizontal_index] =
                85.0 + west_east as f32 * 0.0075 - south_north as f32 * 0.005;
            fixture.mass_point_mass.values_mut()[horizontal_index] =
                90.0 + west_east as f32 * 0.0025 + south_north as f32 * 0.008;
            fixture.west_east_map_factor.values_mut()[horizontal_index] =
                1.0 + west_east as f32 * 0.0001 + south_north as f32 * 0.00005;
            fixture.inverse_south_north_map_factor.values_mut()[horizontal_index] =
                1.0 / (1.1 + west_east as f32 * 0.00008 + south_north as f32 * 0.00004);
            fixture.mass_point_map_factor.values_mut()[horizontal_index] =
                0.9 + west_east as f32 * 0.00006 + south_north as f32 * 0.00003;
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
                fixture.vertical_velocity.values_mut()[volume_index] =
                    0.5 + west_east as f32 * 0.0008 - bottom_top as f32 * 0.0012
                        + south_north as f32 * 0.0007;
            }
        }
    }
}

fn apply_fixture(backend: &CpuBackend, fixture: &mut MomentumCouplingBenchmarkFixture) {
    backend
        .couple_momentum(
            MomentumCouplingOutputs::new(
                &mut fixture.west_east_output,
                &mut fixture.south_north_output,
                &mut fixture.vertical_output,
            ),
            MomentumCouplingVelocities::new(
                &fixture.west_east_velocity,
                &fixture.south_north_velocity,
                &fixture.vertical_velocity,
            ),
            MomentumCouplingMasses::new(
                &fixture.west_east_mass,
                &fixture.south_north_mass,
                &fixture.mass_point_mass,
            ),
            MomentumCouplingMapFactors::new(
                &fixture.west_east_map_factor,
                &fixture.inverse_south_north_map_factor,
                &fixture.mass_point_map_factor,
            ),
            MomentumCouplingCoefficients::new(
                &fixture.half_level_mass_multiplier,
                &fixture.half_level_offset,
                &fixture.full_level_mass_multiplier,
                &fixture.full_level_offset,
            ),
            &fixture.region,
        )
        .unwrap();
}

criterion_group!(benches, benchmark_momentum_coupling);
criterion_main!(benches);
