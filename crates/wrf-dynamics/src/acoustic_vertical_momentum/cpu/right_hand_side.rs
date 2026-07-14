use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField};

use crate::{
    AcousticVerticalAdvection, AcousticVerticalInputs, AcousticVerticalParameters,
    AcousticVerticalRegion, AcousticVerticalResult, VerticalAcousticTopBoundary,
};

use super::super::region::AcousticVerticalActiveRanges;
use super::map_parallel_error;

const GRAVITY: f32 = 9.81;

pub(super) struct RightHandSideCalculation<'a> {
    inputs: AcousticVerticalInputs<'a, CpuField<f32>>,
    parameters: AcousticVerticalParameters,
    top_boundary: VerticalAcousticTopBoundary,
    vertical_advection: AcousticVerticalAdvection,
    region: &'a AcousticVerticalRegion,
    ranges: &'a AcousticVerticalActiveRanges,
}

impl<'a> RightHandSideCalculation<'a> {
    pub(super) const fn new(
        inputs: AcousticVerticalInputs<'a, CpuField<f32>>,
        parameters: AcousticVerticalParameters,
        top_boundary: VerticalAcousticTopBoundary,
        vertical_advection: AcousticVerticalAdvection,
        region: &'a AcousticVerticalRegion,
        ranges: &'a AcousticVerticalActiveRanges,
    ) -> Self {
        Self {
            inputs,
            parameters,
            top_boundary,
            vertical_advection,
            region,
            ranges,
        }
    }

    pub(super) fn calculate(
        &self,
        backend: &CpuBackend,
        time_averaged_thermodynamics: &mut CpuField<f32>,
        right_hand_side: &mut CpuField<f32>,
        perturbation_geopotential: &CpuField<f32>,
        vertical_momentum: &CpuField<f32>,
    ) -> AcousticVerticalResult<()> {
        let shape = self.region.shape();
        let width = shape.west_east_points();
        let levels = shape.bottom_top_points();
        let plane_length = width * levels;
        backend
            .try_for_each_output_pair_block(
                time_averaged_thermodynamics.values_mut(),
                right_hand_side.values_mut(),
                plane_length,
                |south_north, thermodynamics_plane, right_hand_side_plane| {
                    if !self.ranges.south_north.contains(&south_north) {
                        return Ok::<(), Infallible>(());
                    }
                    self.initialize_plane(
                        south_north,
                        width,
                        levels,
                        thermodynamics_plane,
                        right_hand_side_plane,
                        vertical_momentum,
                    );
                    self.apply_vertical_advection(
                        south_north,
                        width,
                        levels,
                        right_hand_side_plane,
                    );
                    self.couple_geopotential(
                        south_north,
                        width,
                        levels,
                        right_hand_side_plane,
                        perturbation_geopotential,
                    );
                    Ok(())
                },
            )
            .map_err(map_parallel_error)
    }

    fn initialize_plane(
        &self,
        south_north: usize,
        width: usize,
        levels: usize,
        thermodynamics_plane: &mut [f32],
        right_hand_side_plane: &mut [f32],
        vertical_momentum: &CpuField<f32>,
    ) {
        let surface_level = self.region.surface_level();
        let plane_start = south_north * width * levels;
        for west_east in self.ranges.west_east.clone() {
            right_hand_side_plane[surface_level * width + west_east] = 0.0;
        }
        for mass_level in self.ranges.mass_levels.clone() {
            let row_start = mass_level * width;
            let upper_row_start = row_start + width;
            for west_east in self.ranges.west_east.clone() {
                let horizontal_index = south_north * width + west_east;
                let plane_index = row_start + west_east;
                let volume_index = plane_start + plane_index;
                let upper_volume_index = volume_index + width;
                let centered = 0.5
                    * ((1.0 + self.parameters.time_centering)
                        * self.inputs.thermodynamics.potential_temperature.values()[volume_index]
                        + (1.0 - self.parameters.time_centering)
                            * thermodynamics_plane[plane_index]);
                thermodynamics_plane[plane_index] = (centered
                    + (self.inputs.vertical.half_level_mass_multiplier[mass_level]
                        * self.inputs.mass.time_centered_column_mass.values()[horizontal_index])
                        * self.parameters.base_potential_temperature)
                    / ((self.inputs.vertical.half_level_mass_multiplier[mass_level]
                        * self.inputs.mass.coupled_column_mass.values()[horizontal_index]
                        + self.inputs.vertical.half_level_mass_offset[mass_level])
                        * (self.parameters.base_potential_temperature
                            + self
                                .inputs
                                .thermodynamics
                                .saved_potential_temperature
                                .values()[volume_index]));
                right_hand_side_plane[upper_row_start + west_east] =
                    self.parameters.acoustic_time_step
                        * (self.inputs.geopotential.tendency.values()[upper_volume_index]
                            + 0.5
                                * GRAVITY
                                * (1.0 - self.parameters.time_centering)
                                * vertical_momentum.values()[upper_volume_index]);
            }
        }
    }

