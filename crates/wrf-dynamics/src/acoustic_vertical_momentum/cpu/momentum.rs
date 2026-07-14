use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField};

use crate::{
    AcousticVerticalDamping, AcousticVerticalInputs, AcousticVerticalParameters,
    AcousticVerticalRegion, AcousticVerticalResult, VerticalAcousticTopBoundary,
};

use super::super::region::AcousticVerticalActiveRanges;
use super::map_parallel_error;

const GRAVITY: f32 = 9.81;

pub(super) struct VerticalMomentumAdvance<'a> {
    inputs: AcousticVerticalInputs<'a, CpuField<f32>>,
    parameters: AcousticVerticalParameters,
    top_boundary: VerticalAcousticTopBoundary,
    damping: AcousticVerticalDamping,
    region: &'a AcousticVerticalRegion,
    ranges: &'a AcousticVerticalActiveRanges,
    right_hand_side: &'a CpuField<f32>,
    perturbation_geopotential: &'a CpuField<f32>,
    time_averaged_thermodynamics: &'a CpuField<f32>,
}

impl<'a> VerticalMomentumAdvance<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(super) const fn new(
        inputs: AcousticVerticalInputs<'a, CpuField<f32>>,
        parameters: AcousticVerticalParameters,
        top_boundary: VerticalAcousticTopBoundary,
        damping: AcousticVerticalDamping,
        region: &'a AcousticVerticalRegion,
        ranges: &'a AcousticVerticalActiveRanges,
        right_hand_side: &'a CpuField<f32>,
        perturbation_geopotential: &'a CpuField<f32>,
        time_averaged_thermodynamics: &'a CpuField<f32>,
    ) -> Self {
        Self {
            inputs,
            parameters,
            top_boundary,
            damping,
            region,
            ranges,
            right_hand_side,
            perturbation_geopotential,
            time_averaged_thermodynamics,
        }
    }

    pub(super) fn advance(
        &self,
        backend: &CpuBackend,
        vertical_momentum: &mut CpuField<f32>,
    ) -> AcousticVerticalResult<()> {
        let shape = self.region.shape();
        let width = shape.west_east_points();
        let levels = shape.bottom_top_points();
        let plane_length = width * levels;
        backend
            .try_for_each_output_block(
                vertical_momentum.values_mut(),
                plane_length,
                |south_north, output_plane| {
                    if !self.ranges.south_north.contains(&south_north) {
                        return Ok::<(), Infallible>(());
                    }
                    self.apply_surface_condition(south_north, width, levels, output_plane);
                    self.add_interior_tendencies(south_north, width, levels, output_plane);
                    self.add_top_tendency(south_north, width, levels, output_plane);
                    self.solve_tridiagonal(south_north, width, levels, output_plane);
                    self.apply_upper_damping(south_north, width, levels, output_plane);
                    Ok(())
                },
            )
            .map_err(map_parallel_error)
    }

    fn apply_surface_condition(
        &self,
        south_north: usize,
        width: usize,
        levels: usize,
        output_plane: &mut [f32],
    ) {
        let surface = self.region.surface_level();
        let [first_weight, second_weight, third_weight] = self.parameters.surface_velocity_weights;
        for west_east in self.ranges.west_east.clone() {
            let horizontal_index = south_north * width + west_east;
            let north_horizontal_index = horizontal_index + width;
            let south_horizontal_index = horizontal_index - width;
            let east_horizontal_index = horizontal_index + 1;
            let west_horizontal_index = horizontal_index - 1;
            let plane_start = south_north * width * levels;
            let north_plane_start = plane_start + width * levels;
            let surface_row = surface * width;
            let south_north_velocity = (self.inputs.maps.terrain_height.values()
                [north_horizontal_index]
                - self.inputs.maps.terrain_height.values()[horizontal_index])
                * (first_weight
                    * self.inputs.momentum.south_north_momentum.values()
                        [north_plane_start + surface_row + west_east]
                    + second_weight
                        * self.inputs.momentum.south_north_momentum.values()
                            [north_plane_start + surface_row + width + west_east]
                    + third_weight
                        * self.inputs.momentum.south_north_momentum.values()
                            [north_plane_start + surface_row + 2 * width + west_east])
                + (self.inputs.maps.terrain_height.values()[horizontal_index]
                    - self.inputs.maps.terrain_height.values()[south_horizontal_index])
                    * (first_weight
                        * self.inputs.momentum.south_north_momentum.values()
                            [plane_start + surface_row + west_east]
                        + second_weight
                            * self.inputs.momentum.south_north_momentum.values()
                                [plane_start + surface_row + width + west_east]
                        + third_weight
                            * self.inputs.momentum.south_north_momentum.values()
                                [plane_start + surface_row + 2 * width + west_east]);
            let west_east_velocity = (self.inputs.maps.terrain_height.values()
                [east_horizontal_index]
                - self.inputs.maps.terrain_height.values()[horizontal_index])
                * (first_weight
                    * self.inputs.momentum.west_east_momentum.values()
                        [plane_start + surface_row + west_east + 1]
                    + second_weight
                        * self.inputs.momentum.west_east_momentum.values()
                            [plane_start + surface_row + width + west_east + 1]
                    + third_weight
                        * self.inputs.momentum.west_east_momentum.values()
                            [plane_start + surface_row + 2 * width + west_east + 1])
                + (self.inputs.maps.terrain_height.values()[horizontal_index]
                    - self.inputs.maps.terrain_height.values()[west_horizontal_index])
                    * (first_weight
                        * self.inputs.momentum.west_east_momentum.values()
                            [plane_start + surface_row + west_east]
                        + second_weight
                            * self.inputs.momentum.west_east_momentum.values()
                                [plane_start + surface_row + width + west_east]
                        + third_weight
                            * self.inputs.momentum.west_east_momentum.values()
                                [plane_start + surface_row + 2 * width + west_east]);
            output_plane[surface_row + west_east] = self.inputs.maps.south_north.values()
                [horizontal_index]
                * 0.5
                * self.parameters.inverse_south_north_grid_spacing
                * south_north_velocity
                + self.inputs.maps.west_east.values()[horizontal_index]
                    * 0.5
                    * self.parameters.inverse_west_east_grid_spacing
                    * west_east_velocity;
        }
    }

    fn add_interior_tendencies(
        &self,
        south_north: usize,
        width: usize,
        levels: usize,
        output_plane: &mut [f32],
    ) {
        let plane_start = south_north * width * levels;
        for full_level in (self.region.surface_level() + 1)..self.region.top_level() {
            let row_start = full_level * width;
            for west_east in self.ranges.west_east.clone() {
                let horizontal_index = south_north * width + west_east;
                let plane_index = row_start + west_east;
                let volume_index = plane_start + plane_index;
                let lower_volume_index = volume_index - width;
                let inverse_map = 1.0 / self.inputs.maps.south_north.values()[horizontal_index];
                let upper_pressure_difference = (1.0 + self.parameters.time_centering)
                    * (self.right_hand_side.values()[volume_index + width]
                        - self.right_hand_side.values()[volume_index])
                    + (1.0 - self.parameters.time_centering)
                        * (self.perturbation_geopotential.values()[volume_index + width]
                            - self.perturbation_geopotential.values()[volume_index]);
                let lower_pressure_difference = (1.0 + self.parameters.time_centering)
                    * (self.right_hand_side.values()[volume_index]
                        - self.right_hand_side.values()[lower_volume_index])
                    + (1.0 - self.parameters.time_centering)
                        * (self.perturbation_geopotential.values()[volume_index]
                            - self.perturbation_geopotential.values()[lower_volume_index]);
                let full_column_mass = self.inputs.mass.full_column_mass.values()[horizontal_index];
                let upper_half_mass = self.inputs.vertical.half_level_mass_multiplier[full_level]
                    * full_column_mass
                    + self.inputs.vertical.half_level_mass_offset[full_level];
                let lower_half_mass = self.inputs.vertical.half_level_mass_multiplier
                    [full_level - 1]
                    * full_column_mass
                    + self.inputs.vertical.half_level_mass_offset[full_level - 1];
                let pressure_force = 0.5
                    * self.parameters.acoustic_time_step
                    * GRAVITY
                    * self.inputs.vertical.inverse_full_level_spacing[full_level]
                    * (self.inputs.solve.pressure_coefficient.values()[volume_index]
                        * self.inputs.vertical.inverse_half_level_spacing[full_level]
                        / upper_half_mass
                        * upper_pressure_difference
                        - self.inputs.solve.pressure_coefficient.values()[lower_volume_index]
                            * self.inputs.vertical.inverse_half_level_spacing[full_level - 1]
                            / lower_half_mass
                            * lower_pressure_difference);
                let buoyancy_force = self.parameters.acoustic_time_step
                    * GRAVITY
                    * inverse_map
                    * (self.inputs.vertical.inverse_full_level_spacing[full_level]
                        * (self.inputs.solve.pressure_coefficient.values()[volume_index]
                            * self.inputs.solve.inverse_density.values()[volume_index]
                            * self.time_averaged_thermodynamics.values()[volume_index]
                            - self.inputs.solve.pressure_coefficient.values()[lower_volume_index]
                                * self.inputs.solve.inverse_density.values()[lower_volume_index]
                                * self.time_averaged_thermodynamics.values()[lower_volume_index])
                        - self.inputs.vertical.full_level_mass_multiplier[full_level]
                            * self.inputs.mass.time_centered_column_mass.values()
                                [horizontal_index]);
                output_plane[plane_index] = output_plane[plane_index]
                    + self.parameters.acoustic_time_step
                        * self.inputs.momentum.tendency.values()[volume_index]
                    + inverse_map
                        * self.inputs.solve.moisture_coefficient.values()[volume_index]
                        * pressure_force
                    + buoyancy_force;
            }
        }
    }

    fn add_top_tendency(
        &self,
        south_north: usize,
        width: usize,
        levels: usize,
        output_plane: &mut [f32],
    ) {
        let top = self.region.top_level();
        let lower_level = top - 1;
        let plane_start = south_north * width * levels;
        for west_east in self.ranges.west_east.clone() {
            let horizontal_index = south_north * width + west_east;
            let plane_index = top * width + west_east;
            let volume_index = plane_start + plane_index;
            let lower_volume_index = volume_index - width;
            let inverse_map = 1.0 / self.inputs.maps.south_north.values()[horizontal_index];
            let full_column_mass = self.inputs.mass.full_column_mass.values()[horizontal_index];
            let lower_half_mass = self.inputs.vertical.half_level_mass_multiplier[lower_level]
                * full_column_mass
                + self.inputs.vertical.half_level_mass_offset[lower_level];
            let pressure_difference = (1.0 + self.parameters.time_centering)
                * (self.right_hand_side.values()[volume_index]
                    - self.right_hand_side.values()[lower_volume_index])
                + (1.0 - self.parameters.time_centering)
                    * (self.perturbation_geopotential.values()[volume_index]
                        - self.perturbation_geopotential.values()[lower_volume_index]);
            let inverse_half_spacing = self.inputs.vertical.inverse_half_level_spacing[lower_level];
            output_plane[plane_index] = output_plane[plane_index]
                + self.parameters.acoustic_time_step
                    * self.inputs.momentum.tendency.values()[volume_index]
                + inverse_map
                    * (-0.5 * self.parameters.acoustic_time_step * GRAVITY / lower_half_mass
                        * (inverse_half_spacing * inverse_half_spacing)
                        * 2.0
                        * self.inputs.solve.pressure_coefficient.values()[lower_volume_index]
                        * pressure_difference
                        - self.parameters.acoustic_time_step
                            * GRAVITY
                            * (2.0
                                * inverse_half_spacing
                                * self.inputs.solve.pressure_coefficient.values()
                                    [lower_volume_index]
                                * self.inputs.solve.inverse_density.values()[lower_volume_index]
                                * self.time_averaged_thermodynamics.values()[lower_volume_index]
                                + self.inputs.vertical.full_level_mass_multiplier[top]
                                    * self.inputs.mass.time_centered_column_mass.values()
                                        [horizontal_index]));
            if self.top_boundary == VerticalAcousticTopBoundary::RigidLid {
                output_plane[plane_index] = 0.0;
            }
        }
    }

    fn solve_tridiagonal(
        &self,
        south_north: usize,
        width: usize,
        levels: usize,
        output_plane: &mut [f32],
    ) {
        let plane_start = south_north * width * levels;
        for full_level in (self.region.surface_level() + 1)..=self.region.top_level() {
            let row_start = full_level * width;
            for west_east in self.ranges.west_east.clone() {
                let plane_index = row_start + west_east;
                let volume_index = plane_start + plane_index;
                output_plane[plane_index] = (output_plane[plane_index]
                    - self.inputs.solve.lower_diagonal.values()[volume_index]
                        * output_plane[plane_index - width])
                    * self.inputs.solve.inverse_eliminated_diagonal.values()[volume_index];
            }
        }
        for full_level in ((self.region.surface_level() + 1)..self.region.top_level()).rev() {
            let row_start = full_level * width;
            for west_east in self.ranges.west_east.clone() {
                let plane_index = row_start + west_east;
                let volume_index = plane_start + plane_index;
                output_plane[plane_index] -= self.inputs.solve.upper_elimination_factor.values()
                    [volume_index]
                    * output_plane[plane_index + width];
            }
        }
    }

    fn apply_upper_damping(
        &self,
        south_north: usize,
        width: usize,
        levels: usize,
        output_plane: &mut [f32],
    ) {
        let AcousticVerticalDamping::UpperLayer { coefficient, depth } = self.damping else {
            return;
        };
        let plane_start = south_north * width * levels;
        let top = self.region.top_level();
        let damping_magnitude = self.parameters.acoustic_time_step * coefficient;
        let pi = 4.0 * 1.0_f32.atan();
        for full_level in ((self.region.surface_level() + 1)..=top).rev() {
            let row_start = full_level * width;
            for west_east in self.ranges.west_east.clone() {
                let horizontal_index = south_north * width + west_east;
                let plane_index = row_start + west_east;
                let volume_index = plane_start + plane_index;
                let top_index = plane_start + top * width + west_east;
                let top_height = (self.inputs.geopotential.saved_perturbation.values()[top_index]
                    + self.inputs.geopotential.base.values()[top_index])
                    / GRAVITY;
                let height = (self.inputs.geopotential.saved_perturbation.values()[volume_index]
                    + self.inputs.geopotential.base.values()[volume_index])
                    / GRAVITY;
                let damping_base = top_height - depth;
                let mut damping_weight = 0.0;
                if height >= damping_base {
                    let sine = (0.5 * pi * (height - damping_base) / depth).sin();
                    damping_weight = damping_magnitude * sine * sine;
                }
                output_plane[plane_index] = (output_plane[plane_index]
                    - damping_weight
                        * (self.inputs.vertical.full_level_mass_multiplier[full_level]
                            * self.inputs.mass.full_column_mass.values()[horizontal_index]
                            + self.inputs.vertical.full_level_mass_offset[full_level])
                        * self.inputs.momentum.saved_vertical_momentum.values()[volume_index])
                    / (1.0 + damping_weight);
            }
        }
    }
}
