use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField};

use crate::{
    AcousticMassThetaInputs, AcousticMassThetaParameters, AcousticMassThetaRegion,
    AcousticMassThetaResult, AcousticMassThetaVerticalCoefficients,
};

use super::super::domain::AcousticMassThetaActiveRanges;
use super::map_parallel_error;

pub(super) fn calculate(
    backend: &CpuBackend,
    divergence_scratch: &mut CpuField<f32>,
    inputs: AcousticMassThetaInputs<'_, CpuField<f32>>,
    parameters: AcousticMassThetaParameters,
    region: &AcousticMassThetaRegion,
    ranges: &AcousticMassThetaActiveRanges,
) -> AcousticMassThetaResult<()> {
    let shape = region.shape();
    let width = shape.west_east_points();
    let levels = shape.bottom_top_points();
    let plane_length = width * levels;
    let momentum = inputs.momentum;
    let mass = inputs.mass;
    let maps = inputs.map_factors;
    let vertical = inputs.vertical;
    backend
        .try_for_each_output_block(
            divergence_scratch.values_mut(),
            plane_length,
            |south_north, output_plane| {
                if !ranges.south_north.contains(&south_north) {
                    return Ok::<(), Infallible>(());
                }
                let plane_start = south_north * plane_length;
                let north_plane_start = plane_start + plane_length;
                for bottom_top in ranges.half_levels.clone() {
                    let row_start = bottom_top * width;
                    let mass_multiplier = vertical.half_level_mass_multiplier[bottom_top];
                    let mass_offset = vertical.half_level_mass_offset[bottom_top];
                    for west_east in ranges.west_east.clone() {
                        let plane_index = row_start + west_east;
                        let volume_index = plane_start + plane_index;
                        let north_volume_index = north_plane_start + plane_index;
                        let east_volume_index = volume_index + 1;
                        let horizontal_index = south_north * width + west_east;
                        let north_horizontal_index = horizontal_index + width;
                        let east_horizontal_index = horizontal_index + 1;
                        let south_north_difference = (momentum.south_north.values()
                            [north_volume_index]
                            + (mass_multiplier
                                * mass.south_north.values()[north_horizontal_index]
                                + mass_offset)
                                * momentum.saved_south_north.values()[north_volume_index]
                                * maps.inverse_south_north_x.values()[north_horizontal_index])
                            - (momentum.south_north.values()[volume_index]
                                + (mass_multiplier * mass.south_north.values()[horizontal_index]
                                    + mass_offset)
                                    * momentum.saved_south_north.values()[volume_index]
                                    * maps.inverse_south_north_x.values()[horizontal_index]);
                        let west_east_difference = (momentum.west_east.values()[east_volume_index]
                            + (mass_multiplier * mass.west_east.values()[east_horizontal_index]
                                + mass_offset)
                                * momentum.saved_west_east.values()[east_volume_index]
                                / maps.west_east_y.values()[east_horizontal_index])
                            - (momentum.west_east.values()[volume_index]
                                + (mass_multiplier * mass.west_east.values()[horizontal_index]
                                    + mass_offset)
                                    * momentum.saved_west_east.values()[volume_index]
                                    / maps.west_east_y.values()[horizontal_index]);
                        output_plane[plane_index] = maps.mass_point_x.values()[horizontal_index]
                            * maps.mass_point_y.values()[horizontal_index]
                            * (parameters.inverse_south_north_grid_spacing
                                * south_north_difference
                                + parameters.inverse_west_east_grid_spacing * west_east_difference);
                    }
                }
                Ok(())
            },
        )
        .map_err(map_parallel_error)
}

pub(super) fn integrate_column_tendency(
    backend: &CpuBackend,
    column_tendency: &mut CpuField<f32>,
    divergence_scratch: &CpuField<f32>,
    vertical: AcousticMassThetaVerticalCoefficients<'_>,
    region: &AcousticMassThetaRegion,
    ranges: &AcousticMassThetaActiveRanges,
) -> AcousticMassThetaResult<()> {
    let shape = region.shape();
    let width = shape.west_east_points();
    let levels = shape.bottom_top_points();
    let divergence = divergence_scratch.values();
    backend
        .try_for_each_output_block(
            column_tendency.values_mut(),
            width,
            |south_north, output_row| {
                if !ranges.south_north.contains(&south_north) {
                    return Ok::<(), Infallible>(());
                }
                let plane_start = south_north * width * levels;
                for west_east in ranges.west_east.clone() {
                    let mut integrated_tendency = 0.0_f32;
                    for bottom_top in ranges.half_levels.clone() {
                        let volume_index = plane_start + bottom_top * width + west_east;
                        integrated_tendency += vertical.half_level_eta_thickness[bottom_top]
                            * divergence[volume_index];
                    }
                    output_row[west_east] = integrated_tendency;
                }
                Ok(())
            },
        )
        .map_err(map_parallel_error)
}
