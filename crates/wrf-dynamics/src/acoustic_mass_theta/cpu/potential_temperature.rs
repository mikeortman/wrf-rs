use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField};

use crate::{
    AcousticMassThetaInputs, AcousticMassThetaParameters, AcousticMassThetaRegion,
    AcousticMassThetaResult,
};

use super::super::domain::AcousticMassThetaActiveRanges;
use super::map_parallel_error;

pub(super) fn add_tendency(
    backend: &CpuBackend,
    potential_temperature: &mut CpuField<f32>,
    previous_potential_temperature: &mut CpuField<f32>,
    inputs: AcousticMassThetaInputs<'_, CpuField<f32>>,
    parameters: AcousticMassThetaParameters,
    region: &AcousticMassThetaRegion,
    ranges: &AcousticMassThetaActiveRanges,
) -> AcousticMassThetaResult<()> {
    let shape = region.shape();
    let width = shape.west_east_points();
    let plane_length = width * shape.bottom_top_points();
    let tendency = inputs
        .thermodynamics
        .potential_temperature_tendency
        .values();
    let mass_point_y_map = inputs.map_factors.mass_point_y.values();
    backend
        .try_for_each_output_pair_block(
            potential_temperature.values_mut(),
            previous_potential_temperature.values_mut(),
            plane_length,
            |south_north, temperature_plane, previous_plane| {
                if !ranges.south_north.contains(&south_north) {
                    return Ok::<(), Infallible>(());
                }
                let plane_start = south_north * plane_length;
                let horizontal_row_start = south_north * width;
                for bottom_top in ranges.half_levels.clone() {
                    let row_start = bottom_top * width;
                    for west_east in ranges.west_east.clone() {
                        let plane_index = row_start + west_east;
                        let volume_index = plane_start + plane_index;
                        previous_plane[plane_index] = temperature_plane[plane_index];
                        temperature_plane[plane_index] += mass_point_y_map
                            [horizontal_row_start + west_east]
                            * parameters.acoustic_time_step
                            * tendency[volume_index];
                    }
                }
                Ok(())
            },
        )
        .map_err(map_parallel_error)
}

pub(super) fn transport(
    backend: &CpuBackend,
    potential_temperature: &mut CpuField<f32>,
    vertical_mass_flux: &CpuField<f32>,
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
    let saved_temperature = inputs.thermodynamics.saved_potential_temperature.values();
    let vertical_flux = vertical_mass_flux.values();
    let maps = inputs.map_factors;
    let vertical = inputs.vertical;
    backend
        .try_for_each_output_block(
            potential_temperature.values_mut(),
            plane_length,
            |south_north, output_plane| {
                if !ranges.south_north.contains(&south_north) {
                    return Ok::<(), Infallible>(());
                }
                let plane_start = south_north * plane_length;
                let north_plane_start = plane_start + plane_length;
                let south_plane_start = plane_start - plane_length;
                let horizontal_row_start = south_north * width;
                for bottom_top in ranges.half_levels.clone() {
                    let row_start = bottom_top * width;
                    for west_east in ranges.west_east.clone() {
                        let plane_index = row_start + west_east;
                        let volume_index = plane_start + plane_index;
                        let north_volume_index = north_plane_start + plane_index;
                        let south_volume_index = south_plane_start + plane_index;
                        let east_volume_index = volume_index + 1;
                        let west_volume_index = volume_index - 1;
                        let horizontal_index = horizontal_row_start + west_east;
                        let south_north_transport = momentum.south_north.values()
                            [north_volume_index]
                            * (saved_temperature[north_volume_index]
                                + saved_temperature[volume_index])
                            - momentum.south_north.values()[volume_index]
                                * (saved_temperature[volume_index]
                                    + saved_temperature[south_volume_index]);
                        let west_east_transport = momentum.west_east.values()[east_volume_index]
                            * (saved_temperature[east_volume_index]
                                + saved_temperature[volume_index])
                            - momentum.west_east.values()[volume_index]
                                * (saved_temperature[volume_index]
                                    + saved_temperature[west_volume_index]);
                        let lower_vertical_transport = interpolated_vertical_transport(
                            bottom_top,
                            west_east,
                            south_north,
                            vertical_flux,
                            saved_temperature,
                            vertical,
                            shape,
                            &ranges.half_levels,
                        );
                        let upper_vertical_transport = interpolated_vertical_transport(
                            bottom_top + 1,
                            west_east,
                            south_north,
                            vertical_flux,
                            saved_temperature,
                            vertical,
                            shape,
                            &ranges.half_levels,
                        );
                        output_plane[plane_index] -= parameters.acoustic_time_step
                            * maps.mass_point_y.values()[horizontal_index]
                            * (maps.mass_point_x.values()[horizontal_index]
                                * (0.5
                                    * parameters.inverse_south_north_grid_spacing
                                    * south_north_transport
                                    + 0.5
                                        * parameters.inverse_west_east_grid_spacing
                                        * west_east_transport)
                                + vertical.inverse_half_level_spacing[bottom_top]
                                    * (upper_vertical_transport - lower_vertical_transport));
                    }
                }
                Ok(())
            },
        )
        .map_err(map_parallel_error)
}

#[allow(clippy::too_many_arguments)]
fn interpolated_vertical_transport(
    full_level: usize,
    west_east: usize,
    south_north: usize,
    vertical_mass_flux: &[f32],
    saved_temperature: &[f32],
    vertical: crate::AcousticMassThetaVerticalCoefficients<'_>,
    shape: wrf_compute::GridShape,
    half_levels: &std::ops::Range<usize>,
) -> f32 {
    if full_level == half_levels.start || full_level == half_levels.end {
        return 0.0;
    }
    let width = shape.west_east_points();
    let levels = shape.bottom_top_points();
    let full_index = (south_north * levels + full_level) * width + west_east;
    let lower_index = full_index - width;
    vertical_mass_flux[full_index]
        * (vertical.lower_interpolation_weight[full_level] * saved_temperature[full_index]
            + vertical.upper_interpolation_weight[full_level] * saved_temperature[lower_index])
}
