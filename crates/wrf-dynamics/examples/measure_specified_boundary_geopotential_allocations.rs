//! Measures warmed allocations in geopotential boundary updates.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
use wrf_dynamics::{
    SpecifiedBoundaryFieldLocation, SpecifiedBoundaryGeopotentialInputs,
    SpecifiedBoundaryGeopotentialKernels, SpecifiedBoundaryUpdateParameters,
    SpecifiedBoundaryUpdateRegion, SpecifiedBoundaryWestEastPeriodicity,
};

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const CALLS: usize = 100;

fn main() {
    println!("kernel,phase,workers,calls,allocations,reallocations,bytes_allocated");
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let shape = GridShape::try_new(258, 258, 42).unwrap();
        let mut geopotential = backend.create_field(shape, -200.0_f32).unwrap();
        let saved = backend.create_field(shape, 100.0_f32).unwrap();
        let tendency = backend.create_field(shape, 1.5_f32).unwrap();
        let mass_tendency = backend
            .create_field(shape.horizontal_shape(), 0.3_f32)
            .unwrap();
        let current_mass = backend
            .create_field(shape.horizontal_shape(), 10.0_f32)
            .unwrap();
        let multiplier = vec![0.4_f32; 42];
        let offset = vec![2.0_f32; 42];
        let region = SpecifiedBoundaryUpdateRegion::try_new(
            shape,
            SpecifiedBoundaryFieldLocation::FullLevel,
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
                    .update_specified_boundary_geopotential(
                        &mut geopotential,
                        SpecifiedBoundaryGeopotentialInputs::new(
                            &saved,
                            &tendency,
                            &mass_tendency,
                            &current_mass,
                            &multiplier,
                            &offset,
                        ),
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
        "specified_boundary_geopotential,settled,{workers},{CALLS},{},{},{}",
        stats.allocations, stats.reallocations, stats.bytes_allocated
    );
}