    fn apply_vertical_advection(
        &self,
        south_north: usize,
        width: usize,
        levels: usize,
        right_hand_side_plane: &mut [f32],
    ) {
        let plane_start = south_north * width * levels;
        let interior_levels = (self.region.surface_level() + 1)..self.region.top_level();
        for full_level in interior_levels {
            let row_start = full_level * width;
            for west_east in self.ranges.west_east.clone() {
                let plane_index = row_start + west_east;
                let volume_index = plane_start + plane_index;
                let upper_transport = self.vertical_transport(volume_index, full_level, width);
                let lower_transport =
                    self.vertical_transport(volume_index - width, full_level - 1, width);
                match self.vertical_advection {
                    AcousticVerticalAdvection::StaggeredGeopotentialGradient => {
                        right_hand_side_plane[plane_index] -= self.parameters.acoustic_time_step
                            * self.inputs.momentum.vertical_mass_flux.values()[volume_index]
                            * (self.inputs.vertical.upper_interpolation_weight[full_level]
                                * upper_transport
                                + self.inputs.vertical.lower_interpolation_weight[full_level]
                                    * lower_transport);
                    }
                    AcousticVerticalAdvection::StaggeredTransportProduct => {
                        right_hand_side_plane[plane_index] -= self.parameters.acoustic_time_step
                            * (self.inputs.vertical.upper_interpolation_weight[full_level]
                                * upper_transport
                                + self.inputs.vertical.lower_interpolation_weight[full_level]
                                    * lower_transport);
                    }
                }
            }
        }
    }

    fn vertical_transport(&self, lower_index: usize, mass_level: usize, width: usize) -> f32 {
        let upper_index = lower_index + width;
        let geopotential_difference = self.inputs.geopotential.saved_perturbation.values()
            [upper_index]
            - self.inputs.geopotential.saved_perturbation.values()[lower_index]
            + self.inputs.geopotential.base.values()[upper_index]
            - self.inputs.geopotential.base.values()[lower_index];
        match self.vertical_advection {
            AcousticVerticalAdvection::StaggeredGeopotentialGradient => {
                self.inputs.vertical.inverse_half_level_spacing[mass_level]
                    * geopotential_difference
            }
            AcousticVerticalAdvection::StaggeredTransportProduct => {
                0.5 * (self.inputs.momentum.vertical_mass_flux.values()[upper_index]
                    + self.inputs.momentum.vertical_mass_flux.values()[lower_index])
                    * self.inputs.vertical.inverse_half_level_spacing[mass_level]
                    * geopotential_difference
            }
        }
    }

    fn couple_geopotential(
        &self,
        south_north: usize,
        width: usize,
        levels: usize,
        right_hand_side_plane: &mut [f32],
        perturbation_geopotential: &CpuField<f32>,
    ) {
        let plane_start = south_north * width * levels;
        let top_level = self.region.top_level();
        for full_level in (self.region.surface_level() + 1)..=top_level {
            let row_start = full_level * width;
            for west_east in self.ranges.west_east.clone() {
                let horizontal_index = south_north * width + west_east;
                let plane_index = row_start + west_east;
                let volume_index = plane_start + plane_index;
                right_hand_side_plane[plane_index] = perturbation_geopotential.values()
                    [volume_index]
                    + self.inputs.maps.south_north.values()[horizontal_index]
                        * right_hand_side_plane[plane_index]
                        / (self.inputs.vertical.full_level_mass_multiplier[full_level]
                            * self.inputs.mass.full_column_mass.values()[horizontal_index]
                            + self.inputs.vertical.full_level_mass_offset[full_level]);
                if self.top_boundary == VerticalAcousticTopBoundary::RigidLid
                    && full_level == top_level
                {
                    right_hand_side_plane[plane_index] = 0.0;
                }
            }
        }
    }
}
