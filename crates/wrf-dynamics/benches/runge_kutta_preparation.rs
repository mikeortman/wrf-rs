//! Criterion benchmark for the integrated WRF Runge-Kutta preparation pass.

#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
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

const ACTIVE_WEST_EAST_POINTS: usize = 256;
const ACTIVE_SOUTH_NORTH_POINTS: usize = 256;
const ACTIVE_BOTTOM_TOP_POINTS: usize = 40;

fn benchmark_runge_kutta_preparation(criterion: &mut Criterion) {
    let mass_point_count =
        ACTIVE_WEST_EAST_POINTS * ACTIVE_SOUTH_NORTH_POINTS * ACTIVE_BOTTOM_TOP_POINTS;
    let mut group = criterion.benchmark_group("runge_kutta_preparation");
    group.throughput(Throughput::Elements(mass_point_count as u64));

    for worker_count in benchmark_worker_counts() {
        let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
        let mut fixture = create_fixture(&backend);
        group.bench_with_input(
            BenchmarkId::new("seven_diagnostics", worker_count),
            &worker_count,
            |bencher, _| {
                bencher.iter(|| apply(black_box(&backend), black_box(&mut fixture)));
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
    half_level_mass_multiplier: Vec<f32>,
    half_level_mass_offset: Vec<f32>,
    full_level_mass_multiplier: Vec<f32>,
    full_level_mass_offset: Vec<f32>,
    eta_layer_thickness: Vec<f32>,
    regions: RungeKuttaPreparationRegions,
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
    let vertical_length = shape.bottom_top_points();

    Fixture {
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
        mass_outputs: [
            horizontal(-9_999.0),
            horizontal(-9_999.0),
            horizontal(-9_999.0),
        ],
        momentum_outputs: [volume(-9_999.0), volume(-9_999.0), volume(-9_999.0)],
        diagnostic_outputs: [
            volume(-9_999.0),
            volume(-9_999.0),
            volume(-9_999.0),
            volume(-9_999.0),
            volume(-9_999.0),
            volume(-9_999.0),
        ],
        half_level_mass_multiplier: vec![1.0; vertical_length],
        half_level_mass_offset: vec![0.1; vertical_length],
        full_level_mass_multiplier: vec![0.9; vertical_length],
        full_level_mass_offset: vec![0.2; vertical_length],
        eta_layer_thickness: vec![-0.025; vertical_length],
        regions: create_regions(shape),
    }
}

fn create_regions(shape: GridShape) -> RungeKuttaPreparationRegions {
    let west_east_domain = 1..(ACTIVE_WEST_EAST_POINTS + 1);
    let south_north_domain = 1..(ACTIVE_SOUTH_NORTH_POINTS + 1);
    let bottom_top_domain = 1..(ACTIVE_BOTTOM_TOP_POINTS + 1);
    let west_east_tile = 1..(ACTIVE_WEST_EAST_POINTS + 2);
    let south_north_tile = 1..(ACTIVE_SOUTH_NORTH_POINTS + 2);
    let full_level_tile = 1..(ACTIVE_BOTTOM_TOP_POINTS + 2);

    RungeKuttaPreparationRegions::new(
        ColumnMassStaggeringRegion::try_new(
            shape.horizontal_shape(),
            west_east_domain.clone(),
            south_north_domain.clone(),
            west_east_tile.clone(),
            south_north_tile.clone(),
        )
        .unwrap(),
        MomentumCouplingRegion::try_new(
            shape,
            west_east_domain.clone(),
            south_north_domain.clone(),
            bottom_top_domain.clone(),
            west_east_tile.clone(),
            south_north_tile.clone(),
            full_level_tile.clone(),
        )
        .unwrap(),
        OmegaDiagnosisRegion::try_new(
            shape,
            west_east_domain.clone(),
            south_north_domain.clone(),
            bottom_top_domain.clone(),
            west_east_tile.clone(),
            south_north_tile.clone(),
            full_level_tile.clone(),
        )
        .unwrap(),
        MoistureCoefficientRegion::try_new(
            shape,
            west_east_domain.clone(),
            south_north_domain.clone(),
            bottom_top_domain.clone(),
            west_east_tile.clone(),
            south_north_tile.clone(),
            full_level_tile,
        )
        .unwrap(),
        InverseDensityRegion::try_new(
            shape,
            west_east_domain.clone(),
            south_north_domain.clone(),
            bottom_top_domain.clone(),
            west_east_tile.clone(),
            south_north_tile.clone(),
            bottom_top_domain.clone(),
        )
        .unwrap(),
        PressurePointGeopotentialRegion::try_new(
            shape,
            west_east_domain,
            south_north_domain,
            bottom_top_domain.clone(),
            west_east_tile,
            south_north_tile,
            bottom_top_domain,
        )
        .unwrap(),
    )
}

fn apply(backend: &CpuBackend, fixture: &mut Fixture) {
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
            &fixture.half_level_mass_multiplier,
            &fixture.half_level_mass_offset,
            &fixture.full_level_mass_multiplier,
            &fixture.full_level_mass_offset,
            &fixture.eta_layer_thickness,
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
    let [
        omega,
        west_east_moisture,
        south_north_moisture,
        vertical_moisture,
        inverse_density,
        geopotential,
    ] = &mut fixture.diagnostic_outputs;
    let outputs = RungeKuttaPreparationOutputs::new(
        RungeKuttaPreparationMassOutputs::new(full_mass, west_east_mass, south_north_mass),
        RungeKuttaPreparationMomentumOutputs::new(
            west_east_momentum,
            south_north_momentum,
            vertical_momentum,
        ),
        RungeKuttaPreparationDiagnosticOutputs::new(
            omega,
            west_east_moisture,
            south_north_moisture,
            vertical_moisture,
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

criterion_group!(benches, benchmark_runge_kutta_preparation);
criterion_main!(benches);
