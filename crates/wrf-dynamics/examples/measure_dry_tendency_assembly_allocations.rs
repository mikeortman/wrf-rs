//! Measures warmed allocations in WRF dry-tendency assembly.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    DryTendencyAssemblyCoefficients, DryTendencyAssemblyForwardTendencies,
    DryTendencyAssemblyKernels, DryTendencyAssemblyMapFactors, DryTendencyAssemblyPhase,
    DryTendencyAssemblyRegion, DryTendencyAssemblyRungeKuttaTendencies,
    DryTendencyAssemblySavedTendencies, DryTendencyAssemblyThermodynamics,
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
    runge_kutta: [CpuField<f32>; 5],
    forward: [CpuField<f32>; 5],
    saved: [CpuField<f32>; 5],
    heating: CpuField<f32>,
    runge_kutta_mass: CpuField<f32>,
    forward_mass: CpuField<f32>,
    full_mass: CpuField<f32>,
    maps: [CpuField<f32>; 4],
    c1: Vec<f32>,
    c2: Vec<f32>,
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
        runge_kutta: [
            volume(1.0),
            volume(2.0),
            volume(-1.0),
            volume(3.0),
            volume(-2.0),
        ],
        forward: [
            volume(0.3),
            volume(-0.4),
            volume(0.5),
            volume(-0.6),
            volume(0.7),
        ],
        saved: [
            volume(0.09),
            volume(-0.08),
            volume(0.07),
            volume(-0.06),
            volume(0.05),
        ],
        heating: volume(0.001),
        runge_kutta_mass: horizontal(0.6),
        forward_mass: horizontal(-0.2),
        full_mass: horizontal(50.0),
        maps: [
            horizontal(1.03),
            horizontal(0.97),
            horizontal(1.0 / 0.97),
            horizontal(1.12),
        ],
        c1: vec![0.2; 42],
        c2: vec![0.4; 42],
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

fn run_dispatches(backend: &CpuBackend, fixture: &mut Fixture) {
    for _ in 0..DISPATCHES {
        let [ru, rv, rw, ph, t] = &mut fixture.runge_kutta;
        let [ruf, rvf, rwf, phf, tf] = &mut fixture.forward;
        backend
            .assemble_dry_tendencies(
                DryTendencyAssemblyRungeKuttaTendencies::new(
                    ru,
                    rv,
                    rw,
                    ph,
                    t,
                    &mut fixture.runge_kutta_mass,
                ),
                DryTendencyAssemblyForwardTendencies::new(
                    ruf,
                    rvf,
                    rwf,
                    phf,
                    tf,
                    &fixture.forward_mass,
                ),
                DryTendencyAssemblySavedTendencies::new(
                    &fixture.saved[0],
                    &fixture.saved[1],
                    &fixture.saved[2],
                    &fixture.saved[3],
                    &fixture.saved[4],
                ),
                DryTendencyAssemblyThermodynamics::new(&fixture.heating, &fixture.full_mass),
                DryTendencyAssemblyMapFactors::new(
                    &fixture.maps[0],
                    &fixture.maps[1],
                    &fixture.maps[2],
                    &fixture.maps[3],
                ),
                DryTendencyAssemblyCoefficients::new(&fixture.c1, &fixture.c2),
                DryTendencyAssemblyPhase::FirstSubstep,
                &fixture.region,
            )
            .unwrap();
    }
}

fn print_stats(phase: &str, workers: usize, stats: Stats) {
    println!(
        "dry_tendency_assembly,{phase},{workers},{DISPATCHES},{},{},{}",
        stats.allocations, stats.reallocations, stats.bytes_allocated
    );
}

fn assert_budget(stats: Stats) {
    assert_eq!(stats.reallocations, 0);
    assert!(stats.allocations <= DISPATCHES * 7);
    assert!(stats.bytes_allocated <= 2 * 1_024 * 1_024);
}
