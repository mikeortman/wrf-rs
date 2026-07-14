//! Measures warmed steady-state allocations in column-mass staggering.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, FieldStorage, GridShape};
use wrf_dynamics::{
    ColumnMassStaggeringKernels, ColumnMassStaggeringPeriodicity, ColumnMassStaggeringRegion,
};

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const ACTIVE_WEST_EAST_MASS_POINTS: usize = 1_024;
const ACTIVE_SOUTH_NORTH_MASS_POINTS: usize = 1_024;
const WARMUP_DISPATCHES: usize = 100;
const MEASURED_DISPATCHES: usize = 100;

fn main() {
    println!("kernel,phase,workers,dispatches,allocations,reallocations,bytes_allocated");
    for kernel in [
        ColumnMassKernel::Staggered,
        ColumnMassKernel::PeriodicBigStep,
    ] {
        for worker_count in worker_counts() {
            let (first, settled) = measure_worker_count(kernel, worker_count);
            print_stats(kernel, "first", worker_count, first);
            print_stats(kernel, "settled", worker_count, settled);
            assert_allocation_budget(first);
            assert_allocation_budget(settled);
        }
    }
}

#[derive(Clone, Copy)]
enum ColumnMassKernel {
    Staggered,
    PeriodicBigStep,
}

impl ColumnMassKernel {
    const fn name(self) -> &'static str {
        match self {
            Self::Staggered => "column_mass_staggering",
            Self::PeriodicBigStep => "periodic_big_step_column_mass",
        }
    }
}

fn worker_counts() -> Vec<usize> {
    let available_workers = std::thread::available_parallelism().map_or(1, |count| count.get());
    let mut counts = vec![1, available_workers.min(4), available_workers];
    counts.sort_unstable();
    counts.dedup();
    counts
}

fn measure_worker_count(kernel: ColumnMassKernel, worker_count: usize) -> (Stats, Stats) {
    let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
    let (mut fields, region) = create_fields(&backend);

    run_dispatches(kernel, &backend, &mut fields, &region, WARMUP_DISPATCHES);
    let first_allocations = Region::new(GLOBAL_ALLOCATOR);
    run_dispatches(kernel, &backend, &mut fields, &region, MEASURED_DISPATCHES);
    let first = first_allocations.change();
    let settled_allocations = Region::new(GLOBAL_ALLOCATOR);
    run_dispatches(kernel, &backend, &mut fields, &region, MEASURED_DISPATCHES);
    let settled = settled_allocations.change();
    (first, settled)
}

fn run_dispatches(
    kernel: ColumnMassKernel,
    backend: &CpuBackend,
    fields: &mut ColumnMassStaggeringAllocationFields,
    region: &ColumnMassStaggeringRegion,
    dispatch_count: usize,
) {
    for _ in 0..dispatch_count {
        match kernel {
            ColumnMassKernel::Staggered => backend.stagger_column_mass(
                &fields.perturbation_mass,
                &fields.base_mass,
                &mut fields.west_east_momentum_mass,
                &mut fields.south_north_momentum_mass,
                region,
            ),
            ColumnMassKernel::PeriodicBigStep => backend.stagger_column_mass_for_big_step(
                &fields.perturbation_mass,
                &fields.base_mass,
                &mut fields.west_east_momentum_mass,
                &mut fields.south_north_momentum_mass,
                region,
                ColumnMassStaggeringPeriodicity::Both,
            ),
        }
        .unwrap();
    }
}

struct ColumnMassStaggeringAllocationFields {
    perturbation_mass: CpuField<f32>,
    base_mass: CpuField<f32>,
    west_east_momentum_mass: CpuField<f32>,
    south_north_momentum_mass: CpuField<f32>,
}

fn create_fields(
    backend: &CpuBackend,
) -> (
    ColumnMassStaggeringAllocationFields,
    ColumnMassStaggeringRegion,
) {
    let west_east_points = ACTIVE_WEST_EAST_MASS_POINTS + 2;
    let south_north_points = ACTIVE_SOUTH_NORTH_MASS_POINTS + 2;
    let shape = GridShape::try_new(west_east_points, south_north_points, 1).unwrap();
    let mut perturbation_mass = backend.create_field(shape, 0.0_f32).unwrap();
    let mut base_mass = backend.create_field(shape, 0.0_f32).unwrap();
    initialize_mass_fields(&mut perturbation_mass, &mut base_mass);
    let west_east_momentum_mass = backend.create_field(shape, -999.0_f32).unwrap();
    let south_north_momentum_mass = backend.create_field(shape, -999.0_f32).unwrap();
    let region = ColumnMassStaggeringRegion::try_new(
        shape,
        1..(ACTIVE_WEST_EAST_MASS_POINTS + 1),
        1..(ACTIVE_SOUTH_NORTH_MASS_POINTS + 1),
        1..(ACTIVE_WEST_EAST_MASS_POINTS + 2),
        1..(ACTIVE_SOUTH_NORTH_MASS_POINTS + 2),
    )
    .unwrap();
    (
        ColumnMassStaggeringAllocationFields {
            perturbation_mass,
            base_mass,
            west_east_momentum_mass,
            south_north_momentum_mass,
        },
        region,
    )
}

fn initialize_mass_fields(perturbation_mass: &mut CpuField<f32>, base_mass: &mut CpuField<f32>) {
    let shape = perturbation_mass.shape();
    for south_north_index in 0..shape.south_north_points() {
        for west_east_index in 0..shape.west_east_points() {
            let index = south_north_index * shape.west_east_points() + west_east_index;
            perturbation_mass.values_mut()[index] =
                west_east_index as f32 * 0.25 + south_north_index as f32 * 1.5 - 0.3;
            base_mass.values_mut()[index] =
                100.0 + west_east_index as f32 * 0.5 - south_north_index as f32 * 0.75;
        }
    }
}

fn print_stats(kernel: ColumnMassKernel, phase: &str, worker_count: usize, stats: Stats) {
    println!(
        "{},{phase},{worker_count},{MEASURED_DISPATCHES},{},{},{}",
        kernel.name(),
        stats.allocations,
        stats.reallocations,
        stats.bytes_allocated
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
