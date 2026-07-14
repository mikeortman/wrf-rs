//! Measures warmed steady-state allocations in Kessler microphysics.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_physics::{
    CpuKesslerMicrophysicsWorkspace, KesslerMicrophysicsFields, KesslerMicrophysicsKernels,
    KesslerMicrophysicsParameters, KesslerMicrophysicsRegion,
};

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const WEST_EAST_POINTS: usize = 128;
const SOUTH_NORTH_POINTS: usize = 128;
const BOTTOM_TOP_POINTS: usize = 40;
const WARMUP_DISPATCHES: usize = 20;
const MEASURED_DISPATCHES: usize = 100;

fn main() {
    println!(
        "kernel,phase,workers,dispatches,allocations,reallocations,bytes_allocated,numeric_scratch_bytes"
    );
    for worker_count in worker_counts() {
        let (first, settled, scratch_bytes) = measure_worker_count(worker_count);
        print_stats("first", worker_count, first, scratch_bytes);
        print_stats("settled", worker_count, settled, scratch_bytes);
        assert_allocation_budget(first);
        assert_allocation_budget(settled);
    }
}

fn worker_counts() -> Vec<usize> {
    let available_workers = std::thread::available_parallelism().map_or(1, |count| count.get());
    let mut counts = vec![1, available_workers.min(4), available_workers];
    counts.sort_unstable();
    counts.dedup();
    counts
}

fn measure_worker_count(worker_count: usize) -> (Stats, Stats, usize) {
    let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
    let mut fixture = KesslerAllocationFixture::new(&backend);
    let scratch_bytes = fixture.workspace.numeric_scratch_byte_count();

    fixture.run_dispatches(&backend, WARMUP_DISPATCHES);
    let first_allocations = Region::new(GLOBAL_ALLOCATOR);
    fixture.run_dispatches(&backend, MEASURED_DISPATCHES);
    let first = first_allocations.change();
    let settled_allocations = Region::new(GLOBAL_ALLOCATOR);
    fixture.run_dispatches(&backend, MEASURED_DISPATCHES);
    let settled = settled_allocations.change();
    (first, settled, scratch_bytes)
}

struct KesslerAllocationFixture {
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
    parameters: KesslerMicrophysicsParameters,
    region: KesslerMicrophysicsRegion,
    workspace: CpuKesslerMicrophysicsWorkspace,
}

impl KesslerAllocationFixture {
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
        let mut height = backend.create_field(shape, 0.0).unwrap();
        for south_north_index in 0..SOUTH_NORTH_POINTS {
            for bottom_top_index in 0..BOTTOM_TOP_POINTS {
                let row_start =
                    (south_north_index * BOTTOM_TOP_POINTS + bottom_top_index) * WEST_EAST_POINTS;
                height.values_mut()[row_start..row_start + WEST_EAST_POINTS]
                    .fill(50.0 + 150.0 * bottom_top_index as f32);
            }
        }

        Self {
            potential_temperature: backend.create_field(shape, 280.0).unwrap(),
            water_vapor_mixing_ratio: backend.create_field(shape, 0.005).unwrap(),
            cloud_water_mixing_ratio: backend.create_field(shape, 0.001).unwrap(),
            rain_water_mixing_ratio: backend.create_field(shape, 0.005).unwrap(),
            dry_air_density: backend.create_field(shape, 1.0).unwrap(),
            exner_function: backend.create_field(shape, 0.95).unwrap(),
            height,
            vertical_layer_thickness: backend.create_field(shape, 150.0).unwrap(),
            accumulated_precipitation: backend
                .create_field(shape.horizontal_shape(), 10.0)
                .unwrap(),
            step_precipitation: backend.create_field(shape.horizontal_shape(), 0.0).unwrap(),
            parameters: KesslerMicrophysicsParameters::try_from_wrf_defaults(60.0).unwrap(),
            workspace: backend
                .create_kessler_microphysics_workspace(&region)
                .unwrap(),
            region,
        }
    }

    fn run_dispatches(&mut self, backend: &CpuBackend, dispatch_count: usize) {
        for _ in 0..dispatch_count {
            self.reset_mutable_fields();
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

    fn reset_mutable_fields(&mut self) {
        self.potential_temperature.values_mut().fill(280.0);
        self.water_vapor_mixing_ratio.values_mut().fill(0.005);
        self.cloud_water_mixing_ratio.values_mut().fill(0.001);
        self.rain_water_mixing_ratio.values_mut().fill(0.005);
        self.accumulated_precipitation.values_mut().fill(10.0);
        self.step_precipitation.values_mut().fill(0.0);
    }
}

fn print_stats(phase: &str, worker_count: usize, stats: Stats, scratch_bytes: usize) {
    println!(
        "kessler_microphysics,{phase},{worker_count},{MEASURED_DISPATCHES},{},{},{},{scratch_bytes}",
        stats.allocations, stats.reallocations, stats.bytes_allocated
    );
}

fn assert_allocation_budget(stats: Stats) {
    assert_eq!(
        stats.reallocations, 0,
        "Kessler dispatch reallocated memory"
    );
    assert!(
        stats.allocations <= MEASURED_DISPATCHES / 5,
        "Kessler dispatch allocation rate exceeded one allocation per five calls"
    );
    assert!(
        stats.bytes_allocated <= 256 * 1_024,
        "Kessler dispatch allocated more than 256 KiB per measured batch"
    );
}
