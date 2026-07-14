//! Criterion benchmark for WRF acoustic horizontal-momentum advancement.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    AcousticHorizontalBoundaryPolicy, AcousticHorizontalMapFactors, AcousticHorizontalMassFields,
    AcousticHorizontalMoistureCoefficients, AcousticHorizontalMomentumInputs,
    AcousticHorizontalMomentumKernels, AcousticHorizontalMomentumParameters,
    AcousticHorizontalMomentumRegion, AcousticHorizontalMomentumState,
    AcousticHorizontalMomentumTendencies, AcousticHorizontalPressureFields,
    AcousticHorizontalVerticalCoefficients, AcousticPressureMode, AcousticRelaxationZone,
    AcousticSouthNorthBoundary, AcousticWestEastBoundary, AcousticWestEastPeriodicity,
    VerticalAcousticTopBoundary,
};

const HORIZONTAL_MASS_POINTS: usize = 256;
const HALF_LEVELS: usize = 40;

fn benchmark(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("acoustic_horizontal_momentum");
    group.throughput(Throughput::Elements(
        (2 * HORIZONTAL_MASS_POINTS * HORIZONTAL_MASS_POINTS * HALF_LEVELS) as u64,
    ));
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = Fixture::new(&backend);
        group.bench_with_input(
            BenchmarkId::new("nonhydrostatic", workers),
            &workers,
            |bencher, _| {
                bencher.iter(|| fixture.apply(black_box(&backend)));
            },
        );
    }
    group.finish();
}

fn worker_counts() -> Vec<usize> {
    let available = thread::available_parallelism().map_or(1, |count| count.get());
    let mut counts = vec![1, available.min(4), available];
    counts.sort_unstable();
    counts.dedup();
    counts
}

struct Fixture {
    outputs: [CpuField<f32>; 2],
    volume: [CpuField<f32>; 10],
    horizontal: [CpuField<f32>; 9],
    vertical: [Vec<f32>; 5],
    region: AcousticHorizontalMomentumRegion,
}

impl Fixture {
    fn new(backend: &CpuBackend) -> Self {
        let extent = HORIZONTAL_MASS_POINTS + 2;
        let shape = GridShape::try_new(extent, extent, HALF_LEVELS + 2).unwrap();
        let volume = |value| backend.create_field(shape, value).unwrap();
        let horizontal = |value| {
            backend
                .create_field(shape.horizontal_shape(), value)
                .unwrap()
        };
        Self {
            outputs: [volume(1.0), volume(1.0)],
            volume: [
                volume(0.01),
                volume(0.02),
                volume(3.0),
                volume(4.0),
                volume(5.0),
                volume(2.0),
                volume(0.8),
                volume(0.1),
                volume(0.95),
                volume(0.96),
            ],
            horizontal: [
                horizontal(1.0),
                horizontal(2.0),
                horizontal(2.0),
                horizontal(0.5),
                horizontal(1.0),
                horizontal(1.0),
                horizontal(1.0),
                horizontal(1.0),
                horizontal(1.0),
            ],
            vertical: [
                vec![0.5; HALF_LEVELS + 2],
                vec![0.25; HALF_LEVELS + 2],
                vec![0.6; HALF_LEVELS + 2],
                vec![0.4; HALF_LEVELS + 2],
                vec![1.2; HALF_LEVELS + 2],
            ],
            region: AcousticHorizontalMomentumRegion::try_new(
                shape,
                1..HORIZONTAL_MASS_POINTS + 1,
                1..HORIZONTAL_MASS_POINTS + 1,
                1..HALF_LEVELS + 1,
                1..HORIZONTAL_MASS_POINTS + 2,
                1..HORIZONTAL_MASS_POINTS + 2,
                1..HALF_LEVELS + 2,
            )
            .unwrap(),
        }
    }

    fn apply(&mut self, backend: &CpuBackend) {
        let [west_east, south_north] = &mut self.outputs;
        backend
            .advance_acoustic_horizontal_momentum(
                AcousticHorizontalMomentumState::new(west_east, south_north),
                AcousticHorizontalMomentumInputs::new(
                    AcousticHorizontalMomentumTendencies::new(&self.volume[0], &self.volume[1]),
                    AcousticHorizontalPressureFields::new(
                        &self.volume[2],
                        &self.volume[3],
                        &self.volume[4],
                        &self.volume[5],
                        &self.volume[6],
                        &self.volume[7],
                    ),
                    AcousticHorizontalMassFields::new(
                        &self.horizontal[0],
                        &self.horizontal[1],
                        &self.horizontal[2],
                        &self.horizontal[3],
                    ),
                    AcousticHorizontalMoistureCoefficients::new(&self.volume[8], &self.volume[9]),
                    AcousticHorizontalMapFactors::new(
                        &self.horizontal[4],
                        &self.horizontal[5],
                        &self.horizontal[6],
                        &self.horizontal[7],
                        &self.horizontal[8],
                    ),
                    AcousticHorizontalVerticalCoefficients::new(
                        &self.vertical[0],
                        &self.vertical[1],
                        &self.vertical[2],
                        &self.vertical[3],
                        &self.vertical[4],
                    ),
                ),
                AcousticHorizontalMomentumParameters::new(
                    0.001,
                    0.001,
                    0.5,
                    [0.7, 0.2, 0.1],
                    0.1,
                    AcousticPressureMode::Nonhydrostatic,
                    VerticalAcousticTopBoundary::Nonrigid,
                ),
                AcousticHorizontalBoundaryPolicy::new(
                    AcousticRelaxationZone::Disabled,
                    AcousticWestEastPeriodicity::Nonperiodic,
                    AcousticWestEastBoundary::Closed,
                    AcousticWestEastBoundary::Closed,
                    AcousticSouthNorthBoundary::Closed,
                    AcousticSouthNorthBoundary::Closed,
                ),
                &self.region,
            )
            .unwrap();
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
