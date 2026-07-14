use std::ops::Range;

use wrf_compute::{CpuField, GridShape};

use super::super::{
    SpecifiedBoundaryFinalizationBoundaryFields, SpecifiedBoundaryFinalizationFieldLocation,
    SpecifiedBoundaryFinalizationInputs, SpecifiedBoundaryFinalizationParameters,
};
use super::boundary_side::SpecifiedBoundaryFinalizationSide;
use crate::specified_boundary_update::geometry::SpecifiedBoundaryRanges;

pub(super) struct SpecifiedBoundaryFinalizationCpuUpdateContext<'a> {
    column_mass: &'a [f32],
    map_factor: &'a [f32],
    column_mass_multiplier: &'a [f32],
    column_mass_offset: &'a [f32],
    boundary_values: SpecifiedBoundaryFinalizationBoundaryFields<'a, CpuField<f32>>,
    boundary_tendencies: SpecifiedBoundaryFinalizationBoundaryFields<'a, CpuField<f32>>,
    west_east_points: usize,
    south_north_points: usize,
    west_east_domain: Range<usize>,
    south_north_domain: Range<usize>,
    bottom_top_range: Range<usize>,
    half_level_domain_start: usize,
    boundary_vertical_points: usize,
    time_step: f32,
    uses_column_mass: bool,
    uses_map_factor: bool,
}

impl<'a> SpecifiedBoundaryFinalizationCpuUpdateContext<'a> {
    pub(super) fn new(
        inputs: &SpecifiedBoundaryFinalizationInputs<'a, CpuField<f32>>,
        parameters: SpecifiedBoundaryFinalizationParameters,
        location: SpecifiedBoundaryFinalizationFieldLocation,
        shape: GridShape,
        boundary_ranges: &SpecifiedBoundaryRanges,
        half_level_domain_start: usize,
        boundary_vertical_points: usize,
    ) -> Self {
        Self {
            column_mass: inputs.column_mass.values(),
            map_factor: inputs.map_factor.values(),
            column_mass_multiplier: inputs.column_mass_multiplier,
            column_mass_offset: inputs.column_mass_offset,
            boundary_values: inputs.boundary_values,
            boundary_tendencies: inputs.boundary_tendencies,
            west_east_points: shape.west_east_points(),
            south_north_points: shape.south_north_points(),
            west_east_domain: boundary_ranges.west_east_domain(),
            south_north_domain: boundary_ranges.south_north_domain(),
            bottom_top_range: boundary_ranges.bottom_top_range(),
            half_level_domain_start,
            boundary_vertical_points,
            time_step: parameters.time_step,
            uses_column_mass: location.uses_column_mass(),
            uses_map_factor: location.uses_map_factor(),
        }
    }

    pub(super) fn update_range(
        &self,
        output_plane: &mut [f32],
        south_north: usize,
        west_east_range: Range<usize>,
        side: SpecifiedBoundaryFinalizationSide,
    ) {
        let values = side.values(&self.boundary_values);
        let tendencies = side.values(&self.boundary_tendencies);
        let line_points = side.line_points(self.west_east_points, self.south_north_points);
        for bottom_top in self.bottom_top_range.clone() {
            let output_row_start = bottom_top * self.west_east_points;
            let boundary_vertical = bottom_top - self.half_level_domain_start;
            for west_east in west_east_range.clone() {
                let distance = side.distance(
                    west_east,
                    south_north,
                    &self.west_east_domain,
                    &self.south_north_domain,
                );
                let boundary_index = side.line_point(west_east, south_north)
                    + line_points * (boundary_vertical + self.boundary_vertical_points * distance);
                let boundary_value =
                    values[boundary_index] + self.time_step * tendencies[boundary_index];
                let horizontal_index = south_north * self.west_east_points + west_east;
                let point_map_factor = if self.uses_map_factor {
                    self.map_factor[horizontal_index]
                } else {
                    1.0
                };
                let point_column_mass = if self.uses_column_mass {
                    self.column_mass_multiplier[bottom_top] * self.column_mass[horizontal_index]
                        + self.column_mass_offset[bottom_top]
                } else {
                    1.0
                };
                output_plane[output_row_start + west_east] =
                    point_map_factor * boundary_value / point_column_mass;
            }
        }
    }
}
