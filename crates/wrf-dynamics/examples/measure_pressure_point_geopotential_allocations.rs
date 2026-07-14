//! Measures warmed allocations in WRF pressure-point geopotential.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{PressurePointGeopotentialKernels, PressurePointGeopotentialRegion};

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const ACTIVE_WEST_EAST_POINTS: usize = 256;
const ACTIVE_SOUTH_NORTH_POINTS: usize = 256;
const ACTIVE_BOTTOM_TOP_POINTS: usize = 40;
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

struct PressurePointGeopotentialAllocationFixture {
    pressure_point_geopotential: CpuField<f32>,
    perturbation_full_level_geopotential: CpuField<f32>,
    base_state_full_level_geopotential: CpuField<f32>,
    region: PressurePointGeopotentialRegion,
}

fn create_fixture(backend: &CpuBackend) -> PressurePointGeopotentialAllocationFixture {
    let shape = GridShape::try_new(
        ACTIVE_WEST_EAST_POINTS + 2,
        ACTIVE_SOUTH_NORTH_POINTS + 2,
        ACTIVE_BOTTOM_TOP_POINTS + 2,
    )
    .unwrap();
    PressurePointGeopotentialAllocationFixture {
        pressure_point_geopotential: backend.create_field(shape, -999.0).unwrap(),
        perturbation_full_level_geopotential: backend.create_field(shape, 125.0).unwrap(),
        base_state_full_level_geopotential: backend.create_field(shape, 875.0).unwrap(),
        region: PressurePointGeopotentialRegion::try_new(
            shape,
            1..(ACTIVE_WEST_EAST_POINTS + 1),
            1..(ACTIVE_SOUTH_NORTH_POINTS + 1),
            1..(ACTIVE_BOTTOM_TOP_POINTS + 1),
            1..(ACTIVE_WEST_EAST_POINTS + 2),
            1..(ACTIVE_SOUTH_NORTH_POINTS + 2),
            1..(ACTIVE_BOTTOM_TOP_POINTS + 2),
        )
        .unwrap(),
    }
}

fn run_dispatches(
    backend: &CpuBackend,
    fixture: &mut PressurePointGeopotentialAllocationFixture,
    dispatch_count: usize,
) {
    for _ in 0..dispatch_count {
        backend
            .calculate_pressure_point_geopotential(
                &mut fixture.pressure_point_geopotential,
                &fixture.perturbation_full_level_geopotential,
                &fixture.base_state_full_level_geopotential,
                &fixture.region,
            )
            .unwrap();
    }
}

fn print_stats(phase: &str, worker_count: usize, stats: Stats) {
    println!(
        "pressure_point_geopotential,{phase},{worker_count},{MEASURED_DISPATCHES},{},{},{}",
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
