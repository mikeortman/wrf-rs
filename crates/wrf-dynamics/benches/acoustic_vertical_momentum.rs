//! Criterion benchmark for WRF implicit acoustic vertical advancement.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use std::alloc::System;
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    AcousticVerticalAdvection, AcousticVerticalBoundaryPolicy, AcousticVerticalDamping,
    AcousticVerticalGeopotentialInputs, AcousticVerticalInputs, AcousticVerticalKernels,
    AcousticVerticalLateralDomain, AcousticVerticalLevelCoefficients, AcousticVerticalMapFactors,
    AcousticVerticalMassInputs, AcousticVerticalMomentumInputs, AcousticVerticalParameters,
    AcousticVerticalRegion, AcousticVerticalSolveInputs, AcousticVerticalState,
    AcousticVerticalThermodynamicInputs, AcousticVerticalWestEastPeriodicity,
    AcousticVerticalWorkspace, VerticalAcousticTopBoundary,
};

const HORIZONTAL_MASS_POINTS: usize = 256;
const MASS_LEVELS: usize = 40;

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn benchmark(criterion: &mut Criterion) {
    report_settled_allocations();
    let mut group = criterion.benchmark_group("acoustic_vertical_momentum");
    group.throughput(Throughput::Elements(
        (HORIZONTAL_MASS_POINTS * HORIZONTAL_MASS_POINTS * MASS_LEVELS) as u64,
    ));
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = Fixture::new(&backend);
        group.bench_with_input(
            BenchmarkId::new("advance", workers),
            &workers,
            |bencher, _| {
                bencher.iter(|| fixture.apply(black_box(&backend)));
            },
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

struct Fixture {
    outputs: [CpuField<f32>; 3],
    workspace: CpuField<f32>,
    inputs: [CpuField<f32>; 16],
    horizontal: [CpuField<f32>; 6],
    vertical: [Vec<f32>; 8],
    region: AcousticVerticalRegion,
}

impl Fixture {
    fn new(backend: &CpuBackend) -> Self {
        let horizontal_extent = HORIZONTAL_MASS_POINTS + 2;
        let shape =
            GridShape::try_new(horizontal_extent, horizontal_extent, MASS_LEVELS + 2).unwrap();
        let volume = |value| backend.create_field(shape, value).unwrap();
        let horizontal = |value| {
            backend
                .create_field(shape.horizontal_shape(), value)
                .unwrap()
        };
        Self {
            outputs: [volume(0.8), volume(20.0), volume(294.0)],
            workspace: volume(0.0),
            inputs: [
                volume(0.012),
                volume(0.35),
                volume(0.31),
                volume(0.2),
                volume(0.3),
                volume(300.0),
                volume(1.3),
                volume(18.0),
                volume(30_000.0),
                volume(0.05),
                volume(1.1),
                volume(0.82),
                volume(0.9),
                volume(-0.03),
                volume(0.83),
                volume(-0.02),
            ],
            horizontal: [
                horizontal(11.0),
                horizontal(2.1),
                horizontal(12.7),
                horizontal(140.0),
                horizontal(1.03),
                horizontal(0.97),
            ],
            vertical: [
                vec![0.42; MASS_LEVELS + 2],
                vec![0.19; MASS_LEVELS + 2],
                vec![0.37; MASS_LEVELS + 2],
                vec![0.23; MASS_LEVELS + 2],
                vec![0.58; MASS_LEVELS + 2],
                vec![0.42; MASS_LEVELS + 2],
                vec![1.05; MASS_LEVELS + 2],
                vec![0.91; MASS_LEVELS + 2],
            ],
            region: AcousticVerticalRegion::try_new(
                shape,
                1..HORIZONTAL_MASS_POINTS + 1,
                1..HORIZONTAL_MASS_POINTS + 1,
                1..MASS_LEVELS + 1,
                1..HORIZONTAL_MASS_POINTS + 2,
                1..HORIZONTAL_MASS_POINTS + 2,
                1..MASS_LEVELS + 2,
            )
            .unwrap(),
        }
    }

    fn apply(&mut self, backend: &CpuBackend) {
        let [vertical_momentum, geopotential, thermodynamics] = &mut self.outputs;
        backend
            .advance_acoustic_vertical_momentum(
                AcousticVerticalState::new(vertical_momentum, geopotential, thermodynamics),
                AcousticVerticalWorkspace::new(&mut self.workspace),
                AcousticVerticalInputs::new(
                    AcousticVerticalMomentumInputs::new(
                        &self.inputs[0],
                        &self.inputs[1],
                        &self.inputs[2],
                        &self.inputs[3],
                        &self.inputs[4],
                    ),
                    AcousticVerticalMassInputs::new(
                        &self.horizontal[0],
                        &self.horizontal[1],
                        &self.horizontal[2],
                    ),
                    AcousticVerticalThermodynamicInputs::new(&self.inputs[5], &self.inputs[6]),
                    AcousticVerticalGeopotentialInputs::new(
                        &self.inputs[7],
                        &self.inputs[8],
                        &self.inputs[9],
                    ),
                    AcousticVerticalMapFactors::new(
                        &self.horizontal[3],
                        &self.horizontal[4],
                        &self.horizontal[5],
                    ),
                    AcousticVerticalSolveInputs::new(
                        &self.inputs[10],
                        &self.inputs[11],
                        &self.inputs[12],
                        &self.inputs[13],
                        &self.inputs[14],
                        &self.inputs[15],
                    ),
                    AcousticVerticalLevelCoefficients::new(
                        &self.vertical[0],
                        &self.vertical[1],
                        &self.vertical[2],
                        &self.vertical[3],
                        &self.vertical[4],
                        &self.vertical[5],
                        &self.vertical[6],
                        &self.vertical[7],
                    ),
                ),
                AcousticVerticalParameters::new(0.002, 0.003, 0.4, 300.0, 0.1, [0.5, 0.3, 0.2]),
                AcousticVerticalBoundaryPolicy::new(
                    AcousticVerticalLateralDomain::Global,
                    AcousticVerticalWestEastPeriodicity::Nonperiodic,
                ),
                VerticalAcousticTopBoundary::Nonrigid,
                AcousticVerticalAdvection::StaggeredGeopotentialGradient,
                AcousticVerticalDamping::UpperLayer {
                    coefficient: 0.15,
                    depth: 220.0,
                },
                &self.region,
            )
            .unwrap();
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
