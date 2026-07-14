use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField};

use crate::{AcousticMassThetaParameters, AcousticMassThetaRegion, AcousticMassThetaResult};

use super::super::domain::AcousticMassThetaActiveRanges;
use super::map_parallel_error;

pub(super) fn save_previous(
    backend: &CpuBackend,
    previous_mass_scratch: &mut CpuField<f32>,
    column_mass: &CpuField<f32>,
    region: &AcousticMassThetaRegion,
    ranges: &AcousticMassThetaActiveRanges,
) -> AcousticMassThetaResult<()> {
    copy_active_horizontal(backend, previous_mass_scratch, column_mass, region, ranges)
}

pub(super) fn advance(
    backend: &CpuBackend,
    column_mass: &mut CpuField<f32>,
    integrated_tendency: &CpuField<f32>,
    large_step_tendency: &CpuField<f32>,
    parameters: AcousticMassThetaParameters,
    region: &AcousticMassThetaRegion,
    ranges: &AcousticMassThetaActiveRanges,
) -> AcousticMassThetaResult<()> {
    let width = region.shape().west_east_points();
    let integrated = integrated_tendency.values();
    let large_step = large_step_tendency.values();
    backend
        .try_for_each_output_block(
            column_mass.values_mut(),
            width,
            |south_north, output_row| {
                if ranges.south_north.contains(&south_north) {
                    let row_start = south_north * width;
                    for west_east in ranges.west_east.clone() {
                        let index = row_start + west_east;
                        output_row[west_east] +=
                            parameters.acoustic_time_step * (integrated[index] + large_step[index]);
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

pub(super) fn save_damping_tendency(
    backend: &CpuBackend,
    damping_tendency: &mut CpuField<f32>,
    integrated_tendency: &CpuField<f32>,
    large_step_tendency: &CpuField<f32>,
    region: &AcousticMassThetaRegion,
    ranges: &AcousticMassThetaActiveRanges,
) -> AcousticMassThetaResult<()> {
    let width = region.shape().west_east_points();
    let integrated = integrated_tendency.values();
    let large_step = large_step_tendency.values();
    backend
        .try_for_each_output_block(
            damping_tendency.values_mut(),
            width,
            |south_north, output_row| {
                if ranges.south_north.contains(&south_north) {
                    let row_start = south_north * width;
                    for west_east in ranges.west_east.clone() {
                        let index = row_start + west_east;
                        output_row[west_east] = integrated[index] + large_step[index];
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

pub(super) fn center(
    backend: &CpuBackend,
    time_centered_mass: &mut CpuField<f32>,
    column_mass: &CpuField<f32>,
    previous_mass_scratch: &CpuField<f32>,
    parameters: AcousticMassThetaParameters,
    region: &AcousticMassThetaRegion,
    ranges: &AcousticMassThetaActiveRanges,
) -> AcousticMassThetaResult<()> {
    let width = region.shape().west_east_points();
    let current = column_mass.values();
    let previous = previous_mass_scratch.values();
    backend
        .try_for_each_output_block(
            time_centered_mass.values_mut(),
            width,
            |south_north, output_row| {
                if ranges.south_north.contains(&south_north) {
                    let row_start = south_north * width;
                    for west_east in ranges.west_east.clone() {
                        let index = row_start + west_east;
                        output_row[west_east] = 0.5
                            * ((1.0 + parameters.time_centering) * current[index]
                                + (1.0 - parameters.time_centering) * previous[index]);
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

pub(super) fn couple(
    backend: &CpuBackend,
    coupled_mass: &mut CpuField<f32>,
    column_mass: &CpuField<f32>,
    base_full_mass: &CpuField<f32>,
    region: &AcousticMassThetaRegion,
    ranges: &AcousticMassThetaActiveRanges,
) -> AcousticMassThetaResult<()> {
    let width = region.shape().west_east_points();
    let perturbation = column_mass.values();
    let base = base_full_mass.values();
    backend
        .try_for_each_output_block(
            coupled_mass.values_mut(),
            width,
            |south_north, output_row| {
                if ranges.south_north.contains(&south_north) {
                    let row_start = south_north * width;
                    for west_east in ranges.west_east.clone() {
                        let index = row_start + west_east;
                        output_row[west_east] = base[index] + perturbation[index];
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

fn copy_active_horizontal(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    input: &CpuField<f32>,
    region: &AcousticMassThetaRegion,
    ranges: &AcousticMassThetaActiveRanges,
) -> AcousticMassThetaResult<()> {
    let width = region.shape().west_east_points();
    let input_values = input.values();
    backend
        .try_for_each_output_block(output.values_mut(), width, |south_north, output_row| {
            if ranges.south_north.contains(&south_north) {
                let row_start = south_north * width;
                for west_east in ranges.west_east.clone() {
                    output_row[west_east] = input_values[row_start + west_east];
                }
            }
            Ok::<(), Infallible>(())
        })
        .map_err(map_parallel_error)
}
