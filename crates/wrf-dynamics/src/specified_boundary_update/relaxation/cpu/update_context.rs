use std::ops::Range;

use wrf_compute::{CpuField, FieldStorage, GridShape};

use super::super::SpecifiedBoundaryRelaxationInputs;
use super::boundary_side::SpecifiedBoundaryRelaxationSide;
use super::side_view::SpecifiedBoundaryRelaxationSideView;
use crate::specified_boundary_update::relaxation::geometry::SpecifiedBoundaryRelaxationRanges;

pub(super) struct SpecifiedBoundaryRelaxationCpuUpdateContext<'a> {
    inputs: &'a SpecifiedBoundaryRelaxationInputs<'a, CpuField<f32>>,
    model_west_east_points: usize,
    model_south_north_points: usize,
    field_west_east_start: usize,
    field_south_north_start: usize,
    field_bottom_top_start: usize,
    field_west_east_points: usize,
    field_bottom_top_points: usize,
    west_east_domain: Range<usize>,
    south_north_domain: Range<usize>,
    bottom_top_range: Range<usize>,
    half_level_domain_start: usize,
    boundary_vertical_points: usize,
    boundary_time_offset: f32,
}

impl<'a> SpecifiedBoundaryRelaxationCpuUpdateContext<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        inputs: &'a SpecifiedBoundaryRelaxationInputs<'a, CpuField<f32>>,
        model_shape: GridShape,
        ranges: &SpecifiedBoundaryRelaxationRanges,
        half_level_domain_start: usize,
        boundary_vertical_points: usize,
        boundary_time_offset: f32,
    ) -> Self {
        let field_shape = inputs.field.field.shape();
        Self {
            inputs,
            model_west_east_points: model_shape.west_east_points(),
            model_south_north_points: model_shape.south_north_points(),
            field_west_east_start: inputs.field.west_east.start,
            field_south_north_start: inputs.field.south_north.start,
            field_bottom_top_start: inputs.field.bottom_top.start,
            field_west_east_points: field_shape.west_east_points(),
            field_bottom_top_points: field_shape.bottom_top_points(),
            west_east_domain: ranges.west_east_domain(),
            south_north_domain: ranges.south_north_domain(),
            bottom_top_range: ranges.bottom_top_range(),
            half_level_domain_start,
            boundary_vertical_points,
            boundary_time_offset,
        }
    }

    #[inline(always)]
    pub(super) fn update_range(
        &self,
        output_plane: &mut [f32],
        south_north: usize,
        west_east_range: Range<usize>,
        side: SpecifiedBoundaryRelaxationSide,
    ) {
        for bottom_top in self.bottom_top_range.clone() {
            let output_row_start = bottom_top * self.model_west_east_points;
            let boundary_vertical = bottom_top - self.half_level_domain_start;
            let side_view = SpecifiedBoundaryRelaxationSideView::new(
                side,
                self.inputs,
                self.model_west_east_points,
                self.model_south_north_points,
                boundary_vertical,
            );
            for west_east in west_east_range.clone() {
                let output_index = output_row_start + west_east;
                output_plane[output_index] = self.updated_tendency(
                    output_plane[output_index],
                    west_east,
                    bottom_top,
                    south_north,
                    &side_view,
                );
            }
        }
    }

    #[inline(always)]
    fn updated_tendency(
        &self,
        current_tendency: f32,
        west_east: usize,
        bottom_top: usize,
        south_north: usize,
        side_view: &SpecifiedBoundaryRelaxationSideView<'_>,
    ) -> f32 {
        let side = side_view.side;
        let distance = side.distance(
            west_east,
            south_north,
            &self.west_east_domain,
            &self.south_north_domain,
        );
        let predecessor =
            side.tangential_predecessor(west_east, south_north, &self.west_east_domain);
        let successor = side.tangential_successor(west_east, south_north, &self.west_east_domain);
        let outer = side.outer_neighbor(west_east, south_north);
        let inner = side.inner_neighbor(west_east, south_north);
        let center = self.discrepancy(west_east, bottom_top, south_north, distance, side_view);
        let tangential_predecessor = self.discrepancy(
            predecessor.0,
            bottom_top,
            predecessor.1,
            distance,
            side_view,
        );
        let tangential_successor =
            self.discrepancy(successor.0, bottom_top, successor.1, distance, side_view);
        let outer = self.discrepancy(outer.0, bottom_top, outer.1, distance - 1, side_view);
        let inner = self.discrepancy(inner.0, bottom_top, inner.1, distance + 1, side_view);
        let neighbor_laplacian =
            (((tangential_predecessor + tangential_successor) + outer) + inner) - 4.0 * center;
        (current_tendency + self.inputs.coefficients.forcing[distance] * center)
            - self.inputs.coefficients.smoothing[distance] * neighbor_laplacian
    }

    #[inline(always)]
    fn discrepancy(
        &self,
        west_east: usize,
        bottom_top: usize,
        south_north: usize,
        boundary_distance: usize,
        side_view: &SpecifiedBoundaryRelaxationSideView<'_>,
    ) -> f32 {
        let line_point = side_view.side.line_point(west_east, south_north);
        let boundary_index = line_point
            + side_view.line_points
                * (side_view.boundary_vertical + self.boundary_vertical_points * boundary_distance);
        (side_view.boundary_values[boundary_index]
            + self.boundary_time_offset * side_view.boundary_tendencies[boundary_index])
            - self.field_value(west_east, bottom_top, south_north)
    }

    #[inline(always)]
    fn field_value(&self, west_east: usize, bottom_top: usize, south_north: usize) -> f32 {
        let local_west_east = west_east - self.field_west_east_start;
        let local_bottom_top = bottom_top - self.field_bottom_top_start;
        let local_south_north = south_north - self.field_south_north_start;
        let index = local_west_east
            + self.field_west_east_points
                * (local_bottom_top + self.field_bottom_top_points * local_south_north);
        self.inputs.field.field.values()[index]
    }
}
