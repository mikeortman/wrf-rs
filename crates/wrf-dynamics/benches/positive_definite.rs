//! Criterion throughput benchmarks for the positive-definite kernel family.

// Criterion 0.7 generates a public harness function without a doc comment.
// Rust 1.85 prevents adopting Criterion 0.8, which fixes that warning.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{PositiveDefiniteKernels, PositiveDefiniteSlabRegion};

const WEST_EAST_POINTS: usize = 256;
const LINE_COUNT: usize = 4_096;

fn benchmark_positive_definite(c: &mut Criterion) {
    let point_count = WEST_EAST_POINTS * LINE_COUNT;
    let mut group = c.benchmark_group("positive_definite");
    group.throughput(Throughput::Elements(point_count as u64));

    for worker_count in benchmark_worker_counts() {
        let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
        let (sheet_template, line_totals) = create_sheet_template(&backend);
        group.bench_with_input(
            BenchmarkId::new("sheet", worker_count),
            &worker_count,
            |bencher, _| {
                bencher.iter_batched_ref(
                    || sheet_template.clone(),
                    |field| {
                        backend
                            .apply_positive_definite_sheet(
                                black_box(field),
                                black_box(&line_totals),
                            )
                            .unwrap();
                    },
                    BatchSize::LargeInput,
                );
            },
        );

        let (slab_template, slab_region) = create_slab_template(&backend);
        group.bench_with_input(
            BenchmarkId::new("slab", worker_count),
            &worker_count,
            |bencher, _| {
                bencher.iter_batched_ref(
                    || slab_template.clone(),
                    |field| {
                        backend
                            .apply_positive_definite_slab(black_box(field), black_box(&slab_region))
                            .unwrap();
                    },
                    BatchSize::LargeInput,
                );
            },
        );
    }

    group.finish();
}

fn benchmark_worker_counts() -> Vec<usize> {
    let available_workers = thread::available_parallelism().map_or(1, |count| count.get());
    let mut worker_counts = vec![1, available_workers.min(4), available_workers];
    worker_counts.sort_unstable();
    worker_counts.dedup();
    worker_counts
}

fn create_sheet_template(backend: &CpuBackend) -> (CpuField<f32>, Vec<f32>) {
    let shape = GridShape::try_new(WEST_EAST_POINTS, LINE_COUNT, 1).unwrap();
    let mut field = backend.create_field(shape, 0.0_f32).unwrap();
    initialize_lines(field.values_mut(), WEST_EAST_POINTS);
    let line_totals = field
        .values()
        .chunks_exact(WEST_EAST_POINTS)
        .map(|line| line.iter().copied().fold(0.0_f32, |sum, value| sum + value))
        .collect();
    (field, line_totals)
}

fn create_slab_template(backend: &CpuBackend) -> (CpuField<f32>, PositiveDefiniteSlabRegion) {
    let bottom_top_points = 64;
    let south_north_points = LINE_COUNT / bottom_top_points;
    let shape =
        GridShape::try_new(WEST_EAST_POINTS, south_north_points, bottom_top_points).unwrap();
    let mut field = backend.create_field(shape, 0.0_f32).unwrap();
    initialize_lines(field.values_mut(), WEST_EAST_POINTS);
    let region = PositiveDefiniteSlabRegion::try_new(
        shape,
        0..WEST_EAST_POINTS,
        0..bottom_top_points,
        0..south_north_points,
    )
    .unwrap();
    (field, region)
}

fn initialize_lines(values: &mut [f32], line_length: usize) {
    for (line_index, line) in values.chunks_exact_mut(line_length).enumerate() {
        let offset = line_index as f32 * 1.0e-6;
        for (point_index, value) in line.iter_mut().enumerate() {
            *value = 0.01 + point_index as f32 * 0.001 + offset;
        }
        line[line_index % line_length] = -0.001 - offset;
    }
}

criterion_group!(benches, benchmark_positive_definite);
criterion_main!(benches);
