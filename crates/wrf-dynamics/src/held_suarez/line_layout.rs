use std::ops::Range;

use super::simd::MomentumDampingLine;

pub(super) struct MomentumDampingInputSlices<'a> {
    west_east_momentum: &'a [f32],
    south_north_momentum: &'a [f32],
    perturbation_pressure: &'a [f32],
    base_pressure: &'a [f32],
    west_east_points: usize,
    bottom_top_points: usize,
    west_east_range: Range<usize>,
    surface_level: usize,
}

pub(super) struct MomentumDampingLayout {
    pub(super) west_east_points: usize,
    pub(super) bottom_top_points: usize,
    pub(super) west_east_range: Range<usize>,
    pub(super) surface_level: usize,
}

impl<'a> MomentumDampingInputSlices<'a> {
    pub(super) fn new(
        west_east_momentum: &'a [f32],
        south_north_momentum: &'a [f32],
        perturbation_pressure: &'a [f32],
        base_pressure: &'a [f32],
        layout: MomentumDampingLayout,
    ) -> Self {
        Self {
            west_east_momentum,
            south_north_momentum,
            perturbation_pressure,
            base_pressure,
            west_east_points: layout.west_east_points,
            bottom_top_points: layout.bottom_top_points,
            west_east_range: layout.west_east_range,
            surface_level: layout.surface_level,
        }
    }

    #[inline(always)]
    pub(super) fn south_north_momentum_line<'line>(
        &'line self,
        tendency_line: &'line mut [f32],
        bottom_top_index: usize,
        south_north_index: usize,
    ) -> MomentumDampingLine<'line> {
        let current_line_start = self.line_start(bottom_top_index, south_north_index);
        let adjacent_line_start = self.line_start(bottom_top_index, south_north_index - 1);
        let current_surface_start = self.line_start(self.surface_level, south_north_index);
        let adjacent_surface_start = self.line_start(self.surface_level, south_north_index - 1);
        MomentumDampingLine {
            tendency: &mut tendency_line[self.west_east_range.clone()],
            momentum: self.field_line(self.south_north_momentum, current_line_start),
            current_pressure: self.field_line(self.perturbation_pressure, current_line_start),
            current_base_pressure: self.field_line(self.base_pressure, current_line_start),
            adjacent_pressure: self.field_line(self.perturbation_pressure, adjacent_line_start),
            adjacent_base_pressure: self.field_line(self.base_pressure, adjacent_line_start),
            current_surface_pressure: self
                .field_line(self.perturbation_pressure, current_surface_start),
            current_surface_base_pressure: self
                .field_line(self.base_pressure, current_surface_start),
            adjacent_surface_pressure: self
                .field_line(self.perturbation_pressure, adjacent_surface_start),
            adjacent_surface_base_pressure: self
                .field_line(self.base_pressure, adjacent_surface_start),
        }
    }

    #[inline(always)]
    pub(super) fn west_east_momentum_line<'line>(
        &'line self,
        tendency_line: &'line mut [f32],
        bottom_top_index: usize,
        south_north_index: usize,
    ) -> MomentumDampingLine<'line> {
        let current_line_start = self.line_start(bottom_top_index, south_north_index);
        let surface_line_start = self.line_start(self.surface_level, south_north_index);
        let adjacent_range = (self.west_east_range.start - 1)..(self.west_east_range.end - 1);
        MomentumDampingLine {
            tendency: &mut tendency_line[self.west_east_range.clone()],
            momentum: self.field_line(self.west_east_momentum, current_line_start),
            current_pressure: self.field_line(self.perturbation_pressure, current_line_start),
            current_base_pressure: self.field_line(self.base_pressure, current_line_start),
            adjacent_pressure: field_line(
                self.perturbation_pressure,
                current_line_start,
                &adjacent_range,
            ),
            adjacent_base_pressure: field_line(
                self.base_pressure,
                current_line_start,
                &adjacent_range,
            ),
            current_surface_pressure: self
                .field_line(self.perturbation_pressure, surface_line_start),
            current_surface_base_pressure: self.field_line(self.base_pressure, surface_line_start),
            adjacent_surface_pressure: field_line(
                self.perturbation_pressure,
                surface_line_start,
                &adjacent_range,
            ),
            adjacent_surface_base_pressure: field_line(
                self.base_pressure,
                surface_line_start,
                &adjacent_range,
            ),
        }
    }

    #[inline(always)]
    fn field_line<'field>(&self, field: &'field [f32], line_start: usize) -> &'field [f32] {
        field_line(field, line_start, &self.west_east_range)
    }

    const fn line_start(&self, bottom_top_index: usize, south_north_index: usize) -> usize {
        linear_index(
            0,
            bottom_top_index,
            south_north_index,
            self.west_east_points,
            self.bottom_top_points,
        )
    }
}

#[inline(always)]
fn field_line<'a>(field: &'a [f32], line_start: usize, range: &Range<usize>) -> &'a [f32] {
    &field[(line_start + range.start)..(line_start + range.end)]
}

pub(super) const fn linear_index(
    west_east_index: usize,
    bottom_top_index: usize,
    south_north_index: usize,
    west_east_points: usize,
    bottom_top_points: usize,
) -> usize {
    (south_north_index * bottom_top_points + bottom_top_index) * west_east_points + west_east_index
}
