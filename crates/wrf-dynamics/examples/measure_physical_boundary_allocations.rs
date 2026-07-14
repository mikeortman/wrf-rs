//! Measures warmed allocations in WRF physical boundary-zone assignment.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
use wrf_dynamics::{
    PhysicalBoundaryConditions, PhysicalBoundaryKernels, PhysicalBoundaryRegion,
    PhysicalBoundaryVariable,
};

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const CALLS: usize = 100;

fn main() {
    println!("kernel,phase,workers,calls,allocations,reallocations,bytes_allocated");
    for workers in [1, 4, 16] {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let shape = GridShape::try_new(73, 73, 22).unwrap();
        let mut field = backend.create_field(shape, 1.2_f32).unwrap();
        let region =
            PhysicalBoundaryRegion::try_new(shape, 4..68, 4..68, 1..21, 4..69, 4..69, 1..22)
                .unwrap();
        let mut run = || {
            for _ in 0..CALLS {
                backend
                    .apply_physical_boundary(
                        &mut field,
                        PhysicalBoundaryVariable::MassHalfLevel,
                        PhysicalBoundaryConditions::specified_lateral(),
                        &region,
                    )
                    .unwrap();
            }
        };
        run();
        let first_region = Region::new(GLOBAL_ALLOCATOR);
        run();
        let first = first_region.change();
        let settled_region = Region::new(GLOBAL_ALLOCATOR);
        run();
        let settled = settled_region.change();
        print_stats("first", workers, first);
        print_stats("settled", workers, settled);
        assert_budget(first);
        assert_budget(settled);
    }
}

fn print_stats(phase: &str, workers: usize, stats: Stats) {
    println!(
        "physical_boundary,{phase},{workers},{CALLS},{},{},{}",
        stats.allocations, stats.reallocations, stats.bytes_allocated
    );
}

fn assert_budget(stats: Stats) {
    assert_eq!(stats.reallocations, 0);
    assert!(stats.allocations <= 16);
    assert!(stats.bytes_allocated <= 8 * 1_024);
}
