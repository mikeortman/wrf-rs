use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField};

use crate::{
    AcousticFluxAccumulationRegion, AcousticFluxAccumulationResult, AcousticFluxCurrentFields,
    AcousticFluxLinearFields, AcousticFluxMapFactors, AcousticFluxMassFields,
    AcousticFluxRunningAverages, AcousticSubstepPhase,
};

use super::map_parallel_error;

#[allow(clippy::too_many_arguments)]
pub(super) fn apply(
    backend: &CpuBackend,
    averages: AcousticFluxRunningAverages<'_, CpuField<f32>>,
    current: AcousticFluxCurrentFields<'_, CpuField<f32>>,
    linear: AcousticFluxLinearFields<'_, CpuField<f32>>,
    masses: AcousticFluxMassFields<'_, CpuField<f32>>,
    map_factors: AcousticFluxMapFactors<'_, CpuField<f32>>,
    column_mass_multiplier: &[f32],
    column_mass_offset: &[f32],
    phase: AcousticSubstepPhase,
    region: &AcousticFluxAccumulationRegion,
) -> AcousticFluxAccumulationResult<()> {
    let ranges = region.active_ranges();
    if phase.is_first() {
        zero_tile(backend, &mut *averages.west_east, region, &ranges)?;
        zero_tile(backend, &mut *averages.south_north, region, &ranges)?;
        zero_tile(backend, &mut *averages.vertical, region, &ranges)?;
    }
    update_field(
        backend,
        averages.west_east,
        current.west_east,
        phase,
        region,
        &ranges.staggered_west_east,
        &ranges.half_levels,
        &ranges.mass_south_north,
        |west_east, level, south_north| {
            let horizontal = horizontal_index(region, west_east, south_north);
            (column_mass_multiplier[level] * masses.west_east.values()[horizontal]
                + column_mass_offset[level])
                * linear.west_east.values()[index(region, west_east, level, south_north)]
                / map_factors.west_east_y.values()[horizontal]
        },
    )?;
    update_field(
        backend,
        averages.south_north,
        current.south_north,
        phase,
        region,
        &ranges.mass_west_east,
        &ranges.half_levels,
        &ranges.staggered_south_north,
        |west_east, level, south_north| {
            let horizontal = horizontal_index(region, west_east, south_north);
            (column_mass_multiplier[level] * masses.south_north.values()[horizontal]
                + column_mass_offset[level])
                * linear.south_north.values()[index(region, west_east, level, south_north)]
                * map_factors.inverse_south_north_x.values()[horizontal]
        },
    )?;
    update_field(
        backend,
        averages.vertical,
        current.vertical,
        phase,
        region,
        &ranges.mass_west_east,
        &ranges.full_levels,
        &ranges.mass_south_north,
        |west_east, level, south_north| {
            linear.vertical.values()[index(region, west_east, level, south_north)]
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn update_field<LinearTerm>(
    backend: &CpuBackend,
    average: &mut CpuField<f32>,
    current: &CpuField<f32>,
    phase: AcousticSubstepPhase,
    region: &AcousticFluxAccumulationRegion,
    west_east_range: &std::ops::Range<usize>,
    level_range: &std::ops::Range<usize>,
    south_north_range: &std::ops::Range<usize>,
    linear_term: LinearTerm,
) -> AcousticFluxAccumulationResult<()>
where
    LinearTerm: Fn(usize, usize, usize) -> f32 + Send + Sync,
{
    let width = region.shape().west_east_points();
    let levels = region.shape().bottom_top_points();
    let plane_length = width * levels;
    let current_values = current.values();
    backend
        .try_for_each_output_block(
            average.values_mut(),
            plane_length,
            |south_north, output_plane| {
                if south_north_range.contains(&south_north) {
                    for level in level_range.clone() {
                        let row_start = level * width;
                        for west_east in west_east_range.clone() {
                            let local = row_start + west_east;
                            let global = south_north * plane_length + local;
                            output_plane[local] += current_values[global];
                            if phase.is_last() {
                                output_plane[local] = output_plane[local] / phase.count() as f32
                                    + linear_term(west_east, level, south_north);
                            }
                        }
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

fn zero_tile(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    region: &AcousticFluxAccumulationRegion,
    ranges: &super::super::region::AcousticFluxActiveRanges,
) -> AcousticFluxAccumulationResult<()> {
    let width = region.shape().west_east_points();
    let plane_length = width * region.shape().bottom_top_points();
    backend
        .try_for_each_output_block(
            output.values_mut(),
            plane_length,
            |south_north, output_plane| {
                if ranges.staggered_south_north.contains(&south_north) {
                    for level in ranges.full_levels.clone() {
                        let row_start = level * width;
                        for west_east in ranges.staggered_west_east.clone() {
                            output_plane[row_start + west_east] = 0.0;
                        }
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

fn index(
    region: &AcousticFluxAccumulationRegion,
    west_east: usize,
    level: usize,
    south_north: usize,
) -> usize {
    west_east
        + region.shape().west_east_points()
            * (level + region.shape().bottom_top_points() * south_north)
}

fn horizontal_index(
    region: &AcousticFluxAccumulationRegion,
    west_east: usize,
    south_north: usize,
) -> usize {
    west_east + region.shape().west_east_points() * south_north
}
