//! Measures warmed allocations in a three-substep acoustic flux sequence.

use std::alloc::System;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    AcousticFluxAccumulationKernels, AcousticFluxAccumulationRegion, AcousticFluxCurrentFields,
    AcousticFluxLinearFields, AcousticFluxMapFactors, AcousticFluxMassFields,
    AcousticFluxRunningAverages, AcousticSubstepPhase,
};

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const HORIZONTAL_POINTS: usize = 64;
const HALF_LEVELS: usize = 40;
const SEQUENCES: usize = 100;

fn main() {
    println!("kernel,phase,workers,sequences,allocations,reallocations,bytes_allocated");
    for workers in worker_counts() {
        let backend = CpuBackend::try_with_worker_count(workers).unwrap();
        let mut fixture = Fixture::new(&backend);
        run_sequences(&backend, &mut fixture);
        let first_region = Region::new(GLOBAL_ALLOCATOR);
        run_sequences(&backend, &mut fixture);
        let first = first_region.change();
        let settled_region = Region::new(GLOBAL_ALLOCATOR);
        run_sequences(&backend, &mut fixture);
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
    current: [CpuField<f32>; 3],
    linear: [CpuField<f32>; 3],
    averages: [CpuField<f32>; 3],
    horizontal: [CpuField<f32>; 4],
    coefficients: [Vec<f32>; 2],
    region: AcousticFluxAccumulationRegion,
}

impl Fixture {
    fn new(backend: &CpuBackend) -> Self {
        let shape = GridShape::try_new(
            HORIZONTAL_POINTS + 2,
            HORIZONTAL_POINTS + 2,
            HALF_LEVELS + 2,
        )
        .unwrap();
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
                1..HORIZONTAL_POINTS + 1,
                1..HORIZONTAL_POINTS + 1,
                1..HALF_LEVELS + 1,
                1..HORIZONTAL_POINTS + 2,
                1..HORIZONTAL_POINTS + 2,
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

fn run_sequences(backend: &CpuBackend, fixture: &mut Fixture) {
    for _ in 0..SEQUENCES {
        fixture.apply(backend);
    }
}

fn print_stats(phase: &str, workers: usize, stats: Stats) {
    println!(
        "acoustic_flux_accumulation,{phase},{workers},{SEQUENCES},{},{},{}",
        stats.allocations, stats.reallocations, stats.bytes_allocated
    );
}

fn assert_budget(stats: Stats) {
    assert_eq!(stats.reallocations, 0);
    assert!(stats.allocations <= 20);
    assert!(stats.bytes_allocated <= 32 * 1_024);
}
