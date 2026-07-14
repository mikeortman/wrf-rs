//! Criterion benchmark for WRF acoustic flux accumulation.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    AcousticFluxAccumulationKernels, AcousticFluxAccumulationRegion, AcousticFluxCurrentFields,
    AcousticFluxLinearFields, AcousticFluxMapFactors, AcousticFluxMassFields,
    AcousticFluxRunningAverages, AcousticSubstepPhase,
};

const HORIZONTAL_MASS_POINTS: usize = 256;
const HALF_LEVELS: usize = 40;

fn benchmark(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("acoustic_flux_accumulation");
    group.throughput(Throughput::Elements(
        (3 * HORIZONTAL_MASS_POINTS * HORIZONTAL_MASS_POINTS * HALF_LEVELS) as u64,
    ));
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = Fixture::new(&backend);
        group.bench_with_input(
            BenchmarkId::new("three_substeps", workers),
            &workers,
            |bencher, _| bencher.iter(|| fixture.apply(black_box(&backend))),
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
    current: [CpuField<f32>; 3],
    linear: [CpuField<f32>; 3],
    averages: [CpuField<f32>; 3],
    horizontal: [CpuField<f32>; 4],
    coefficients: [Vec<f32>; 2],
    region: AcousticFluxAccumulationRegion,
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
            current: [volume(0.2), volume(0.3), volume(0.4)],
            linear: [volume(0.15), volume(0.25), volume(0.35)],
            averages: [volume(-900.0), volume(-901.0), volume(-902.0)],
            horizontal: [
                horizontal(11.0),
                horizontal(12.0),
                horizontal(1.03),
                horizontal(0.97),
            ],
            coefficients: [vec![0.45; HALF_LEVELS + 2], vec![0.2; HALF_LEVELS + 2]],
            region: AcousticFluxAccumulationRegion::try_new(
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
        for iteration in 1..=3 {
            let [west_east, south_north, vertical] = &mut self.averages;
            backend
                .accumulate_acoustic_fluxes(
                    AcousticFluxRunningAverages {
                        west_east,
                        south_north,
                        vertical,
                    },
                    AcousticFluxCurrentFields {
                        west_east: &self.current[0],
                        south_north: &self.current[1],
                        vertical: &self.current[2],
                    },
                    AcousticFluxLinearFields {
                        west_east: &self.linear[0],
                        south_north: &self.linear[1],
                        vertical: &self.linear[2],
                    },
                    AcousticFluxMassFields {
                        west_east: &self.horizontal[0],
                        south_north: &self.horizontal[1],
                    },
                    AcousticFluxMapFactors {
                        west_east_y: &self.horizontal[2],
                        inverse_south_north_x: &self.horizontal[3],
                    },
                    &self.coefficients[0],
                    &self.coefficients[1],
                    AcousticSubstepPhase::try_new(iteration, 3).unwrap(),
                    &self.region,
                )
                .unwrap();
        }
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
