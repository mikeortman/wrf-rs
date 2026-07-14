use std::ops::Range;

use wrf_compute::{CpuField, GridShape};

use super::super::SpecifiedBoundaryTendencies;
use super::boundary_side::SpecifiedBoundaryTendencySide;
use crate::specified_boundary_update::geometry::SpecifiedBoundaryRanges;

pub(super) struct SpecifiedBoundaryTendencyCpuUpdateContext<'a> {
    boundary_tendencies: SpecifiedBoundaryTendencies<'a, CpuField<f32>>,
    west_east_points: usize,
    south_north_points: usize,
    west_east_domain: Range<usize>,
    south_north_domain: Range<usize>,
    bottom_top_range: Range<usize>,
    half_level_domain_start: usize,
    boundary_vertical_points: usize,
}

impl<'a> SpecifiedBoundaryTendencyCpuUpdateContext<'a> {
    pub(super) fn new(
        boundary_tendencies: SpecifiedBoundaryTendencies<'a, CpuField<f32>>,
        shape: GridShape,
        boundary_ranges: &SpecifiedBoundaryRanges,
        half_level_domain_start: usize,
        boundary_vertical_points: usize,
    ) -> Self {
        Self {
            boundary_tendencies,
            west_east_points: shape.west_east_points(),
            south_north_points: shape.south_north_points(),
            west_east_domain: boundary_ranges.west_east_domain(),
            south_north_domain: boundary_ranges.south_north_domain(),
            bottom_top_range: boundary_ranges.bottom_top_range(),
            half_level_domain_start,
            boundary_vertical_points,
        }
    }

    pub(super) fn assign_range(
        &self,
        output_plane: &mut [f32],
        south_north: usize,
        west_east_range: Range<usize>,
        side: SpecifiedBoundaryTendencySide,
    ) {
        let values = side.values(&self.boundary_tendencies);
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
                output_plane[output_row_start + west_east] = values[boundary_index];
            }
        }
    }
}
