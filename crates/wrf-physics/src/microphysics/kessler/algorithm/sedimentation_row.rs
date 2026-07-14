use std::ops::Range;

use crate::{KesslerMicrophysicsParameters, KesslerMicrophysicsResult};

const MAXIMUM_SEDIMENTATION_COURANT_NUMBER: f32 = 0.75;

/// Mutable view of one south-north row during Kessler sedimentation.
pub(super) struct KesslerSedimentationRow<'a> {
    rain_water_mixing_ratio: &'a [f32],
    dry_air_density: &'a [f32],
    height: &'a [f32],
    vertical_layer_thickness: &'a [f32],
    production: &'a mut [f32],
    accumulated_precipitation: &'a mut [f32],
    step_precipitation: &'a mut [f32],
    west_east_points: usize,
}

impl<'a> KesslerSedimentationRow<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        rain_water_mixing_ratio: &'a [f32],
        dry_air_density: &'a [f32],
        height: &'a [f32],
        vertical_layer_thickness: &'a [f32],
        production: &'a mut [f32],
        accumulated_precipitation: &'a mut [f32],
        step_precipitation: &'a mut [f32],
        west_east_points: usize,
    ) -> Self {
        Self {
            rain_water_mixing_ratio,
            dry_air_density,
            height,
            vertical_layer_thickness,
            production,
            accumulated_precipitation,
            step_precipitation,
            west_east_points,
        }
    }

    pub(super) fn apply(
        &mut self,
        west_east_range: Range<usize>,
        bottom_top_range: Range<usize>,
        terminal_velocity: &mut [f32],
        parameters: KesslerMicrophysicsParameters,
    ) -> KesslerMicrophysicsResult<()> {
        for west_east_index in west_east_range {
            self.apply_column(
                west_east_index,
                bottom_top_range.clone(),
                terminal_velocity,
                parameters,
            );
        }
        Ok(())
    }

    fn apply_column(
        &mut self,
        west_east_index: usize,
        bottom_top_range: Range<usize>,
        terminal_velocity: &mut [f32],
        parameters: KesslerMicrophysicsParameters,
    ) {
        let level_count = bottom_top_range.end;
        let mut maximum_courant_number = 0.0_f32;
        let bottom_density = self.value(self.dry_air_density, west_east_index, 0);

        for bottom_top_index in bottom_top_range.clone() {
            let linear_index = self.linear_index(west_east_index, bottom_top_index);
            self.production[linear_index] = self.rain_water_mixing_ratio[linear_index];
            let density = self.dry_air_density[linear_index];
            let rain_mass = (self.production[linear_index] * 0.001_f32 * density).max(0.0);
            let density_correction = (bottom_density / density).sqrt();
            terminal_velocity[bottom_top_index] =
                36.34_f32 * rain_mass.powf(0.1364_f32) * density_correction;
            let courant_number = terminal_velocity[bottom_top_index] * parameters.time_step()
                / self.vertical_layer_thickness[linear_index];
            maximum_courant_number = maximum_courant_number.max(courant_number);
        }

        let mut fallout_step_count = (0.5_f32
            + maximum_courant_number / MAXIMUM_SEDIMENTATION_COURANT_NUMBER)
            .round()
            .max(1.0) as usize;
        let mut fallout_time_step = parameters.time_step() / fallout_step_count as f32;
        let mut remaining_sedimentation_time = parameters.time_step();

        while fallout_step_count > 0 {
            remaining_sedimentation_time -= fallout_time_step;
            self.accumulate_surface_precipitation(
                west_east_index,
                bottom_density,
                terminal_velocity[0],
                fallout_time_step,
                parameters.liquid_water_density(),
            );
            self.advect_fallout(
                west_east_index,
                level_count,
                terminal_velocity,
                fallout_time_step,
            );

            if fallout_step_count == 1 {
                fallout_step_count = 0;
                continue;
            }

            fallout_step_count -= 1;
            maximum_courant_number = self.recalculate_terminal_velocity(
                west_east_index,
                bottom_top_range.clone(),
                bottom_density,
                terminal_velocity,
                remaining_sedimentation_time,
            );
            let revised_step_count = (0.5_f32
                + maximum_courant_number / MAXIMUM_SEDIMENTATION_COURANT_NUMBER)
                .round()
                .max(1.0) as usize;
            if revised_step_count != fallout_step_count {
                fallout_step_count = revised_step_count;
                fallout_time_step = remaining_sedimentation_time / fallout_step_count as f32;
            }
        }
    }

    fn accumulate_surface_precipitation(
        &mut self,
        west_east_index: usize,
        bottom_density: f32,
        bottom_terminal_velocity: f32,
        fallout_time_step: f32,
        liquid_water_density: f32,
    ) {
        let bottom_production = self.production[self.linear_index(west_east_index, 0)];
        let precipitation_depth =
            bottom_density * bottom_production * bottom_terminal_velocity * fallout_time_step
                / liquid_water_density;
        let precipitation_millimeters = precipitation_depth * 1000.0_f32;
        self.step_precipitation[west_east_index] = precipitation_millimeters;
        self.accumulated_precipitation[west_east_index] += precipitation_millimeters;
    }

    fn advect_fallout(
        &mut self,
        west_east_index: usize,
        level_count: usize,
        terminal_velocity: &[f32],
        fallout_time_step: f32,
    ) {
        for bottom_top_index in 0..level_count - 1 {
            let current_index = self.linear_index(west_east_index, bottom_top_index);
            let upper_index = self.linear_index(west_east_index, bottom_top_index + 1);
            let current_density = self.dry_air_density[current_index];
            let upper_density = self.dry_air_density[upper_index];
            let inverse_spacing = 1.0_f32 / (self.height[upper_index] - self.height[current_index]);
            let factor = fallout_time_step * inverse_spacing / current_density;
            let current_flux = current_density
                * self.production[current_index]
                * terminal_velocity[bottom_top_index];
            let upper_flux = upper_density
                * self.production[upper_index]
                * terminal_velocity[bottom_top_index + 1];
            self.production[current_index] -= factor * (current_flux - upper_flux);
        }

        let top_level = level_count - 1;
        let top_index = self.linear_index(west_east_index, top_level);
        let lower_index = self.linear_index(west_east_index, top_level - 1);
        let inverse_spacing = 1.0_f32 / (self.height[top_index] - self.height[lower_index]);
        let factor = fallout_time_step * inverse_spacing;
        self.production[top_index] -=
            factor * self.production[top_index] * terminal_velocity[top_level];
    }

    fn recalculate_terminal_velocity(
        &self,
        west_east_index: usize,
        bottom_top_range: Range<usize>,
        bottom_density: f32,
        terminal_velocity: &mut [f32],
        remaining_sedimentation_time: f32,
    ) -> f32 {
        let mut maximum_courant_number = 0.0_f32;
        for bottom_top_index in bottom_top_range {
            let linear_index = self.linear_index(west_east_index, bottom_top_index);
            let density = self.dry_air_density[linear_index];
            let rain_mass = (self.production[linear_index] * 0.001_f32 * density).max(0.0);
            let density_correction = (bottom_density / density).sqrt();
            terminal_velocity[bottom_top_index] =
                36.34_f32 * rain_mass.powf(0.1364_f32) * density_correction;
            let courant_number = terminal_velocity[bottom_top_index] * remaining_sedimentation_time
                / self.vertical_layer_thickness[linear_index];
            maximum_courant_number = maximum_courant_number.max(courant_number);
        }
        maximum_courant_number
    }

    fn value(&self, values: &[f32], west_east_index: usize, bottom_top_index: usize) -> f32 {
        values[self.linear_index(west_east_index, bottom_top_index)]
    }

    fn linear_index(&self, west_east_index: usize, bottom_top_index: usize) -> usize {
        bottom_top_index * self.west_east_points + west_east_index
    }
}
