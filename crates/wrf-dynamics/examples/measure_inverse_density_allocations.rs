//! Measures warmed steady-state allocations in WRF full inverse density.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{InverseDensityKernels, InverseDensityRegion};

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

struct InverseDensityAllocationFixture {
    full_inverse_density: CpuField<f32>,
    perturbation_inverse_density: CpuField<f32>,
    base_state_inverse_density: CpuField<f32>,
    region: InverseDensityRegion,
}

fn create_fixture(backend: &CpuBackend) -> InverseDensityAllocationFixture {
    let shape = GridShape::try_new(
        ACTIVE_WEST_EAST_POINTS + 2,
        ACTIVE_SOUTH_NORTH_POINTS + 2,
        ACTIVE_BOTTOM_TOP_POINTS + 2,
    )
    .unwrap();
    InverseDensityAllocationFixture {
        full_inverse_density: backend.create_field(shape, -999.0).unwrap(),
        perturbation_inverse_density: backend.create_field(shape, 0.125).unwrap(),
        base_state_inverse_density: backend.create_field(shape, 0.875).unwrap(),
        region: InverseDensityRegion::try_new(
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
    fixture: &mut InverseDensityAllocationFixture,
    dispatch_count: usize,
) {
    for _ in 0..dispatch_count {
        backend
            .calculate_full_inverse_density(
                &mut fixture.full_inverse_density,
                &fixture.perturbation_inverse_density,
                &fixture.base_state_inverse_density,
                &fixture.region,
            )
            .unwrap();
    }
}

fn print_stats(phase: &str, worker_count: usize, stats: Stats) {
    println!(
        "inverse_density,{phase},{worker_count},{MEASURED_DISPATCHES},{},{},{}",
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
