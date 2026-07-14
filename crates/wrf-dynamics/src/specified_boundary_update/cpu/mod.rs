use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, ParallelExecutionError};

use crate::{
    SpecifiedBoundaryUpdateError, SpecifiedBoundaryUpdateKernels,
    SpecifiedBoundaryUpdateParameters, SpecifiedBoundaryUpdateRegion,
    SpecifiedBoundaryUpdateResult, SpecifiedBoundaryWestEastPeriodicity,
};

mod point_membership;
#[cfg(test)]
mod tests;
mod validation;

use point_membership::SpecifiedBoundaryPointMembership;

impl SpecifiedBoundaryUpdateKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn update_specified_boundary(
        &self,
        field: &mut Self::Field,
        tendency: &Self::Field,
        parameters: SpecifiedBoundaryUpdateParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryUpdateResult<()> {
        validation::validate(field, tendency, region)?;
        if parameters.specified_zone_width == 0 {
            return Ok(());
        }
        let shape = region.shape();
        let west_east_points = shape.west_east_points();
        let bottom_top_points = shape.bottom_top_points();
        let plane_length = west_east_points * bottom_top_points;
        let tendency_values = tendency.values();
        let ranges = region.active_ranges();
        let membership = SpecifiedBoundaryPointMembership::new(
            ranges,
            parameters.specified_zone_width,
            west_east_periodicity,
        );
        self.try_for_each_output_block::<_, Infallible, _>(
            field.values_mut(),
            plane_length,
            |south_north, output_plane| {
                let row_ranges = membership.ranges_for_row(south_north);
                let plane_start = south_north * plane_length;
                let tendency_plane = &tendency_values[plane_start..plane_start + plane_length];
                for west_east_range in [
                    row_ranges.south,
                    row_ranges.north,
                    row_ranges.west,
                    row_ranges.east,
                ]
                .into_iter()
                .flatten()
                {
                    update_range(
                        output_plane,
                        tendency_plane,
                        west_east_points,
                        membership.bottom_top_range(),
                        west_east_range,
                        parameters.time_step,
                    );
                }
                Ok(())
            },
        )
        .map_err(|error| match error {
            ParallelExecutionError::WorkerPanicked => SpecifiedBoundaryUpdateError::WorkerPanicked,
            ParallelExecutionError::Kernel(infallible) => match infallible {},
            ParallelExecutionError::ZeroBlockLength
            | ParallelExecutionError::IncompleteOutputBlock { .. }
            | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
                unreachable!("validated field shape creates complete planes")
            }
        })
    }
}

fn update_range(
    output_plane: &mut [f32],
    tendency_plane: &[f32],
    west_east_points: usize,
    bottom_top_range: std::ops::Range<usize>,
    west_east_range: std::ops::Range<usize>,
    time_step: f32,
) {
    for bottom_top in bottom_top_range {
        let row_start = bottom_top * west_east_points;
        for west_east in west_east_range.clone() {
            let index = row_start + west_east;
            output_plane[index] += time_step * tendency_plane[index];
        }
    }
}
