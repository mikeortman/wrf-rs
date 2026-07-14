//! Measures warmed steady-state allocations in Held-Suarez damping.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{HeldSuarezDampingFields, HeldSuarezDampingKernels, HeldSuarezDampingRegion};

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const ACTIVE_WEST_EAST_POINTS: usize = 256;
const ACTIVE_BOTTOM_TOP_POINTS: usize = 64;
const ACTIVE_SOUTH_NORTH_POINTS: usize = 64;
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
    let (mut fields, region) = create_fields(&backend);
    let west_east_tendency_template = fields.west_east_momentum_tendency.values().to_vec();
    let south_north_tendency_template = fields.south_north_momentum_tendency.values().to_vec();

    run_dispatches(
        &backend,
        &mut fields,
        &region,
        &west_east_tendency_template,
        &south_north_tendency_template,
        WARMUP_DISPATCHES,
    );
    let first_allocations = Region::new(GLOBAL_ALLOCATOR);
    run_dispatches(
        &backend,
        &mut fields,
        &region,
        &west_east_tendency_template,
        &south_north_tendency_template,
        MEASURED_DISPATCHES,
    );
    let first = first_allocations.change();
    let settled_allocations = Region::new(GLOBAL_ALLOCATOR);
    run_dispatches(
        &backend,
        &mut fields,
        &region,
        &west_east_tendency_template,
        &south_north_tendency_template,
        MEASURED_DISPATCHES,
    );
    let settled = settled_allocations.change();
    (first, settled)
}

fn run_dispatches(
    backend: &CpuBackend,
    fields: &mut HeldSuarezAllocationFields,
    region: &HeldSuarezDampingRegion,
    west_east_tendency_template: &[f32],
    south_north_tendency_template: &[f32],
    dispatch_count: usize,
) {
    for _ in 0..dispatch_count {
        fields
            .west_east_momentum_tendency
            .values_mut()
            .copy_from_slice(west_east_tendency_template);
        fields
            .south_north_momentum_tendency
            .values_mut()
            .copy_from_slice(south_north_tendency_template);
        backend
            .apply_held_suarez_damping(
                HeldSuarezDampingFields::new(
                    &mut fields.west_east_momentum_tendency,
                    &mut fields.south_north_momentum_tendency,
                    &fields.west_east_momentum,
                    &fields.south_north_momentum,
                    &fields.perturbation_pressure,
                    &fields.base_pressure,
                ),
                region,
            )
            .unwrap();
    }
}

struct HeldSuarezAllocationFields {
    west_east_momentum_tendency: CpuField<f32>,
    south_north_momentum_tendency: CpuField<f32>,
    west_east_momentum: CpuField<f32>,
    south_north_momentum: CpuField<f32>,
    perturbation_pressure: CpuField<f32>,
    base_pressure: CpuField<f32>,
}

fn create_fields(backend: &CpuBackend) -> (HeldSuarezAllocationFields, HeldSuarezDampingRegion) {
    let west_east_points = ACTIVE_WEST_EAST_POINTS + 2;
    let bottom_top_points = ACTIVE_BOTTOM_TOP_POINTS + 1;
    let south_north_points = ACTIVE_SOUTH_NORTH_POINTS + 2;
    let shape =
        GridShape::try_new(west_east_points, south_north_points, bottom_top_points).unwrap();
    let mut fields = HeldSuarezAllocationFields {
        west_east_momentum_tendency: backend.create_field(shape, 0.001).unwrap(),
        south_north_momentum_tendency: backend.create_field(shape, -0.002).unwrap(),
        west_east_momentum: backend.create_field(shape, 10.0).unwrap(),
        south_north_momentum: backend.create_field(shape, -7.0).unwrap(),
        perturbation_pressure: backend.create_field(shape, 0.0).unwrap(),
        base_pressure: backend.create_field(shape, 0.0).unwrap(),
    };
    for south_north_index in 0..south_north_points {
        for bottom_top_index in 0..bottom_top_points {
            for west_east_index in 0..west_east_points {
                let index = (south_north_index * bottom_top_points + bottom_top_index)
                    * west_east_points
                    + west_east_index;
                fields.perturbation_pressure.values_mut()[index] =
                    west_east_index as f32 * 0.125 + south_north_index as f32 * 0.25;
                fields.base_pressure.values_mut()[index] =
                    100_500.0 - bottom_top_index as f32 * 500.0;
            }
        }
    }
    let region = HeldSuarezDampingRegion::try_new(
        shape,
        1..(ACTIVE_WEST_EAST_POINTS + 1),
        1..(ACTIVE_BOTTOM_TOP_POINTS + 1),
        1..(ACTIVE_SOUTH_NORTH_POINTS + 1),
        1..(ACTIVE_SOUTH_NORTH_POINTS + 1),
        1,
    )
    .unwrap();
    (fields, region)
}

fn print_stats(phase: &str, worker_count: usize, stats: Stats) {
    println!(
        "held_suarez,{phase},{worker_count},{MEASURED_DISPATCHES},{},{},{}",
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
        stats.bytes_allocated <= 128 * 1_024,
        "kernel dispatch allocated more than 128 KiB per measured batch"
    );
}
