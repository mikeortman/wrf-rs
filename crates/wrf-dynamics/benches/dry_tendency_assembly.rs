//! Criterion throughput benchmark for WRF dry-tendency assembly.

#![allow(missing_docs)]

use std::hint::black_box;
use std::thread;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    DryTendencyAssemblyCoefficients, DryTendencyAssemblyForwardTendencies,
    DryTendencyAssemblyKernels, DryTendencyAssemblyMapFactors, DryTendencyAssemblyPhase,
    DryTendencyAssemblyRegion, DryTendencyAssemblyRungeKuttaTendencies,
    DryTendencyAssemblySavedTendencies, DryTendencyAssemblyThermodynamics,
};

const ACTIVE_WEST_EAST_POINTS: usize = 256;
const ACTIVE_SOUTH_NORTH_POINTS: usize = 256;
const ACTIVE_BOTTOM_TOP_POINTS: usize = 40;

fn benchmark_dry_tendency_assembly(criterion: &mut Criterion) {
    let active_volume_points =
        ACTIVE_WEST_EAST_POINTS * ACTIVE_SOUTH_NORTH_POINTS * ACTIVE_BOTTOM_TOP_POINTS;
    let active_horizontal_points = ACTIVE_WEST_EAST_POINTS * ACTIVE_SOUTH_NORTH_POINTS;
    let updated_values = active_volume_points * 10 + active_horizontal_points * 5;
    let mut group = criterion.benchmark_group("dry_tendency_assembly");
    group.throughput(Throughput::Elements(updated_values as u64));

    for worker_count in benchmark_worker_counts() {
        let backend = CpuBackend::try_with_worker_count(worker_count).unwrap();
        let mut fixture = create_fixture(&backend);
        group.bench_with_input(
            BenchmarkId::new("first_substep", worker_count),
            &worker_count,
            |bencher, _| {
                bencher.iter(|| apply_fixture(black_box(&backend), black_box(&mut fixture)));
            },
        );
    }
    group.finish();
}

fn benchmark_worker_counts() -> Vec<usize> {
    let available = thread::available_parallelism().map_or(1, |count| count.get());
    let mut counts = vec![1, available.min(4), available];
    counts.sort_unstable();
    counts.dedup();
    counts
}

struct Fixture {
    runge_kutta_volume: [CpuField<f32>; 5],
    forward_volume: [CpuField<f32>; 5],
    saved_volume: [CpuField<f32>; 5],
    diabatic_heating: CpuField<f32>,
    runge_kutta_column_mass: CpuField<f32>,
    forward_column_mass: CpuField<f32>,
    full_column_mass: CpuField<f32>,
    map_factors: [CpuField<f32>; 4],
    full_mass_multiplier: Vec<f32>,
    vertical_offset: Vec<f32>,
    region: DryTendencyAssemblyRegion,
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
        runge_kutta_volume: [
            volume(1.0),
            volume(2.0),
            volume(-1.0),
            volume(3.0),
            volume(-2.0),
        ],
        forward_volume: [
            volume(0.3),
            volume(-0.4),
            volume(0.5),
            volume(-0.6),
            volume(0.7),
        ],
        saved_volume: [
            volume(0.09),
            volume(-0.08),
            volume(0.07),
            volume(-0.06),
            volume(0.05),
        ],
        diabatic_heating: volume(0.001),
        runge_kutta_column_mass: horizontal(0.6),
        forward_column_mass: horizontal(-0.2),
        full_column_mass: horizontal(50.0),
        map_factors: [
            horizontal(1.03),
            horizontal(0.97),
            horizontal(1.0 / 0.97),
            horizontal(1.12),
        ],
        full_mass_multiplier: vec![0.2; shape.bottom_top_points()],
        vertical_offset: vec![0.4; shape.bottom_top_points()],
        region: DryTendencyAssemblyRegion::try_new(
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

fn apply_fixture(backend: &CpuBackend, fixture: &mut Fixture) {
    let [ru, rv, rw, ph, t] = &mut fixture.runge_kutta_volume;
    let [ruf, rvf, rwf, phf, tf] = &mut fixture.forward_volume;
    backend
        .assemble_dry_tendencies(
            DryTendencyAssemblyRungeKuttaTendencies::new(
                ru,
                rv,
                rw,
                ph,
                t,
                &mut fixture.runge_kutta_column_mass,
            ),
            DryTendencyAssemblyForwardTendencies::new(
                ruf,
                rvf,
                rwf,
                phf,
                tf,
                &fixture.forward_column_mass,
            ),
            DryTendencyAssemblySavedTendencies::new(
                &fixture.saved_volume[0],
                &fixture.saved_volume[1],
                &fixture.saved_volume[2],
                &fixture.saved_volume[3],
                &fixture.saved_volume[4],
            ),
            DryTendencyAssemblyThermodynamics::new(
                &fixture.diabatic_heating,
                &fixture.full_column_mass,
            ),
            DryTendencyAssemblyMapFactors::new(
                &fixture.map_factors[0],
                &fixture.map_factors[1],
                &fixture.map_factors[2],
                &fixture.map_factors[3],
            ),
            DryTendencyAssemblyCoefficients::new(
                &fixture.full_mass_multiplier,
                &fixture.vertical_offset,
            ),
            DryTendencyAssemblyPhase::FirstSubstep,
            &fixture.region,
        )
        .unwrap();
}

criterion_group!(benches, benchmark_dry_tendency_assembly);
criterion_main!(benches);
