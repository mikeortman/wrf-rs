use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField};

use crate::{
    VerticalAcousticCoefficientInputs, VerticalAcousticCoefficientParameters,
    VerticalAcousticCoefficientRegion, VerticalAcousticCoefficientResult,
    VerticalAcousticMassCoefficients, VerticalAcousticMetrics,
};

use super::map_parallel_error;

pub(super) struct LowerDiagonalCalculation<'a> {
    inputs: VerticalAcousticCoefficientInputs<'a, CpuField<f32>>,
    mass_coefficients: VerticalAcousticMassCoefficients<'a>,
    metrics: VerticalAcousticMetrics<'a>,
    parameters: VerticalAcousticCoefficientParameters,
    region: &'a VerticalAcousticCoefficientRegion,
}

impl<'a> LowerDiagonalCalculation<'a> {
    pub(super) const fn new(
        inputs: VerticalAcousticCoefficientInputs<'a, CpuField<f32>>,
        mass_coefficients: VerticalAcousticMassCoefficients<'a>,
        metrics: VerticalAcousticMetrics<'a>,
        parameters: VerticalAcousticCoefficientParameters,
        region: &'a VerticalAcousticCoefficientRegion,
    ) -> Self {
        Self {
            inputs,
            mass_coefficients,
            metrics,
            parameters,
            region,
        }
    }

    pub(super) fn calculate(
        self,
        backend: &CpuBackend,
        lower_diagonal: &mut CpuField<f32>,
    ) -> VerticalAcousticCoefficientResult<()> {
        let shape = self.region.shape();
        let west_east_points = shape.west_east_points();
        let plane_length = west_east_points * shape.bottom_top_points();
        let active_west_east = self.region.active_west_east();
        let active_south_north = self.region.active_south_north();
        let lower_full_level = self.region.lower_full_level();
        let top_full_level = self.region.top_full_level();
        let squared_factor = self.parameters.squared_time_gravity_factor();
        let top_multiplier = self.parameters.top_boundary.lower_diagonal_multiplier();
        backend
            .try_for_each_output_block(
                lower_diagonal.values_mut(),
                plane_length,
                |south_north_index, lower_plane| -> Result<(), Infallible> {
                    if !active_south_north.contains(&south_north_index) {
                        return Ok(());
                    }
                    for west_east_index in active_west_east.clone() {
                        let horizontal_index =
                            south_north_index * west_east_points + west_east_index;
                        let full_column_mass =
                            self.inputs.full_column_mass.values()[horizontal_index];
                        let second_full_level = lower_full_level + 1;
                        lower_plane[second_full_level * west_east_points + west_east_index] = 0.0;
                        self.calculate_top_boundary(
                            lower_plane,
                            west_east_index,
                            west_east_points,
                            top_full_level,
                            full_column_mass,
                            squared_factor,
                            top_multiplier,
                            south_north_index,
                        );
                    }
                    for full_level in (lower_full_level + 2)..top_full_level {
                        for west_east_index in active_west_east.clone() {
                            let horizontal_index =
                                south_north_index * west_east_points + west_east_index;
                            let full_column_mass =
                                self.inputs.full_column_mass.values()[horizontal_index];
                            self.calculate_interior(
                                lower_plane,
                                west_east_index,
                                west_east_points,
                                full_level,
                                full_column_mass,
                                squared_factor,
                                south_north_index,
                            );
                        }
                    }
                    Ok(())
                },
            )
            .map_err(map_parallel_error)
    }

    #[allow(clippy::too_many_arguments)]
    fn calculate_top_boundary(
        &self,
        lower_plane: &mut [f32],
        west_east_index: usize,
        west_east_points: usize,
        top_full_level: usize,
        full_column_mass: f32,
        squared_factor: f32,
        top_multiplier: f32,
        south_north_index: usize,
    ) {
        let upper_half_level = top_full_level - 1;
        let pressure_index = (south_north_index * self.region.shape().bottom_top_points()
            + upper_half_level)
            * west_east_points
            + west_east_index;
        let half_mass = self.mass_coefficients.half_level_multiplier[upper_half_level]
            * full_column_mass
            + self.mass_coefficients.half_level_offset[upper_half_level];
        let full_mass = self.mass_coefficients.full_level_multiplier[upper_half_level]
            * full_column_mass
            + self.mass_coefficients.full_level_offset[upper_half_level];
        let inverse_spacing = self.metrics.inverse_half_level_spacing[upper_half_level];
        lower_plane[top_full_level * west_east_points + west_east_index] = -2.0_f32
            * squared_factor
            * (inverse_spacing * inverse_spacing)
            * self.inputs.pressure_coefficient.values()[pressure_index]
            * top_multiplier
            / (half_mass * full_mass);
    }

    #[allow(clippy::too_many_arguments)]
    fn calculate_interior(
        &self,
        lower_plane: &mut [f32],
        west_east_index: usize,
        west_east_points: usize,
        full_level: usize,
        full_column_mass: f32,
        squared_factor: f32,
        south_north_index: usize,
    ) {
        let lower_half_level = full_level - 1;
        let full_index = (south_north_index * self.region.shape().bottom_top_points() + full_level)
            * west_east_points
            + west_east_index;
        let lower_half_index = full_index - west_east_points;
        let half_mass = self.mass_coefficients.half_level_multiplier[lower_half_level]
            * full_column_mass
            + self.mass_coefficients.half_level_offset[lower_half_level];
        let full_mass = self.mass_coefficients.full_level_multiplier[lower_half_level]
            * full_column_mass
            + self.mass_coefficients.full_level_offset[lower_half_level];
        lower_plane[full_level * west_east_points + west_east_index] =
            -self.inputs.vertical_moisture_coefficient.values()[full_index]
                * squared_factor
                * self.metrics.inverse_full_level_spacing[full_level]
                * self.metrics.inverse_half_level_spacing[lower_half_level]
                * self.inputs.pressure_coefficient.values()[lower_half_index]
                / (half_mass * full_mass);
    }
}
