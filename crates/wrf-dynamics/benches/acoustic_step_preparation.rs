//! Criterion benchmark for WRF acoustic small-step preparation.
#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::thread;
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::*;

const NX: usize = 256;
const NY: usize = 256;
const NZ: usize = 40;

fn benchmark(criterion: &mut Criterion) {
    let updated_values = 17 * NX * NY * NZ + 3 * (NX + NY) * NZ + 14 * NX * NY + NX + NY;
    let mut group = criterion.benchmark_group("acoustic_step_preparation");
    group.throughput(Throughput::Elements(updated_values as u64));
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = create_fixture(&backend);
        group.bench_with_input(
            BenchmarkId::new("first_substep", workers),
            &workers,
            |bencher, _| {
                bencher.iter(|| apply_fixture(black_box(&backend), black_box(&mut fixture)));
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
    previous: [CpuField<f32>; 5],
    current: [CpuField<f32>; 5],
    saved: [CpuField<f32>; 5],
    saved_omega: CpuField<f32>,
    pressure_coefficient: CpuField<f32>,
    diagnostics: [CpuField<f32>; 4],
    previous_mass: CpuField<f32>,
    current_mass: CpuField<f32>,
    mass_inputs: [CpuField<f32>; 4],
    mass_outputs: [CpuField<f32>; 4],
    saved_mass: CpuField<f32>,
    maps: [CpuField<f32>; 3],
    coefficients: [Vec<f32>; 4],
    region: AcousticStepPreparationRegion,
}

fn create_fixture(backend: &CpuBackend) -> Fixture {
    let shape = GridShape::try_new(NX + 2, NY + 2, NZ + 2).unwrap();
    let horizontal_shape = shape.horizontal_shape();
    let volume = |value| backend.create_field(shape, value).unwrap();
    let horizontal = |value| backend.create_field(horizontal_shape, value).unwrap();
    Fixture {
        previous: [
            volume(1.),
            volume(2.),
            volume(-1.),
            volume(300.),
            volume(1000.),
        ],
        current: [
            volume(0.8),
            volume(1.6),
            volume(-0.7),
            volume(299.),
            volume(900.),
        ],
        saved: [
            volume(-999.),
            volume(-999.),
            volume(-999.),
            volume(-999.),
            volume(-999.),
        ],
        saved_omega: volume(-999.),
        pressure_coefficient: volume(-999.),
        diagnostics: [volume(80_000.), volume(500.), volume(0.8), volume(0.3)],
        previous_mass: horizontal(1.),
        current_mass: horizontal(-0.5),
        mass_inputs: [
            horizontal(40.),
            horizontal(42.),
            horizontal(43.),
            horizontal(44.),
        ],
        mass_outputs: [
            horizontal(-999.),
            horizontal(-999.),
            horizontal(-999.),
            horizontal(-999.),
        ],
        saved_mass: horizontal(-999.),
        maps: [horizontal(1.03), horizontal(1. / 0.97), horizontal(1.12)],
        coefficients: [
            vec![0.2; NZ + 2],
            vec![0.4; NZ + 2],
            vec![0.3; NZ + 2],
            vec![0.5; NZ + 2],
        ],
        region: AcousticStepPreparationRegion::try_new(
            shape,
            1..NX + 1,
            1..NY + 1,
            1..NZ + 1,
            1..NX + 2,
            1..NY + 2,
            1..NZ + 2,
        )
        .unwrap(),
    }
}

fn apply_fixture(backend: &CpuBackend, fixture: &mut Fixture) {
    let [u1, v1, w1, t1, ph1] = &mut fixture.previous;
    let [u2, v2, w2, t2, ph2] = &mut fixture.current;
    let [us, vs, ws, ts, phs] = &mut fixture.saved;
    let [mub, muu, muv, mutf] = &fixture.mass_inputs;
    let [muus, muvs, muts, mudf] = &mut fixture.mass_outputs;
    let [pb, p, alt, ww] = &fixture.diagnostics;
    let [msfuy, msfvxi, msfty] = &fixture.maps;
    backend
        .prepare_acoustic_step(
            AcousticStepPreparationVolumeTimeLevels::new(u1, u2, v1, v2, w1, w2, t1, t2, ph1, ph2),
            AcousticStepPreparationColumnMassTimeLevels::new(
                &mut fixture.previous_mass,
                &mut fixture.current_mass,
            ),
            AcousticStepPreparationSavedOutputs::new(
                us,
                vs,
                ws,
                ts,
                phs,
                &mut fixture.saved_mass,
                &mut fixture.saved_omega,
                &mut fixture.pressure_coefficient,
            ),
            AcousticStepPreparationMassOutputs::new(muus, muvs, muts, mudf),
            AcousticStepPreparationMassInputs::new(mub, muu, muv, mutf),
            AcousticStepPreparationDiagnosticInputs::new(pb, p, alt, ww),
            AcousticStepPreparationMapFactors::new(msfuy, msfvxi, msfty),
            AcousticStepPreparationCoefficients::new(
                &fixture.coefficients[0],
                &fixture.coefficients[1],
                &fixture.coefficients[2],
                &fixture.coefficients[3],
            ),
            AcousticStepPreparationPhase::FirstSubstep,
            &fixture.region,
        )
        .unwrap();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
