//! Measures warmed allocations in WRF acoustic pressure diagnosis.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    AcousticPressureCoefficients, AcousticPressureDampingPhase, AcousticPressureKernels,
    AcousticPressureMasses, AcousticPressureMode, AcousticPressureParameters,
    AcousticPressureRegion, AcousticPressureState, AcousticPressureThermodynamics,
    AcousticPressureVerticalMetrics,
};

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const ACTIVE_WEST_EAST_POINTS: usize = 64;
const ACTIVE_SOUTH_NORTH_POINTS: usize = 64;
const ACTIVE_BOTTOM_TOP_POINTS: usize = 40;
const DISPATCHES: usize = 100;

fn main() {
    println!("kernel,mode,phase,workers,dispatches,allocations,reallocations,bytes_allocated");
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        for mode in [
            AcousticPressureMode::Nonhydrostatic,
            AcousticPressureMode::Hydrostatic,
        ] {
            let mut fixture = create_fixture(&backend);
            run_dispatches(&backend, &mut fixture, mode);
            let first_region = Region::new(GLOBAL_ALLOCATOR);
            run_dispatches(&backend, &mut fixture, mode);
            let first = first_region.change();
            let settled_region = Region::new(GLOBAL_ALLOCATOR);
            run_dispatches(&backend, &mut fixture, mode);
            let settled = settled_region.change();
            print_stats(mode, "first", workers, first);
            print_stats(mode, "settled", workers, settled);
            assert_budget(first);
            assert_budget(settled);
        }
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
    state: [CpuField<f32>; 4],
    thermodynamics: [CpuField<f32>; 4],
    masses: [CpuField<f32>; 2],
    coefficients: [Vec<f32>; 3],
    vertical_metrics: [Vec<f32>; 2],
    region: AcousticPressureRegion,
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
        state: [volume(-999.0), volume(-999.0), volume(900.0), volume(500.0)],
        thermodynamics: [volume(0.8), volume(2.0), volume(1.0), volume(140_000.0)],
        masses: [horizontal(1.0), horizontal(40.0)],
        coefficients: [
            vec![0.2; ACTIVE_BOTTOM_TOP_POINTS + 2],
            vec![0.4; ACTIVE_BOTTOM_TOP_POINTS + 2],
            vec![1.1; ACTIVE_BOTTOM_TOP_POINTS + 2],
        ],
        vertical_metrics: [
            vec![1.3; ACTIVE_BOTTOM_TOP_POINTS + 2],
            vec![0.7; ACTIVE_BOTTOM_TOP_POINTS + 2],
        ],
        region: AcousticPressureRegion::try_new(
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

fn run_dispatches(backend: &CpuBackend, fixture: &mut Fixture, mode: AcousticPressureMode) {
    for _ in 0..DISPATCHES {
        let [inverse_density, pressure, geopotential, previous_pressure] = &mut fixture.state;
        backend
            .update_acoustic_pressure(
                AcousticPressureState::new(
                    inverse_density,
                    pressure,
                    geopotential,
                    previous_pressure,
                ),
                AcousticPressureThermodynamics::new(
                    &fixture.thermodynamics[0],
                    &fixture.thermodynamics[1],
                    &fixture.thermodynamics[2],
                    &fixture.thermodynamics[3],
                ),
                AcousticPressureMasses::new(&fixture.masses[0], &fixture.masses[1]),
                AcousticPressureCoefficients::new(
                    &fixture.coefficients[0],
                    &fixture.coefficients[1],
                    &fixture.coefficients[2],
                ),
                AcousticPressureVerticalMetrics::new(
                    &fixture.vertical_metrics[0],
                    &fixture.vertical_metrics[1],
                ),
                AcousticPressureParameters::new(300.0, 0.17),
                mode,
                AcousticPressureDampingPhase::Initialize,
                &fixture.region,
            )
            .unwrap();
    }
}

fn print_stats(mode: AcousticPressureMode, phase: &str, workers: usize, stats: Stats) {
    let mode = match mode {
        AcousticPressureMode::Nonhydrostatic => "nonhydrostatic",
        AcousticPressureMode::Hydrostatic => "hydrostatic",
    };
    println!(
        "acoustic_pressure,{mode},{phase},{workers},{DISPATCHES},{},{},{}",
        stats.allocations, stats.reallocations, stats.bytes_allocated
    );
}

fn assert_budget(stats: Stats) {
    assert_eq!(stats.reallocations, 0);
    assert!(stats.allocations <= 10);
    assert!(stats.bytes_allocated <= 32 * 1_024);
}
