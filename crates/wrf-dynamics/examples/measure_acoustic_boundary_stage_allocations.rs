//! Measures warmed allocations in the complete WRF acoustic boundary stage.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::CpuBackend;

#[path = "../benches/support/acoustic_boundary_stage_fixture.rs"]
mod fixture;

use fixture::AcousticBoundaryStageFixture;

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const ACTIVE_WEST_EAST_POINTS: usize = 34;
const ACTIVE_SOUTH_NORTH_POINTS: usize = 34;
const HALF_LEVELS: usize = 12;
const DISPATCHES: usize = 25;

fn main() {
    println!("kernel,phase,workers,dispatches,allocations,reallocations,bytes_allocated");
    for workers in [1, 4, 16] {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = AcousticBoundaryStageFixture::new(
            &backend,
            ACTIVE_WEST_EAST_POINTS,
            ACTIVE_SOUTH_NORTH_POINTS,
            HALF_LEVELS,
        );
        run_dispatches(&backend, &mut fixture);
        let first_region = Region::new(GLOBAL_ALLOCATOR);
        run_dispatches(&backend, &mut fixture);
        let first = first_region.change();
        let settled_region = Region::new(GLOBAL_ALLOCATOR);
        run_dispatches(&backend, &mut fixture);
        let settled = settled_region.change();
        print_stats("first", workers, first);
        print_stats("settled", workers, settled);
        assert_budget(first);
        assert_budget(settled);
    }
}

fn run_dispatches(backend: &CpuBackend, fixture: &mut AcousticBoundaryStageFixture) {
    for _ in 0..DISPATCHES {
        fixture.reset();
        fixture.apply(backend);
    }
}

fn print_stats(phase: &str, workers: usize, stats: Stats) {
    println!(
        "acoustic_boundary_stage,{phase},{workers},{DISPATCHES},{},{},{}",
        stats.allocations, stats.reallocations, stats.bytes_allocated
    );
}

fn assert_budget(stats: Stats) {
    assert_eq!(stats.reallocations, 0);
    assert!(stats.allocations <= DISPATCHES * 3);
    assert!(stats.bytes_allocated <= 128 * 1_024);
}
