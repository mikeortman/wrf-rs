use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField};

use crate::{AcousticMassThetaInputs, AcousticMassThetaRegion, AcousticMassThetaResult};

use super::super::domain::AcousticMassThetaActiveRanges;
use super::map_parallel_error;

pub(super) fn advance(
    backend: &CpuBackend,
    vertical_mass_flux: &mut CpuField<f32>,
    divergence_scratch: &CpuField<f32>,
    integrated_tendency: &CpuField<f32>,
    inputs: AcousticMassThetaInputs<'_, CpuField<f32>>,
    region: &AcousticMassThetaRegion,
    ranges: &AcousticMassThetaActiveRanges,
) -> AcousticMassThetaResult<()> {
    let shape = region.shape();
    let width = shape.west_east_points();
    let levels = shape.bottom_top_points();
    let plane_length = width * levels;
    let divergence = divergence_scratch.values();
    let column_tendency = integrated_tendency.values();
    let large_step_mass_tendency = inputs.mass.tendency.values();
    let saved_vertical_mass_flux = inputs.momentum.saved_vertical_mass_flux.values();
    let mass_point_y_map = inputs.map_factors.mass_point_y.values();
    let vertical = inputs.vertical;
    backend
        .try_for_each_output_block(
            vertical_mass_flux.values_mut(),
            plane_length,
            |south_north, output_plane| {
                if !ranges.south_north.contains(&south_north) {
                    return Ok::<(), Infallible>(());
                }
                let plane_start = south_north * plane_length;
                let horizontal_row_start = south_north * width;
                for full_level in (ranges.half_levels.start + 1)..ranges.half_levels.end {
                    let half_level = full_level - 1;
                    let output_row_start = full_level * width;
                    let lower_row_start = half_level * width;
                    for west_east in ranges.west_east.clone() {
                        let horizontal_index = horizontal_row_start + west_east;
                        let divergence_index = plane_start + half_level * width + west_east;
                        output_plane[output_row_start + west_east] = output_plane
                            [lower_row_start + west_east]
                            - vertical.half_level_eta_thickness[half_level]
                                * ((vertical.half_level_mass_multiplier[half_level]
                                    * column_tendency[horizontal_index])
                                    + divergence[divergence_index]
                                    + (vertical.half_level_mass_multiplier[half_level]
                                        * large_step_mass_tendency[horizontal_index]))
                                / mass_point_y_map[horizontal_index];
                    }
                }
                for half_level in ranges.half_levels.clone() {
                    let row_start = half_level * width;
                    for west_east in ranges.west_east.clone() {
                        let plane_index = row_start + west_east;
                        output_plane[plane_index] -=
                            saved_vertical_mass_flux[plane_start + plane_index];
                    }
                }
                Ok(())
            },
        )
        .map_err(map_parallel_error)
}
