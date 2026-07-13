//! Measures warmed steady-state allocations in the positive-definite kernels.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{PositiveDefiniteKernels, PositiveDefiniteSlabRegion};

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const WEST_EAST_POINTS: usize = 256;
const LINE_COUNT: usize = 4_096;
const WARMUP_DISPATCHES: usize = 100;
const MEASURED_DISPATCHES: usize = 100;

fn main() {
    println!("kernel,phase,workers,dispatches,allocations,reallocations,bytes_allocated");
    for worker_count in worker_counts() {
        let (sheet_first, sheet_settled, slab_first, slab_settled) =
            measure_worker_count(worker_count);
        print_stats("sheet", "first", worker_count, sheet_first);
        print_stats("sheet", "settled", worker_count, sheet_settled);
        print_stats("slab", "first", worker_count, slab_first);
        print_stats("slab", "settled", worker_count, slab_settled);
        for stats in [sheet_first, sheet_settled, slab_first, slab_settled] {
            assert_allocation_budget(stats);
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

fn measure_worker_count(worker_count: usize) -> (Stats, Stats, Stats, Stats) {
    let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
    let (mut sheet, sheet_template, line_totals) = create_sheet(&backend);
    let (mut slab, slab_template, slab_region) = create_slab(&backend);

    run_sheet_dispatches(
        &backend,
        &mut sheet,
        &sheet_template,
        &line_totals,
        WARMUP_DISPATCHES,
    );
    run_slab_dispatches(
        &backend,
        &mut slab,
        &slab_template,
        &slab_region,
        WARMUP_DISPATCHES,
    );

    let sheet_allocations = Region::new(GLOBAL_ALLOCATOR);
    run_sheet_dispatches(
        &backend,
        &mut sheet,
        &sheet_template,
        &line_totals,
        MEASURED_DISPATCHES,
    );
    let sheet_first = sheet_allocations.change();

    let sheet_settled_allocations = Region::new(GLOBAL_ALLOCATOR);
    run_sheet_dispatches(
        &backend,
        &mut sheet,
        &sheet_template,
        &line_totals,
        MEASURED_DISPATCHES,
    );
    let sheet_settled = sheet_settled_allocations.change();

    let slab_allocations = Region::new(GLOBAL_ALLOCATOR);
    run_slab_dispatches(
        &backend,
        &mut slab,
        &slab_template,
        &slab_region,
        MEASURED_DISPATCHES,
    );
    let slab_first = slab_allocations.change();

    let slab_settled_allocations = Region::new(GLOBAL_ALLOCATOR);
    run_slab_dispatches(
        &backend,
        &mut slab,
        &slab_template,
        &slab_region,
        MEASURED_DISPATCHES,
    );
    let slab_settled = slab_settled_allocations.change();

    (sheet_first, sheet_settled, slab_first, slab_settled)
}

fn run_sheet_dispatches(
    backend: &CpuBackend,
    field: &mut CpuField<f32>,
    template: &[f32],
    line_totals: &[f32],
    dispatch_count: usize,
) {
    for _ in 0..dispatch_count {
        field.values_mut().copy_from_slice(template);
        backend
            .apply_positive_definite_sheet(field, line_totals)
            .unwrap();
    }
}

fn run_slab_dispatches(
    backend: &CpuBackend,
    field: &mut CpuField<f32>,
    template: &[f32],
    region: &PositiveDefiniteSlabRegion,
    dispatch_count: usize,
) {
    for _ in 0..dispatch_count {
        field.values_mut().copy_from_slice(template);
        backend.apply_positive_definite_slab(field, region).unwrap();
    }
}

fn create_sheet(backend: &CpuBackend) -> (CpuField<f32>, Vec<f32>, Vec<f32>) {
    let shape = GridShape::try_new(WEST_EAST_POINTS, LINE_COUNT, 1).unwrap();
    let mut field = backend.create_field(shape, 0.0_f32).unwrap();
    initialize_lines(field.values_mut());
    let template = field.values().to_vec();
    let line_totals = template
        .chunks_exact(WEST_EAST_POINTS)
        .map(|line| line.iter().copied().fold(0.0_f32, |sum, value| sum + value))
        .collect();
    (field, template, line_totals)
}

fn create_slab(backend: &CpuBackend) -> (CpuField<f32>, Vec<f32>, PositiveDefiniteSlabRegion) {
    let bottom_top_points = 64;
    let south_north_points = LINE_COUNT / bottom_top_points;
    let shape =
        GridShape::try_new(WEST_EAST_POINTS, south_north_points, bottom_top_points).unwrap();
    let mut field = backend.create_field(shape, 0.0_f32).unwrap();
    initialize_lines(field.values_mut());
    let template = field.values().to_vec();
    let region = PositiveDefiniteSlabRegion::try_new(
        shape,
        0..WEST_EAST_POINTS,
        0..bottom_top_points,
        0..south_north_points,
    )
    .unwrap();
    (field, template, region)
}

fn initialize_lines(values: &mut [f32]) {
    for (line_index, line) in values.chunks_exact_mut(WEST_EAST_POINTS).enumerate() {
        let offset = line_index as f32 * 1.0e-6;
        for (point_index, value) in line.iter_mut().enumerate() {
            *value = 0.01 + point_index as f32 * 0.001 + offset;
        }
        line[line_index % WEST_EAST_POINTS] = -0.001 - offset;
    }
}

fn print_stats(kernel: &str, phase: &str, worker_count: usize, stats: Stats) {
    println!(
        "{kernel},{phase},{worker_count},{MEASURED_DISPATCHES},{},{},{}",
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
        stats.bytes_allocated <= 64 * 1_024,
        "kernel dispatch allocated more than 64 KiB per measured batch"
    );
}
