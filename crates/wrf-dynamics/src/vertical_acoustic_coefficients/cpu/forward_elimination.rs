use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField};

use crate::{
    VerticalAcousticCoefficientInputs, VerticalAcousticCoefficientParameters,
    VerticalAcousticCoefficientRegion, VerticalAcousticCoefficientResult,
    VerticalAcousticMassCoefficients, VerticalAcousticMetrics,
};

use super::map_parallel_error;

pub(super) struct ForwardElimination<'a> {
    lower_diagonal: &'a CpuField<f32>,
    inputs: VerticalAcousticCoefficientInputs<'a, CpuField<f32>>,
    mass_coefficients: VerticalAcousticMassCoefficients<'a>,
    metrics: VerticalAcousticMetrics<'a>,
    parameters: VerticalAcousticCoefficientParameters,
    region: &'a VerticalAcousticCoefficientRegion,
}

impl<'a> ForwardElimination<'a> {
    pub(super) const fn new(
        lower_diagonal: &'a CpuField<f32>,
        inputs: VerticalAcousticCoefficientInputs<'a, CpuField<f32>>,
        mass_coefficients: VerticalAcousticMassCoefficients<'a>,
        metrics: VerticalAcousticMetrics<'a>,
        parameters: VerticalAcousticCoefficientParameters,
        region: &'a VerticalAcousticCoefficientRegion,
    ) -> Self {
        Self {
            lower_diagonal,
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
        inverse_diagonal: &mut CpuField<f32>,
        upper_factor: &mut CpuField<f32>,
    ) -> VerticalAcousticCoefficientResult<()> {
        let shape = self.region.shape();
        let west_east_points = shape.west_east_points();
        let plane_length = west_east_points * shape.bottom_top_points();
        let active_west_east = self.region.active_west_east();
        let active_south_north = self.region.active_south_north();
        let lower_full_level = self.region.lower_full_level();
        let top_full_level = self.region.top_full_level();
        let squared_factor = self.parameters.squared_time_gravity_factor();
        backend
            .try_for_each_output_pair_block(
                inverse_diagonal.values_mut(),
                upper_factor.values_mut(),
                plane_length,
                |south_north_index,
                 inverse_diagonal_plane,
                 upper_factor_plane|
                 -> Result<(), Infallible> {
                    if !active_south_north.contains(&south_north_index) {
                        return Ok(());
                    }
                    for west_east_index in active_west_east.clone() {
                        upper_factor_plane[lower_full_level * west_east_points + west_east_index] =
                            0.0;
                    }
                    for full_level in (lower_full_level + 1)..top_full_level {
                        for west_east_index in active_west_east.clone() {
                            let horizontal_index =
                                south_north_index * west_east_points + west_east_index;
                            let full_column_mass =
                                self.inputs.full_column_mass.values()[horizontal_index];
                            self.eliminate_interior_level(
                                inverse_diagonal_plane,
                                upper_factor_plane,
                                west_east_index,
                                west_east_points,
                                full_level,
                                full_column_mass,
                                squared_factor,
                                south_north_index,
                            );
                        }
                    }
                    for west_east_index in active_west_east.clone() {
                        let horizontal_index =
                            south_north_index * west_east_points + west_east_index;
                        let full_column_mass =
                            self.inputs.full_column_mass.values()[horizontal_index];
                        self.eliminate_top_level(
                            inverse_diagonal_plane,
                            upper_factor_plane,
                            west_east_index,
                            west_east_points,
                            top_full_level,
                            full_column_mass,
                            squared_factor,
                            south_north_index,
                        );
                    }
                    Ok(())
                },
            )
            .map_err(map_parallel_error)
    }

    #[allow(clippy::too_many_arguments)]
    fn eliminate_interior_level(
        &self,
        inverse_diagonal_plane: &mut [f32],
        upper_factor_plane: &mut [f32],
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
        let current_half_mass = self.mass_coefficients.half_level_multiplier[full_level]
            * full_column_mass
            + self.mass_coefficients.half_level_offset[full_level];
        let lower_half_mass = self.mass_coefficients.half_level_multiplier[lower_half_level]
            * full_column_mass
            + self.mass_coefficients.half_level_offset[lower_half_level];
        let current_full_mass = self.mass_coefficients.full_level_multiplier[full_level]
            * full_column_mass
            + self.mass_coefficients.full_level_offset[full_level];
        let upper_full_mass = self.mass_coefficients.full_level_multiplier[full_level + 1]
            * full_column_mass
            + self.mass_coefficients.full_level_offset[full_level + 1];
        let moisture = self.inputs.vertical_moisture_coefficient.values()[full_index];
        let inverse_full_spacing = self.metrics.inverse_full_level_spacing[full_level];
        let current_pressure_term = self.metrics.inverse_half_level_spacing[full_level]
            * self.inputs.pressure_coefficient.values()[full_index]
            / (current_half_mass * current_full_mass);
        let lower_pressure_term = self.metrics.inverse_half_level_spacing[lower_half_level]
            * self.inputs.pressure_coefficient.values()[lower_half_index]
            / (lower_half_mass * current_full_mass);
        let diagonal = 1.0_f32
            + moisture
                * squared_factor
                * inverse_full_spacing
                * (current_pressure_term + lower_pressure_term);
        let upper = -moisture
            * squared_factor
            * inverse_full_spacing
            * self.metrics.inverse_half_level_spacing[full_level]
            * self.inputs.pressure_coefficient.values()[full_index]
            / (current_half_mass * upper_full_mass);
        let plane_index = full_level * west_east_points + west_east_index;
        let previous_plane_index = plane_index - west_east_points;
        let inverse = 1.0_f32
            / (diagonal
                - self.lower_diagonal.values()[full_index]
                    * upper_factor_plane[previous_plane_index]);
        inverse_diagonal_plane[plane_index] = inverse;
        upper_factor_plane[plane_index] = upper * inverse;
    }

    #[allow(clippy::too_many_arguments)]
    fn eliminate_top_level(
        &self,
        inverse_diagonal_plane: &mut [f32],
        upper_factor_plane: &mut [f32],
        west_east_index: usize,
        west_east_points: usize,
        top_full_level: usize,
        full_column_mass: f32,
        squared_factor: f32,
        south_north_index: usize,
    ) {
        let upper_half_level = top_full_level - 1;
        let upper_half_index = (south_north_index * self.region.shape().bottom_top_points()
            + upper_half_level)
            * west_east_points
            + west_east_index;
        let top_index = upper_half_index + west_east_points;
        let half_mass = self.mass_coefficients.half_level_multiplier[upper_half_level]
            * full_column_mass
            + self.mass_coefficients.half_level_offset[upper_half_level];
        let full_mass = self.mass_coefficients.full_level_multiplier[top_full_level]
            * full_column_mass
            + self.mass_coefficients.full_level_offset[top_full_level];
        let inverse_spacing = self.metrics.inverse_half_level_spacing[upper_half_level];
        let diagonal = 1.0_f32
            + 2.0_f32
                * squared_factor
                * (inverse_spacing * inverse_spacing)
                * self.inputs.pressure_coefficient.values()[upper_half_index]
                / (half_mass * full_mass);
        let plane_index = top_full_level * west_east_points + west_east_index;
        let previous_plane_index = plane_index - west_east_points;
        let inverse = 1.0_f32
            / (diagonal
                - self.lower_diagonal.values()[top_index]
                    * upper_factor_plane[previous_plane_index]);
        inverse_diagonal_plane[plane_index] = inverse;
        upper_factor_plane[plane_index] = 0.0_f32 * inverse;
    }
}
