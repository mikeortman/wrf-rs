//! Criterion benchmark for WRF acoustic mass, omega, and theta advancement.
#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    AcousticMassThetaBoundaryPolicy, AcousticMassThetaDiagnostics, AcousticMassThetaInputs,
    AcousticMassThetaKernels, AcousticMassThetaLateralDomain, AcousticMassThetaMapFactors,
    AcousticMassThetaMassInputs, AcousticMassThetaMomentumInputs, AcousticMassThetaParameters,
    AcousticMassThetaRegion, AcousticMassThetaState, AcousticMassThetaThermodynamicInputs,
    AcousticMassThetaVerticalCoefficients, AcousticMassThetaWestEastPeriodicity,
};

const HORIZONTAL_MASS_POINTS: usize = 256;
const HALF_LEVELS: usize = 40;

fn benchmark(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("acoustic_mass_theta");
    group.throughput(Throughput::Elements(
        (HORIZONTAL_MASS_POINTS * HORIZONTAL_MASS_POINTS * HALF_LEVELS) as u64,
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

fn worker_counts() -> Vec<usize> {
    let available = thread::available_parallelism().map_or(1, |count| count.get());
    let mut counts = vec![1, available.min(4), available];
    counts.sort_unstable();
    counts.dedup();
    counts
}

struct Fixture {
    output_volume: [CpuField<f32>; 3],
    output_horizontal: [CpuField<f32>; 4],
    input_volume: [CpuField<f32>; 7],
    input_horizontal: [CpuField<f32>; 8],
    vertical: [Vec<f32>; 6],
    region: AcousticMassThetaRegion,
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
            output_volume: [volume(0.8), volume(300.0), volume(-901.0)],
            output_horizontal: [
                horizontal(2.0),
                horizontal(-902.0),
                horizontal(-903.0),
                horizontal(-904.0),
            ],
            input_volume: [
                volume(0.2),
                volume(0.15),
                volume(0.3),
                volume(0.12),
                volume(0.35),
                volume(290.0),
                volume(0.012),
            ],
            input_horizontal: [
                horizontal(11.0),
                horizontal(3.0),
                horizontal(4.0),
                horizontal(0.03),
                horizontal(0.92),
                horizontal(1.08),
                horizontal(1.03),
                horizontal(0.97),
            ],
            vertical: [
                vec![0.45; HALF_LEVELS + 2],
                vec![0.2; HALF_LEVELS + 2],
                vec![0.18; HALF_LEVELS + 2],
                vec![0.61; HALF_LEVELS + 2],
                vec![0.39; HALF_LEVELS + 2],
                vec![1.1; HALF_LEVELS + 2],
            ],
            region: AcousticMassThetaRegion::try_new(
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
        let [
            vertical_mass_flux,
            potential_temperature,
            previous_potential_temperature,
        ] = &mut self.output_volume;
        let [
            column_mass,
            time_centered_mass,
            coupled_mass,
            damping_tendency,
        ] = &mut self.output_horizontal;
        backend
            .advance_acoustic_mass_and_theta(
                AcousticMassThetaState::new(vertical_mass_flux, column_mass, potential_temperature),
                AcousticMassThetaDiagnostics::new(
                    time_centered_mass,
                    coupled_mass,
                    damping_tendency,
                    previous_potential_temperature,
                ),
                AcousticMassThetaInputs::new(
                    AcousticMassThetaMomentumInputs::new(
                        &self.input_volume[0],
                        &self.input_volume[1],
                        &self.input_volume[2],
                        &self.input_volume[3],
                        &self.input_volume[4],
                    ),
                    AcousticMassThetaMassInputs::new(
                        &self.input_horizontal[0],
                        &self.input_horizontal[1],
                        &self.input_horizontal[2],
                        &self.input_horizontal[3],
                    ),
                    AcousticMassThetaThermodynamicInputs::new(
                        &self.input_volume[5],
                        &self.input_volume[6],
                    ),
                    AcousticMassThetaMapFactors::new(
                        &self.input_horizontal[4],
                        &self.input_horizontal[5],
                        &self.input_horizontal[6],
                        &self.input_horizontal[7],
                    ),
                    AcousticMassThetaVerticalCoefficients::new(
                        &self.vertical[0],
                        &self.vertical[1],
                        &self.vertical[2],
                        &self.vertical[3],
                        &self.vertical[4],
                        &self.vertical[5],
                    ),
                ),
                AcousticMassThetaParameters::new(0.002, 0.003, 0.4, 0.1),
                AcousticMassThetaBoundaryPolicy::new(
                    AcousticMassThetaLateralDomain::Global,
                    AcousticMassThetaWestEastPeriodicity::Nonperiodic,
                ),
                &self.region,
            )
            .unwrap();
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
