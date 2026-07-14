//! Measures warmed allocations in integrated Runge-Kutta preparation.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    ColumnMassStaggeringPeriodicity, ColumnMassStaggeringRegion, InverseDensityRegion,
    MoistureCoefficientRegion, MoistureSpecies, MomentumCouplingRegion, OmegaDiagnosisGridMetrics,
    OmegaDiagnosisRegion, PressurePointGeopotentialRegion, RungeKuttaPreparationCoefficients,
    RungeKuttaPreparationDiagnosticOutputs, RungeKuttaPreparationInputs,
    RungeKuttaPreparationKernels, RungeKuttaPreparationMapFactors, RungeKuttaPreparationMassInputs,
    RungeKuttaPreparationMassOutputs, RungeKuttaPreparationMomentumOutputs,
    RungeKuttaPreparationOutputs, RungeKuttaPreparationRegions,
    RungeKuttaPreparationThermodynamicInputs, RungeKuttaPreparationVelocities,
};

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const WARMUP_CALLS: usize = 100;
const MEASURED_CALLS: usize = 100;

fn main() {
    println!("kernel,phase,workers,calls,allocations,reallocations,bytes_allocated");
    for worker_count in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
        let mut fixture = Fixture::new(&backend);
        run(&backend, &mut fixture, WARMUP_CALLS);

        let first_region = Region::new(GLOBAL_ALLOCATOR);
        run(&backend, &mut fixture, MEASURED_CALLS);
        let first = first_region.change();
        let settled_region = Region::new(GLOBAL_ALLOCATOR);
        run(&backend, &mut fixture, MEASURED_CALLS);
        let settled = settled_region.change();

        print_stats("first", worker_count, first);
        print_stats("settled", worker_count, settled);
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
    perturbation_mass: CpuField<f32>,
    base_mass: CpuField<f32>,
    velocities: [CpuField<f32>; 3],
    map_factors: [CpuField<f32>; 4],
    moisture: Vec<CpuField<f32>>,
    thermodynamics: [CpuField<f32>; 4],
    mass_outputs: [CpuField<f32>; 3],
    momentum_outputs: [CpuField<f32>; 3],
    diagnostic_outputs: [CpuField<f32>; 6],
    regions: RungeKuttaPreparationRegions,
}

impl Fixture {
    fn new(backend: &CpuBackend) -> Self {
        let shape = GridShape::try_new(6, 6, 5).unwrap();
        let horizontal_shape = shape.horizontal_shape();
        let volume = |value| backend.create_field(shape, value).unwrap();
        let horizontal = |value| backend.create_field(horizontal_shape, value).unwrap();

        Self {
            perturbation_mass: horizontal(10.0),
            base_mass: horizontal(90.0),
            velocities: [volume(1.0), volume(-0.5), volume(0.25)],
            map_factors: [
                horizontal(1.0),
                horizontal(1.1),
                horizontal(0.9),
                horizontal(0.8),
            ],
            moisture: vec![volume(0.001), volume(0.002)],
            thermodynamics: [volume(0.2), volume(0.8), volume(100.0), volume(1_000.0)],
            mass_outputs: [horizontal(-1.0), horizontal(-1.0), horizontal(-1.0)],
            momentum_outputs: [volume(-1.0), volume(-1.0), volume(-1.0)],
            diagnostic_outputs: [
                volume(-1.0),
                volume(-1.0),
                volume(-1.0),
                volume(-1.0),
                volume(-1.0),
                volume(-1.0),
            ],
            regions: RungeKuttaPreparationRegions::new(
                ColumnMassStaggeringRegion::try_new(horizontal_shape, 1..5, 1..5, 2..5, 2..5)
                    .unwrap(),
                MomentumCouplingRegion::try_new(shape, 1..5, 1..5, 1..4, 2..5, 2..5, 1..5).unwrap(),
                OmegaDiagnosisRegion::try_new(shape, 1..5, 1..5, 1..4, 2..5, 2..5, 1..5).unwrap(),
                MoistureCoefficientRegion::try_new(shape, 1..5, 1..5, 1..4, 2..5, 2..5, 1..5)
                    .unwrap(),
                InverseDensityRegion::try_new(shape, 1..5, 1..5, 1..4, 2..5, 2..5, 1..4).unwrap(),
                PressurePointGeopotentialRegion::try_new(shape, 1..5, 1..5, 1..4, 2..5, 2..5, 1..4)
                    .unwrap(),
            ),
        }
    }
}

fn run(backend: &CpuBackend, fixture: &mut Fixture, calls: usize) {
    for _ in 0..calls {
        let inputs = RungeKuttaPreparationInputs::new(
            RungeKuttaPreparationMassInputs::new(&fixture.perturbation_mass, &fixture.base_mass),
            RungeKuttaPreparationVelocities::new(
                &fixture.velocities[0],
                &fixture.velocities[1],
                &fixture.velocities[2],
            ),
            RungeKuttaPreparationMapFactors::new(
                &fixture.map_factors[0],
                &fixture.map_factors[1],
                &fixture.map_factors[2],
                &fixture.map_factors[3],
            ),
            RungeKuttaPreparationCoefficients::new(
                &[1.0; 5],
                &[0.1; 5],
                &[0.9; 5],
                &[0.2; 5],
                &[-0.25; 5],
            ),
            MoistureSpecies::new(&fixture.moisture),
            RungeKuttaPreparationThermodynamicInputs::new(
                &fixture.thermodynamics[0],
                &fixture.thermodynamics[1],
                &fixture.thermodynamics[2],
                &fixture.thermodynamics[3],
            ),
            OmegaDiagnosisGridMetrics::new(1.0, 1.0),
        );
        let [full_mass, west_east_mass, south_north_mass] = &mut fixture.mass_outputs;
        let [west_east_momentum, south_north_momentum, vertical_momentum] =
            &mut fixture.momentum_outputs;
        let [omega, cqu, cqv, cqw, inverse_density, geopotential] = &mut fixture.diagnostic_outputs;
        let outputs = RungeKuttaPreparationOutputs::new(
            RungeKuttaPreparationMassOutputs::new(full_mass, west_east_mass, south_north_mass),
            RungeKuttaPreparationMomentumOutputs::new(
                west_east_momentum,
                south_north_momentum,
                vertical_momentum,
            ),
            RungeKuttaPreparationDiagnosticOutputs::new(
                omega,
                cqu,
                cqv,
                cqw,
                inverse_density,
                geopotential,
            ),
        );
        backend
            .prepare_runge_kutta_step(
                outputs,
                inputs,
                &fixture.regions,
                ColumnMassStaggeringPeriodicity::None,
            )
            .unwrap();
    }
}

fn print_stats(phase: &str, workers: usize, stats: Stats) {
    println!(
        "runge_kutta_preparation,{phase},{workers},{MEASURED_CALLS},{},{},{}",
        stats.allocations, stats.reallocations, stats.bytes_allocated
    );
}

fn assert_budget(stats: Stats) {
    assert_eq!(stats.reallocations, 0, "preparation reallocated memory");
    assert!(
        stats.allocations <= 24,
        "preparation exceeded 24 scheduler allocations per 100 calls"
    );
    assert!(
        stats.bytes_allocated <= 64 * 1_024,
        "preparation allocated more than 64 KiB per measured batch"
    );
}
