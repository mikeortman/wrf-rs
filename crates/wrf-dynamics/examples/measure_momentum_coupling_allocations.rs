//! Measures warmed steady-state allocations in WRF momentum coupling.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    MomentumCouplingCoefficients, MomentumCouplingKernels, MomentumCouplingMapFactors,
    MomentumCouplingMasses, MomentumCouplingOutputs, MomentumCouplingRegion,
    MomentumCouplingVelocities,
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
    fixture: &mut MomentumCouplingAllocationFixture,
    dispatch_count: usize,
) {
    for _ in 0..dispatch_count {
        apply_fixture(backend, fixture);
    }
}

struct MomentumCouplingAllocationFixture {
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

fn create_fixture(backend: &CpuBackend) -> MomentumCouplingAllocationFixture {
    let shape = GridShape::try_new(
        ACTIVE_WEST_EAST_MASS_POINTS + 2,
        ACTIVE_SOUTH_NORTH_MASS_POINTS + 2,
        ACTIVE_HALF_LEVELS + 2,
    )
    .unwrap();
    let horizontal_shape = shape.horizontal_shape();
    let mut west_east_velocity = backend.create_field(shape, 0.0).unwrap();
    let mut south_north_velocity = backend.create_field(shape, 0.0).unwrap();
    let mut vertical_velocity = backend.create_field(shape, 0.0).unwrap();
    for (index, value) in west_east_velocity.values_mut().iter_mut().enumerate() {
        *value = index as f32 * 0.000_01 - 3.0;
    }
    for (index, value) in south_north_velocity.values_mut().iter_mut().enumerate() {
        *value = 2.0 - index as f32 * 0.000_005;
    }
    for (index, value) in vertical_velocity.values_mut().iter_mut().enumerate() {
        *value = 0.5 + index as f32 * 0.000_001;
    }
    MomentumCouplingAllocationFixture {
        west_east_output: backend.create_field(shape, -999.0).unwrap(),
        south_north_output: backend.create_field(shape, -999.0).unwrap(),
        vertical_output: backend.create_field(shape, -999.0).unwrap(),
        west_east_velocity,
        south_north_velocity,
        vertical_velocity,
        west_east_mass: backend.create_field(horizontal_shape, 80.0).unwrap(),
        south_north_mass: backend.create_field(horizontal_shape, 85.0).unwrap(),
        mass_point_mass: backend.create_field(horizontal_shape, 90.0).unwrap(),
        west_east_map_factor: backend.create_field(horizontal_shape, 1.0).unwrap(),
        inverse_south_north_map_factor: backend.create_field(horizontal_shape, 0.9).unwrap(),
        mass_point_map_factor: backend.create_field(horizontal_shape, 0.95).unwrap(),
        half_level_mass_multiplier: vec![0.7; shape.bottom_top_points()],
        half_level_offset: vec![1.5; shape.bottom_top_points()],
        full_level_mass_multiplier: vec![0.6; shape.bottom_top_points()],
        full_level_offset: vec![2.0; shape.bottom_top_points()],
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
    }
}

fn apply_fixture(backend: &CpuBackend, fixture: &mut MomentumCouplingAllocationFixture) {
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

fn print_stats(phase: &str, worker_count: usize, stats: Stats) {
    println!(
        "momentum_coupling,{phase},{worker_count},{MEASURED_DISPATCHES},{},{},{}",
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
