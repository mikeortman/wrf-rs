use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField};

use crate::{
    AcousticHorizontalMomentumInputs, AcousticHorizontalMomentumParameters,
    AcousticHorizontalMomentumRegion, AcousticHorizontalMomentumResult, AcousticPressureMode,
};

use super::super::domain::AcousticHorizontalMomentumActiveRanges;
use super::{map_parallel_error, pressure_interpolation::PressureInterpolation};

pub(super) fn advance(
    backend: &CpuBackend,
    momentum: &mut CpuField<f32>,
    inputs: AcousticHorizontalMomentumInputs<'_, CpuField<f32>>,
    parameters: AcousticHorizontalMomentumParameters,
    region: &AcousticHorizontalMomentumRegion,
    ranges: &AcousticHorizontalMomentumActiveRanges,
) -> AcousticHorizontalMomentumResult<()> {
    let shape = region.shape();
    let west_east_points = shape.west_east_points();
    let bottom_top_points = shape.bottom_top_points();
    let plane_length = west_east_points * bottom_top_points;
    let tendency = inputs.tendencies.south_north.values();
    let pressure = inputs.pressure;
    let masses = inputs.masses;
    let moisture = inputs.moisture.south_north.values();
    let maps = inputs.map_factors;
    let vertical = inputs.vertical;
    let interpolation =
        PressureInterpolation::new(pressure.pressure_perturbation, vertical, parameters, region);
    let inverse_grid_spacing = parameters.inverse_south_north_grid_spacing;
    let grid_spacing = 1.0 / inverse_grid_spacing;

    backend
        .try_for_each_output_block(
            momentum.values_mut(),
            plane_length,
            |south_north, momentum_plane| {
                let plane_start = south_north * plane_length;
                if ranges
                    .south_north_tendency_south_north
                    .contains(&south_north)
                {
                    for bottom_top in ranges.half_levels.clone() {
                        let row_start = bottom_top * west_east_points;
                        for west_east in ranges.south_north_west_east.clone() {
                            let plane_index = row_start + west_east;
                            let volume_index = plane_start + plane_index;
                            momentum_plane[plane_index] +=
                                parameters.acoustic_time_step * tendency[volume_index];
                        }
                    }
                }
                if ranges
                    .south_north_pressure_south_north
                    .contains(&south_north)
                {
                    for bottom_top in ranges.half_levels.clone() {
                        let row_start = bottom_top * west_east_points;
                        let mass_multiplier = vertical.half_level_mass_multiplier[bottom_top];
                        for west_east in ranges.south_north_west_east.clone() {
                            let plane_index = row_start + west_east;
                            let volume_index = plane_start + plane_index;
                            let south_volume_index = volume_index - plane_length;
                            let horizontal_index = south_north * west_east_points + west_east;
                            let south_horizontal_index = horizontal_index - west_east_points;
                            let map_ratio = maps.south_north_y.values()[horizontal_index]
                                / maps.south_north_x.values()[horizontal_index];
                            let mut pressure_gradient = map_ratio
                                * 0.5
                                * inverse_grid_spacing
                                * (mass_multiplier * masses.south_north.values()[horizontal_index]
                                    + vertical.half_level_mass_offset[bottom_top])
                                * ((pressure.geopotential_perturbation.values()
                                    [volume_index + west_east_points]
                                    - pressure.geopotential_perturbation.values()
                                        [south_volume_index + west_east_points]
                                    + (pressure.geopotential_perturbation.values()[volume_index]
                                        - pressure.geopotential_perturbation.values()
                                            [south_volume_index]))
                                    + (pressure.full_inverse_density.values()[volume_index]
                                        + pressure.full_inverse_density.values()
                                            [south_volume_index])
                                        * (pressure.pressure_perturbation.values()[volume_index]
                                            - pressure.pressure_perturbation.values()
                                                [south_volume_index])
                                    + (pressure.inverse_density_perturbation.values()
                                        [volume_index]
                                        + pressure.inverse_density_perturbation.values()
                                            [south_volume_index])
                                        * (pressure.base_pressure.values()[volume_index]
                                            - pressure.base_pressure.values()[south_volume_index]));
                            if parameters.mode == AcousticPressureMode::Nonhydrostatic {
                                let lower_pressure =
                                    interpolation.south_north(bottom_top, west_east, south_north);
                                let upper_pressure = interpolation.south_north(
                                    bottom_top + 1,
                                    west_east,
                                    south_north,
                                );
                                pressure_gradient += map_ratio
                                    * inverse_grid_spacing
                                    * (pressure.pressure_point_geopotential.values()[volume_index]
                                        - pressure.pressure_point_geopotential.values()
                                            [south_volume_index])
                                    * (vertical.inverse_half_level_spacing[bottom_top]
                                        * (upper_pressure - lower_pressure)
                                        - 0.5
                                            * (mass_multiplier
                                                * masses.perturbation.values()
                                                    [south_horizontal_index]
                                                + mass_multiplier
                                                    * masses.perturbation.values()
                                                        [horizontal_index]));
                            }
                            let damping = -parameters.divergence_damping
                                * grid_spacing
                                * (masses.divergence_damping.values()[horizontal_index]
                                    - masses.divergence_damping.values()[south_horizontal_index])
                                * maps.inverse_south_north_x.values()[horizontal_index];
                            momentum_plane[plane_index] = momentum_plane[plane_index]
                                - parameters.acoustic_time_step
                                    * moisture[volume_index]
                                    * pressure_gradient
                                + mass_multiplier * damping;
                        }
                    }
                }
                if ranges.south_polar_row == Some(south_north)
                    || ranges.north_polar_row == Some(south_north)
                {
                    for bottom_top in ranges.half_levels.clone() {
                        let row_start = bottom_top * west_east_points;
                        for west_east in ranges.south_north_west_east.clone() {
                            momentum_plane[row_start + west_east] = 0.0;
                        }
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}
