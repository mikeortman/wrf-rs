//! Measures warmed allocations in the Registry-backed accepted ARW trajectory.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::CpuBackend;

#[path = "../benches/support/registry_backed_arw_trajectory_fixture.rs"]
mod fixture;

use fixture::RegistryBackedArwTrajectoryFixture;

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const DISPATCHES: usize = 5;
const COMMON_FIELD_BYTES: usize = 64 * 64 * 40 * size_of::<f32>();

fn main() {
    println!("trajectory,phase,workers,dispatches,allocations,reallocations,bytes_allocated");
    for worker_count in [1, 4] {
        let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
        let mut fixture = RegistryBackedArwTrajectoryFixture::new(&backend);
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

fn run_dispatches(backend: &CpuBackend, fixture: &mut RegistryBackedArwTrajectoryFixture) {
    for _ in 0..DISPATCHES {
        fixture.reset();
        fixture
            .model
            .advance_short_trajectory(backend, &mut fixture.state, &mut fixture.workspace)
            .unwrap();
    }
}

fn print_stats(phase: &str, worker_count: usize, stats: Stats) {
    println!(
        "registry_backed_arw_trajectory,{phase},{worker_count},{DISPATCHES},{},{},{}",
        stats.allocations, stats.reallocations, stats.bytes_allocated
    );
}

fn assert_budget(stats: Stats) {
    assert_eq!(stats.reallocations, 0);
    assert!(
        stats.bytes_allocated < DISPATCHES * COMMON_FIELD_BYTES,
        "trajectory allocated storage as large as one model field per dispatch"
    );
}
