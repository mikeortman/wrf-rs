//! Measures warmed allocations in WRF acoustic small-step preparation.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    AcousticStepPreparationCoefficients, AcousticStepPreparationColumnMassTimeLevels,
    AcousticStepPreparationDiagnosticInputs, AcousticStepPreparationKernels,
    AcousticStepPreparationMapFactors, AcousticStepPreparationMassInputs,
    AcousticStepPreparationMassOutputs, AcousticStepPreparationPhase,
    AcousticStepPreparationRegion, AcousticStepPreparationSavedOutputs,
    AcousticStepPreparationVolumeTimeLevels,
};

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const ACTIVE_WEST_EAST_POINTS: usize = 64;
const ACTIVE_SOUTH_NORTH_POINTS: usize = 64;
const ACTIVE_BOTTOM_TOP_POINTS: usize = 40;
const DISPATCHES: usize = 100;

fn main() {
    println!("kernel,phase,workers,dispatches,allocations,reallocations,bytes_allocated");
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = create_fixture(&backend);
        run_dispatches(&backend, &mut fixture);
        let first_region = Region::new(GLOBAL_ALLOCATOR);
        run_dispatches(&backend, &mut fixture);
        let first = first_region.change();
        let settled_region = Region::new(GLOBAL_ALLOCATOR);
        run_dispatches(&backend, &mut fixture);
        let settled = settled_region.change();
        print_stats("first", workers, first);
        print_stats("settled", workers, settled);
        assert_budget(first);
        assert_budget(settled);
    }
}

fn worker_counts() -> Vec<usize> {
    let available = std::thread::available_parallelism().map_or(1, |count| count.get());
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
    let shape = GridShape::try_new(
        ACTIVE_WEST_EAST_POINTS + 2,
        ACTIVE_SOUTH_NORTH_POINTS + 2,
        ACTIVE_BOTTOM_TOP_POINTS + 2,
    )
    .unwrap();
    let horizontal_shape = shape.horizontal_shape();
    let volume = |value| backend.create_field(shape, value).unwrap();
    let horizontal = |value| backend.create_field(horizontal_shape, value).unwrap();
    Fixture {
        previous: [
            volume(1.0),
            volume(2.0),
            volume(-1.0),
            volume(300.0),
            volume(1_000.0),
        ],
        current: [
            volume(0.8),
            volume(1.6),
            volume(-0.7),
            volume(299.0),
            volume(900.0),
        ],
        saved: [
            volume(-999.0),
            volume(-999.0),
            volume(-999.0),
            volume(-999.0),
            volume(-999.0),
        ],
        saved_omega: volume(-999.0),
        pressure_coefficient: volume(-999.0),
        diagnostics: [volume(80_000.0), volume(500.0), volume(0.8), volume(0.3)],
        previous_mass: horizontal(1.0),
        current_mass: horizontal(-0.5),
        mass_inputs: [
            horizontal(40.0),
            horizontal(42.0),
            horizontal(43.0),
            horizontal(44.0),
        ],
        mass_outputs: [
            horizontal(-999.0),
            horizontal(-999.0),
            horizontal(-999.0),
            horizontal(-999.0),
        ],
        saved_mass: horizontal(-999.0),
        maps: [horizontal(1.03), horizontal(1.0 / 0.97), horizontal(1.12)],
        coefficients: [
            vec![0.2; ACTIVE_BOTTOM_TOP_POINTS + 2],
            vec![0.4; ACTIVE_BOTTOM_TOP_POINTS + 2],
            vec![0.3; ACTIVE_BOTTOM_TOP_POINTS + 2],
            vec![0.5; ACTIVE_BOTTOM_TOP_POINTS + 2],
        ],
        region: AcousticStepPreparationRegion::try_new(
            shape,
            1..(ACTIVE_WEST_EAST_POINTS + 1),
            1..(ACTIVE_SOUTH_NORTH_POINTS + 1),
            1..(ACTIVE_BOTTOM_TOP_POINTS + 1),
            1..(ACTIVE_WEST_EAST_POINTS + 2),
            1..(ACTIVE_SOUTH_NORTH_POINTS + 2),
            1..(ACTIVE_BOTTOM_TOP_POINTS + 2),
        )
        .unwrap(),
    }
}

fn run_dispatches(backend: &CpuBackend, fixture: &mut Fixture) {
    for _ in 0..DISPATCHES {
        let [u1, v1, w1, t1, ph1] = &mut fixture.previous;
        let [u2, v2, w2, t2, ph2] = &mut fixture.current;
        let [us, vs, ws, ts, phs] = &mut fixture.saved;
        let [mub, muu, muv, mutf] = &fixture.mass_inputs;
        let [muus, muvs, muts, mudf] = &mut fixture.mass_outputs;
        let [pb, p, alt, ww] = &fixture.diagnostics;
        let [msfuy, msfvxi, msfty] = &fixture.maps;
        backend
            .prepare_acoustic_step(
                AcousticStepPreparationVolumeTimeLevels::new(
                    u1, u2, v1, v2, w1, w2, t1, t2, ph1, ph2,
                ),
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
}

fn print_stats(phase: &str, workers: usize, stats: Stats) {
    println!(
        "acoustic_step_preparation,{phase},{workers},{DISPATCHES},{},{},{}",
        stats.allocations, stats.reallocations, stats.bytes_allocated
    );
}

fn assert_budget(stats: Stats) {
    assert_eq!(stats.reallocations, 0);
    assert!(stats.allocations <= DISPATCHES * 40);
    assert!(stats.bytes_allocated <= 8 * 1_024 * 1_024);
}
