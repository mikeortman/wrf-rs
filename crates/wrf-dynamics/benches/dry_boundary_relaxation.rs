//! Criterion benchmark for WRF dry boundary-relaxation orchestration.
#![allow(missing_docs)]

use std::alloc::System;
use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    DryBoundaryRelaxationBoundaryData, DryBoundaryRelaxationBoundaryFields,
    DryBoundaryRelaxationKernels, DryBoundaryRelaxationMassCoefficients,
    DryBoundaryRelaxationRegion, DryBoundaryRelaxationState, DryBoundaryRelaxationTendencies,
    DryBoundaryRelaxationWorkspace, DryBoundaryVerticalRelaxation,
    SpecifiedBoundaryRelaxationBoundaryValues, SpecifiedBoundaryRelaxationCoefficients,
    SpecifiedBoundaryRelaxationParameters, SpecifiedBoundaryTendencies,
    SpecifiedBoundaryWestEastPeriodicity,
};

const HORIZONTAL_MASS_POINTS: usize = 256;
const HALF_LEVELS: usize = 40;
const BOUNDARY_WIDTH: usize = 8;
const RELAXATION_UPDATES: u64 = 1_209_216;
const MASS_WEIGHTED_POINTS: u64 = 7_995_392;

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn benchmark(criterion: &mut Criterion) {
    report_settled_allocations();
    let mut group = criterion.benchmark_group("dry_boundary_relaxation");
    group.throughput(Throughput::Elements(
        RELAXATION_UPDATES + MASS_WEIGHTED_POINTS,
    ));
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = Fixture::new(&backend);
        group.bench_with_input(
            BenchmarkId::new("nested_full_tile", workers),
            &workers,
            |bencher, _| bencher.iter(|| fixture.apply(black_box(&backend))),
        );
    }
    group.finish();
}

fn report_settled_allocations() {
    let workers = thread::available_parallelism().map_or(1, |count| count.get());
    let backend = CpuBackend::try_with_worker_count(workers).unwrap();
    let mut fixture = Fixture::new(&backend);
    fixture.apply(&backend);
    let allocation_region = Region::new(GLOBAL_ALLOCATOR);
    for _ in 0..100 {
        fixture.apply(&backend);
    }
    eprintln!(
        "settled allocations per 100 calls: {:?}",
        allocation_region.change()
    );
}

fn worker_counts() -> Vec<usize> {
    let available = thread::available_parallelism().map_or(1, |count| count.get());
    let mut counts = vec![1, available.min(4), available];
    counts.sort_unstable();
    counts.dedup();
    counts
}

struct BoundarySet {
    fields: [CpuField<f32>; 8],
}

impl BoundarySet {
    fn new(backend: &CpuBackend, shape: GridShape) -> Self {
        let values = [2.1, 2.2, 2.3, 2.4, 0.01, 0.02, 0.03, 0.04];
        Self {
            fields: values.map(|value| backend.create_field(shape, value).unwrap()),
        }
    }

    fn data(&self) -> DryBoundaryRelaxationBoundaryData<'_, CpuField<f32>> {
        DryBoundaryRelaxationBoundaryData::new(
            SpecifiedBoundaryRelaxationBoundaryValues::new(
                &self.fields[0],
                &self.fields[1],
                &self.fields[2],
                &self.fields[3],
            ),
            SpecifiedBoundaryTendencies::new(
                &self.fields[4],
                &self.fields[5],
                &self.fields[6],
                &self.fields[7],
            ),
        )
    }
}

struct Fixture {
    state: [CpuField<f32>; 5],
    perturbation_column_mass: CpuField<f32>,
    full_column_mass: CpuField<f32>,
    tendencies: [CpuField<f32>; 5],
    column_mass_tendency: CpuField<f32>,
    volume_boundaries: BoundarySet,
    horizontal_boundaries: BoundarySet,
    workspace: CpuField<f32>,
    region: DryBoundaryRelaxationRegion,
    half_multiplier: Vec<f32>,
    half_offset: Vec<f32>,
    full_multiplier: Vec<f32>,
    full_offset: Vec<f32>,
}

