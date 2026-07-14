use wrf_compute::CpuField;

use super::super::SpecifiedBoundaryRelaxationInputs;
use super::boundary_side::SpecifiedBoundaryRelaxationSide;

pub(super) struct SpecifiedBoundaryRelaxationSideView<'a> {
    pub(super) side: SpecifiedBoundaryRelaxationSide,
    pub(super) boundary_values: &'a [f32],
    pub(super) boundary_tendencies: &'a [f32],
    pub(super) line_points: usize,
    pub(super) boundary_vertical: usize,
}

impl<'a> SpecifiedBoundaryRelaxationSideView<'a> {
    #[inline(always)]
    pub(super) fn new(
        side: SpecifiedBoundaryRelaxationSide,
        inputs: &'a SpecifiedBoundaryRelaxationInputs<'_, CpuField<f32>>,
        model_west_east_points: usize,
        model_south_north_points: usize,
        boundary_vertical: usize,
    ) -> Self {
        Self {
            side,
            boundary_values: side.boundary_values(inputs),
            boundary_tendencies: side.boundary_tendencies(inputs),
            line_points: side.line_points(model_west_east_points, model_south_north_points),
            boundary_vertical,
        }
    }
}
