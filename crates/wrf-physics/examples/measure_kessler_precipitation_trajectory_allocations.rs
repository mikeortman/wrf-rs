//! Measures warmed allocations in the coupled Kessler precipitation trajectory.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::CpuBackend;

#[path = "../benches/support/kessler_precipitation_trajectory_fixture.rs"]
mod fixture;

use fixture::KesslerPrecipitationTrajectoryFixture;

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const DISPATCHES: usize = 10;

fn main() {
    println!("trajectory,phase,workers,dispatches,allocations,reallocations,bytes_allocated");
    for worker_count in [1, 4, 16] {
        let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
        let mut fixture = KesslerPrecipitationTrajectoryFixture::new(&backend);
        run_dispatches(&backend, &mut fixture);
        let reset_region = Region::new(GLOBAL_ALLOCATOR);
        for _ in 0..DISPATCHES {
            fixture.reset();
        }
        let reset = reset_region.change();
        let first_region = Region::new(GLOBAL_ALLOCATOR);
        run_dispatches(&backend, &mut fixture);
        let first = first_region.change();
        let settled_region = Region::new(GLOBAL_ALLOCATOR);
        run_dispatches(&backend, &mut fixture);
        let settled = settled_region.change();
        print_stats("reset", worker_count, reset);
        print_stats("first", worker_count, first);
        print_stats("settled", worker_count, settled);
        assert_eq!(reset, Stats::default());
        assert_budget(first);
        assert_budget(settled);
    }
}

fn run_dispatches(backend: &CpuBackend, fixture: &mut KesslerPrecipitationTrajectoryFixture) {
    for _ in 0..DISPATCHES {
        fixture.reset();
        fixture.apply_three_steps(backend);
    }
}

fn print_stats(phase: &str, worker_count: usize, stats: Stats) {
    println!(
        "kessler_precipitation_trajectory,{phase},{worker_count},{DISPATCHES},{},{},{}",
        stats.allocations, stats.reallocations, stats.bytes_allocated
    );
}

fn assert_budget(stats: Stats) {
    assert_eq!(stats.reallocations, 0);
    assert!(stats.allocations <= DISPATCHES * 10);
    assert!(stats.bytes_allocated <= 256 * 1_024);
}
