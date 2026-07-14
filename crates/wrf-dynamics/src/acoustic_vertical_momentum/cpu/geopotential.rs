use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField};

use crate::{
    AcousticVerticalInputs, AcousticVerticalParameters, AcousticVerticalRegion,
    AcousticVerticalResult,
};

use super::super::region::AcousticVerticalActiveRanges;
use super::map_parallel_error;

const GRAVITY: f32 = 9.81;

pub(super) struct GeopotentialUpdate<'a> {
    inputs: AcousticVerticalInputs<'a, CpuField<f32>>,
    parameters: AcousticVerticalParameters,
    region: &'a AcousticVerticalRegion,
    ranges: &'a AcousticVerticalActiveRanges,
    right_hand_side: &'a CpuField<f32>,
    vertical_momentum: &'a CpuField<f32>,
}

impl<'a> GeopotentialUpdate<'a> {
    pub(super) const fn new(
        inputs: AcousticVerticalInputs<'a, CpuField<f32>>,
        parameters: AcousticVerticalParameters,
        region: &'a AcousticVerticalRegion,
        ranges: &'a AcousticVerticalActiveRanges,
        right_hand_side: &'a CpuField<f32>,
        vertical_momentum: &'a CpuField<f32>,
    ) -> Self {
        Self {
            inputs,
            parameters,
            region,
            ranges,
            right_hand_side,
            vertical_momentum,
        }
    }

    pub(super) fn update(
        &self,
        backend: &CpuBackend,
        perturbation_geopotential: &mut CpuField<f32>,
    ) -> AcousticVerticalResult<()> {
        let shape = self.region.shape();
        let width = shape.west_east_points();
        let levels = shape.bottom_top_points();
        let plane_length = width * levels;
        backend
            .try_for_each_output_block(
                perturbation_geopotential.values_mut(),
                plane_length,
                |south_north, output_plane| {
                    if !self.ranges.south_north.contains(&south_north) {
                        return Ok::<(), Infallible>(());
                    }
                    let plane_start = south_north * plane_length;
                    for full_level in
                        ((self.region.surface_level() + 1)..=self.region.top_level()).rev()
                    {
                        let row_start = full_level * width;
                        for west_east in self.ranges.west_east.clone() {
                            let horizontal_index = south_north * width + west_east;
                            let plane_index = row_start + west_east;
                            let volume_index = plane_start + plane_index;
                            output_plane[plane_index] = self.right_hand_side.values()[volume_index]
                                + self.inputs.maps.south_north.values()[horizontal_index]
                                    * 0.5
                                    * self.parameters.acoustic_time_step
                                    * GRAVITY
                                    * (1.0 + self.parameters.time_centering)
                                    * self.vertical_momentum.values()[volume_index]
                                    / (self.inputs.vertical.full_level_mass_multiplier[full_level]
                                        * self.inputs.mass.coupled_column_mass.values()
                                            [horizontal_index]
                                        + self.inputs.vertical.full_level_mass_offset[full_level]);
                        }
                    }
                    Ok(())
                },
            )
            .map_err(map_parallel_error)
    }
}
