//! Measures warmed steady-state allocations in WRF omega diagnosis.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    OmegaDiagnosisCoefficients, OmegaDiagnosisGridMetrics, OmegaDiagnosisKernels,
    OmegaDiagnosisMapFactors, OmegaDiagnosisMasses, OmegaDiagnosisRegion, OmegaDiagnosisVelocities,
};

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const ACTIVE_WEST_EAST_MASS_POINTS: usize = 256;
const ACTIVE_SOUTH_NORTH_MASS_POINTS: usize = 256;
const ACTIVE_HALF_LEVELS: usize = 40;
const WARMUP_DISPATCHES: usize = 100;
const MEASURED_DISPATCHES: usize = 100;

fn main() {
    println!("kernel,phase,workers,dispatches,allocations,reallocations,bytes_allocated");
    for worker_count in worker_counts() {
        let (first, settled) = measure_worker_count(worker_count);
        print_stats("first", worker_count, first);
        print_stats("settled", worker_count, settled);
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

fn measure_worker_count(worker_count: usize) -> (Stats, Stats) {
    let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
    let mut fixture = create_fixture(&backend);
    run_dispatches(&backend, &mut fixture, WARMUP_DISPATCHES);

    let first_allocations = Region::new(GLOBAL_ALLOCATOR);
    run_dispatches(&backend, &mut fixture, MEASURED_DISPATCHES);
    let first = first_allocations.change();
    let settled_allocations = Region::new(GLOBAL_ALLOCATOR);
    run_dispatches(&backend, &mut fixture, MEASURED_DISPATCHES);
    let settled = settled_allocations.change();
    (first, settled)
}

fn run_dispatches(
    backend: &CpuBackend,
    fixture: &mut OmegaDiagnosisAllocationFixture,
    dispatch_count: usize,
) {
    for _ in 0..dispatch_count {
        apply_fixture(backend, fixture);
    }
}

struct OmegaDiagnosisAllocationFixture {
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

fn create_fixture(backend: &CpuBackend) -> OmegaDiagnosisAllocationFixture {
    let shape = GridShape::try_new(
        ACTIVE_WEST_EAST_MASS_POINTS + 2,
        ACTIVE_SOUTH_NORTH_MASS_POINTS + 2,
        ACTIVE_HALF_LEVELS + 2,
    )
    .unwrap();
    let horizontal_shape = shape.horizontal_shape();
    let mut west_east_velocity = backend.create_field(shape, 0.0).unwrap();
    let mut south_north_velocity = backend.create_field(shape, 0.0).unwrap();
    for (index, value) in west_east_velocity.values_mut().iter_mut().enumerate() {
        *value = index as f32 * 0.000_01 - 3.0;
    }
    for (index, value) in south_north_velocity.values_mut().iter_mut().enumerate() {
        *value = 2.0 - index as f32 * 0.000_005;
    }
    OmegaDiagnosisAllocationFixture {
        omega: backend.create_field(shape, -999.0).unwrap(),
        west_east_velocity,
        south_north_velocity,
        perturbation_mass: backend.create_field(horizontal_shape, -4.0).unwrap(),
        base_mass: backend.create_field(horizontal_shape, 95.0).unwrap(),
        mass_point_west_east_map_factor: backend.create_field(horizontal_shape, 0.95).unwrap(),
        west_east_momentum_south_north_map_factor: backend
            .create_field(horizontal_shape, 1.05)
            .unwrap(),
        inverse_south_north_momentum_west_east_map_factor: backend
            .create_field(horizontal_shape, 0.9)
            .unwrap(),
        half_level_mass_multiplier: vec![0.65; shape.bottom_top_points()],
        half_level_mass_offset: vec![1.4; shape.bottom_top_points()],
        eta_layer_thickness: vec![-0.025; shape.bottom_top_points()],
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
    }
}

fn apply_fixture(backend: &CpuBackend, fixture: &mut OmegaDiagnosisAllocationFixture) {
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

fn print_stats(phase: &str, worker_count: usize, stats: Stats) {
    println!(
        "omega_diagnosis,{phase},{worker_count},{MEASURED_DISPATCHES},{},{},{}",
        stats.allocations, stats.reallocations, stats.bytes_allocated
    );
}

fn assert_allocation_budget(stats: Stats) {
    assert_eq!(stats.reallocations, 0, "kernel dispatch reallocated memory");
    assert!(
        stats.allocations <= MEASURED_DISPATCHES / 10,
        "kernel dispatch allocation rate exceeded one allocation per ten calls"
    );
    assert!(
        stats.bytes_allocated <= 256 * 1_024,
        "kernel dispatch allocated more than 256 KiB per measured batch"
    );
}
