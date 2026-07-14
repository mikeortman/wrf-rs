//! Measures warmed allocations in specified-boundary tendency updates.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
use wrf_dynamics::{
    SpecifiedBoundaryFieldLocation, SpecifiedBoundaryUpdateKernels,
    SpecifiedBoundaryUpdateParameters, SpecifiedBoundaryUpdateRegion,
    SpecifiedBoundaryWestEastPeriodicity,
};

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const CALLS: usize = 100;

fn main() {
    println!("kernel,phase,workers,calls,allocations,reallocations,bytes_allocated");
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let shape = GridShape::try_new(258, 258, 42).unwrap();
        let mut field = backend.create_field(shape, 1.2_f32).unwrap();
        let tendency = backend.create_field(shape, 0.1_f32).unwrap();
        let region = SpecifiedBoundaryUpdateRegion::try_new(
            shape,
            SpecifiedBoundaryFieldLocation::MassHalfLevel,
            1..257,
            1..257,
            1..41,
            1..258,
            1..258,
            1..42,
        )
        .unwrap();
        let mut run = || {
            for _ in 0..CALLS {
                backend
                    .update_specified_boundary(
                        &mut field,
                        &tendency,
                        SpecifiedBoundaryUpdateParameters::new(0.25, 5),
                        SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                        &region,
                    )
                    .unwrap();
            }
        };
        run();
        let allocation_region = Region::new(GLOBAL_ALLOCATOR);
        run();
        let stats = allocation_region.change();
        print_stats(workers, stats);
        assert_eq!(stats.reallocations, 0);
        assert!(stats.allocations <= 20);
        assert!(stats.bytes_allocated <= 32 * 1_024);
    }
}

fn worker_counts() -> Vec<usize> {
    let available = std::thread::available_parallelism().map_or(1, |count| count.get());
    let mut counts = vec![1, available.min(4), available];
    counts.sort_unstable();
    counts.dedup();
    counts
}

fn print_stats(workers: usize, stats: Stats) {
    println!(
        "specified_boundary_update,settled,{workers},{CALLS},{},{},{}",
        stats.allocations, stats.reallocations, stats.bytes_allocated
    );
}
