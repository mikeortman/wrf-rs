//! Criterion throughput benchmark for Kessler warm-rain microphysics.

// Criterion 0.7 generates a public harness function without a doc comment.
// Rust 1.85 prevents adopting Criterion 0.8, which fixes that warning.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;
use std::time::{Duration, Instant};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_physics::{
    CpuKesslerMicrophysicsWorkspace, KesslerMicrophysicsFields, KesslerMicrophysicsKernels,
    KesslerMicrophysicsParameters, KesslerMicrophysicsRegion,
};

const WEST_EAST_POINTS: usize = 128;
const SOUTH_NORTH_POINTS: usize = 128;
const BOTTOM_TOP_POINTS: usize = 40;

fn benchmark_kessler_microphysics(criterion: &mut Criterion) {
    let point_count = WEST_EAST_POINTS * SOUTH_NORTH_POINTS * BOTTOM_TOP_POINTS;
    let mut group = criterion.benchmark_group("kessler_microphysics");
    group.throughput(Throughput::Elements(point_count as u64));

    for worker_count in benchmark_worker_counts() {
        let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
        let mut fixture = KesslerBenchmarkFixture::new(&backend);
        group.bench_with_input(
            BenchmarkId::new("mixed_warm_rain", worker_count),
            &worker_count,
            |bencher, _| {
                bencher.iter_custom(|iteration_count| {
                    let mut measured_duration = Duration::ZERO;
                    for _ in 0..iteration_count {
                        fixture.reset_mutable_fields();
                        let started_at = Instant::now();
                        fixture.apply(black_box(&backend));
                        measured_duration += started_at.elapsed();
                    }
                    measured_duration
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

struct KesslerBenchmarkFixture {
    potential_temperature: CpuField<f32>,
    water_vapor_mixing_ratio: CpuField<f32>,
    cloud_water_mixing_ratio: CpuField<f32>,
    rain_water_mixing_ratio: CpuField<f32>,
    dry_air_density: CpuField<f32>,
    exner_function: CpuField<f32>,
    height: CpuField<f32>,
    vertical_layer_thickness: CpuField<f32>,
    accumulated_precipitation: CpuField<f32>,
    step_precipitation: CpuField<f32>,
    initial_potential_temperature: Vec<f32>,
    initial_water_vapor_mixing_ratio: Vec<f32>,
    initial_cloud_water_mixing_ratio: Vec<f32>,
    initial_rain_water_mixing_ratio: Vec<f32>,
    initial_accumulated_precipitation: Vec<f32>,
    initial_step_precipitation: Vec<f32>,
    parameters: KesslerMicrophysicsParameters,
    region: KesslerMicrophysicsRegion,
    workspace: CpuKesslerMicrophysicsWorkspace,
}

impl KesslerBenchmarkFixture {
    fn new(backend: &CpuBackend) -> Self {
        let shape =
            GridShape::try_new(WEST_EAST_POINTS, SOUTH_NORTH_POINTS, BOTTOM_TOP_POINTS).unwrap();
        let region = KesslerMicrophysicsRegion::try_new(
            shape,
            0..WEST_EAST_POINTS,
            0..SOUTH_NORTH_POINTS,
            0..BOTTOM_TOP_POINTS,
        )
        .unwrap();
        let mut fixture = Self {
            potential_temperature: backend.create_field(shape, 0.0).unwrap(),
            water_vapor_mixing_ratio: backend.create_field(shape, 0.0).unwrap(),
            cloud_water_mixing_ratio: backend.create_field(shape, 0.0).unwrap(),
            rain_water_mixing_ratio: backend.create_field(shape, 0.0).unwrap(),
            dry_air_density: backend.create_field(shape, 0.0).unwrap(),
            exner_function: backend.create_field(shape, 0.0).unwrap(),
            height: backend.create_field(shape, 0.0).unwrap(),
            vertical_layer_thickness: backend.create_field(shape, 0.0).unwrap(),
            accumulated_precipitation: backend.create_field(shape.horizontal_shape(), 0.0).unwrap(),
            step_precipitation: backend.create_field(shape.horizontal_shape(), 0.0).unwrap(),
            initial_potential_temperature: Vec::new(),
            initial_water_vapor_mixing_ratio: Vec::new(),
            initial_cloud_water_mixing_ratio: Vec::new(),
            initial_rain_water_mixing_ratio: Vec::new(),
            initial_accumulated_precipitation: Vec::new(),
            initial_step_precipitation: Vec::new(),
            parameters: KesslerMicrophysicsParameters::try_from_wrf_defaults(60.0).unwrap(),
            workspace: backend
                .create_kessler_microphysics_workspace(&region)
                .unwrap(),
            region,
        };
        fixture.initialize_fields();
        fixture.capture_initial_mutable_fields();
        fixture
    }

    fn initialize_fields(&mut self) {
        let shape = self.region.field_shape();
        for south_north_index in 0..shape.south_north_points() {
            for bottom_top_index in 0..shape.bottom_top_points() {
                for west_east_index in 0..shape.west_east_points() {
                    let index =
                        linear_index(shape, west_east_index, bottom_top_index, south_north_index);
                    self.potential_temperature.values_mut()[index] =
                        278.0 + 0.007 * west_east_index as f32 + 0.03 * bottom_top_index as f32
                            - 0.004 * south_north_index as f32;
                    self.water_vapor_mixing_ratio.values_mut()[index] =
                        0.002 + 0.001 * ((west_east_index + 2 * bottom_top_index) % 8) as f32;
                    self.cloud_water_mixing_ratio.values_mut()[index] =
                        if (west_east_index + bottom_top_index) % 3 == 0 {
                            0.002
                        } else {
                            0.0002
                        };
                    self.rain_water_mixing_ratio.values_mut()[index] =
                        [0.0, 0.0005, 0.005, 0.02][(west_east_index + south_north_index) % 4];
                    self.dry_air_density.values_mut()[index] =
                        1.15 - 0.008 * bottom_top_index as f32;
                    self.exner_function.values_mut()[index] =
                        0.99 - 0.0015 * bottom_top_index as f32;
                    self.height.values_mut()[index] = 50.0 + 150.0 * bottom_top_index as f32;
                    self.vertical_layer_thickness.values_mut()[index] = 150.0;
                }
            }
        }
        self.accumulated_precipitation.values_mut().fill(10.0);
        self.step_precipitation.values_mut().fill(0.0);
    }

    fn capture_initial_mutable_fields(&mut self) {
        self.initial_potential_temperature = self.potential_temperature.values().to_vec();
        self.initial_water_vapor_mixing_ratio = self.water_vapor_mixing_ratio.values().to_vec();
        self.initial_cloud_water_mixing_ratio = self.cloud_water_mixing_ratio.values().to_vec();
        self.initial_rain_water_mixing_ratio = self.rain_water_mixing_ratio.values().to_vec();
        self.initial_accumulated_precipitation = self.accumulated_precipitation.values().to_vec();
        self.initial_step_precipitation = self.step_precipitation.values().to_vec();
    }

    fn reset_mutable_fields(&mut self) {
        self.potential_temperature
            .values_mut()
            .copy_from_slice(&self.initial_potential_temperature);
        self.water_vapor_mixing_ratio
            .values_mut()
            .copy_from_slice(&self.initial_water_vapor_mixing_ratio);
        self.cloud_water_mixing_ratio
            .values_mut()
            .copy_from_slice(&self.initial_cloud_water_mixing_ratio);
        self.rain_water_mixing_ratio
            .values_mut()
            .copy_from_slice(&self.initial_rain_water_mixing_ratio);
        self.accumulated_precipitation
            .values_mut()
            .copy_from_slice(&self.initial_accumulated_precipitation);
        self.step_precipitation
            .values_mut()
            .copy_from_slice(&self.initial_step_precipitation);
    }

    fn apply(&mut self, backend: &CpuBackend) {
        backend
            .apply_kessler_microphysics(
                KesslerMicrophysicsFields::new(
                    &mut self.potential_temperature,
                    &mut self.water_vapor_mixing_ratio,
                    &mut self.cloud_water_mixing_ratio,
                    &mut self.rain_water_mixing_ratio,
                    &self.dry_air_density,
                    &self.exner_function,
                    &self.height,
                    &self.vertical_layer_thickness,
                    &mut self.accumulated_precipitation,
                    &mut self.step_precipitation,
                ),
                self.parameters,
                &self.region,
                &mut self.workspace,
            )
            .unwrap();
    }
}

fn linear_index(
    shape: GridShape,
    west_east_index: usize,
    bottom_top_index: usize,
    south_north_index: usize,
) -> usize {
    (south_north_index * shape.bottom_top_points() + bottom_top_index) * shape.west_east_points()
        + west_east_index
}

criterion_group!(benches, benchmark_kessler_microphysics);
criterion_main!(benches);