impl Fixture {
    fn new(backend: &CpuBackend) -> Self {
        let shape = GridShape::try_new(
            HORIZONTAL_MASS_POINTS + 2,
            HORIZONTAL_MASS_POINTS + 2,
            HALF_LEVELS + 2,
        )
        .unwrap();
        let horizontal_shape = shape.horizontal_shape();
        let volume = |value| backend.create_field(shape, value).unwrap();
        let horizontal = |value| backend.create_field(horizontal_shape, value).unwrap();
        let region = DryBoundaryRelaxationRegion::try_new(
            shape,
            1..HORIZONTAL_MASS_POINTS + 1,
            1..HORIZONTAL_MASS_POINTS + 1,
            1..HALF_LEVELS + 1,
            1..HORIZONTAL_MASS_POINTS + 1,
            1..HORIZONTAL_MASS_POINTS + 1,
            1..HALF_LEVELS + 2,
        )
        .unwrap();
        let boundary_shape =
            GridShape::try_new(HORIZONTAL_MASS_POINTS + 2, BOUNDARY_WIDTH, HALF_LEVELS + 1)
                .unwrap();
        let horizontal_boundary_shape =
            GridShape::try_new(HORIZONTAL_MASS_POINTS + 2, BOUNDARY_WIDTH, 1).unwrap();
        Self {
            state: [
                volume(1.2),
                volume(1.3),
                volume(1.4),
                volume(1.5),
                volume(1.6),
            ],
            perturbation_column_mass: horizontal(1.7),
            full_column_mass: horizontal(10.0),
            tendencies: [
                volume(0.1),
                volume(0.2),
                volume(0.3),
                volume(0.4),
                volume(0.5),
            ],
            column_mass_tendency: horizontal(0.6),
            volume_boundaries: BoundarySet::new(backend, boundary_shape),
            horizontal_boundaries: BoundarySet::new(backend, horizontal_boundary_shape),
            workspace: backend.create_field(region.workspace_shape(), 0.0).unwrap(),
            region,
            half_multiplier: vec![0.60; HALF_LEVELS + 2],
            half_offset: vec![0.40; HALF_LEVELS + 2],
            full_multiplier: vec![0.55; HALF_LEVELS + 2],
            full_offset: vec![0.45; HALF_LEVELS + 2],
        }
    }

    fn apply(&mut self, backend: &CpuBackend) {
        let [
            west_east_tendency,
            south_north_tendency,
            geopotential_tendency,
            temperature_tendency,
            vertical_tendency,
        ] = &mut self.tendencies;
        let volume_boundary = self.volume_boundaries.data();
        backend
            .add_dry_boundary_relaxation_tendencies(
                DryBoundaryRelaxationTendencies::new(
                    west_east_tendency,
                    south_north_tendency,
                    geopotential_tendency,
                    temperature_tendency,
                    &mut self.column_mass_tendency,
                ),
                DryBoundaryRelaxationState::new(
                    &self.state[0],
                    &self.state[1],
                    &self.state[2],
                    &self.state[3],
                    &self.perturbation_column_mass,
                    &self.full_column_mass,
                ),
                DryBoundaryRelaxationBoundaryFields::new(
                    volume_boundary,
                    volume_boundary,
                    volume_boundary,
                    volume_boundary,
                    self.horizontal_boundaries.data(),
                ),
                DryBoundaryVerticalRelaxation::Nested {
                    velocity: &self.state[4],
                    tendency: vertical_tendency,
                    boundary: volume_boundary,
                },
                DryBoundaryRelaxationWorkspace::new(&mut self.workspace),
                DryBoundaryRelaxationMassCoefficients::new(
                    &self.half_multiplier,
                    &self.half_offset,
                    &self.full_multiplier,
                    &self.full_offset,
                ),
                SpecifiedBoundaryRelaxationCoefficients::new(
                    &[0.0, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.0],
                    &[0.0, 0.08, 0.07, 0.06, 0.05, 0.04, 0.03, 0.0],
                ),
                SpecifiedBoundaryRelaxationParameters::new(BOUNDARY_WIDTH, 1, 7, 0.25),
                SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                &self.region,
            )
            .unwrap();
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
