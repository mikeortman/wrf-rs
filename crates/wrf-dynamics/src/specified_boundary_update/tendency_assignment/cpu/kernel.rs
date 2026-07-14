use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, GridShape, ParallelExecutionError};

use super::super::{
    SpecifiedBoundaryTendencies, SpecifiedBoundaryTendencyError,
    SpecifiedBoundaryTendencyParameters, SpecifiedBoundaryTendencyResult,
};
use super::boundary_side::SpecifiedBoundaryTendencySide;
use super::update_context::SpecifiedBoundaryTendencyCpuUpdateContext;
use crate::specified_boundary_update::geometry::SpecifiedBoundaryRanges;
use crate::{
    SpecifiedBoundaryFieldLocation, SpecifiedBoundaryUpdateRegion,
    SpecifiedBoundaryWestEastPeriodicity,
};

pub(super) struct SpecifiedBoundaryTendencyCpuKernel<'a> {
    tendency: &'a mut CpuField<f32>,
    boundary_tendencies: SpecifiedBoundaryTendencies<'a, CpuField<f32>>,
    parameters: SpecifiedBoundaryTendencyParameters,
    half_level_domain_start: usize,
    boundary_vertical_points: usize,
    boundary_ranges: SpecifiedBoundaryRanges,
}

impl<'a> SpecifiedBoundaryTendencyCpuKernel<'a> {
    pub(super) fn try_new(
        tendency: &'a mut CpuField<f32>,
        boundary_tendencies: SpecifiedBoundaryTendencies<'a, CpuField<f32>>,
        parameters: SpecifiedBoundaryTendencyParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryTendencyResult<Self> {
        Self::validate_parameters(parameters)?;
        let boundary_vertical_points = Self::boundary_vertical_points(region)?;
        Self::validate_tendency(tendency, region)?;
        Self::validate_boundary_tendencies(
            &boundary_tendencies,
            region.shape(),
            boundary_vertical_points,
            parameters.boundary_width,
        )?;
        Ok(Self {
            tendency,
            boundary_tendencies,
            parameters,
            half_level_domain_start: region.mass_domains().2.start,
            boundary_vertical_points,
            boundary_ranges: SpecifiedBoundaryRanges::new(
                region.active_ranges(),
                parameters.specified_zone_width,
                west_east_periodicity,
            ),
        })
    }

    pub(super) fn execute(self, backend: &CpuBackend) -> SpecifiedBoundaryTendencyResult<()> {
        if self.parameters.specified_zone_width == 0 {
            return Ok(());
        }
        let shape = self.tendency.shape();
        let plane_length = shape.west_east_points() * shape.bottom_top_points();
        let boundary_ranges = self.boundary_ranges;
        let update_context = SpecifiedBoundaryTendencyCpuUpdateContext::new(
            self.boundary_tendencies,
            shape,
            &boundary_ranges,
            self.half_level_domain_start,
            self.boundary_vertical_points,
        );
        backend
            .try_for_each_output_block::<_, Infallible, _>(
                self.tendency.values_mut(),
                plane_length,
                |south_north, output_plane| {
                    let row_ranges = boundary_ranges.ranges_for_row(south_north);
                    for (side, west_east_range) in [
                        (SpecifiedBoundaryTendencySide::South, row_ranges.south),
                        (SpecifiedBoundaryTendencySide::North, row_ranges.north),
                        (SpecifiedBoundaryTendencySide::West, row_ranges.west),
                        (SpecifiedBoundaryTendencySide::East, row_ranges.east),
                    ]
                    .into_iter()
                    .filter_map(|(side, range)| range.map(|range| (side, range)))
                    {
                        update_context.assign_range(
                            output_plane,
                            south_north,
                            west_east_range,
                            side,
                        );
                    }
                    Ok(())
                },
            )
            .map_err(Self::map_parallel_error)
    }

    fn validate_parameters(
        parameters: SpecifiedBoundaryTendencyParameters,
    ) -> SpecifiedBoundaryTendencyResult<()> {
        if parameters.boundary_width == 0 {
            return Err(SpecifiedBoundaryTendencyError::InvalidBoundaryWidth);
        }
        if parameters.specified_zone_width > parameters.boundary_width {
            return Err(
                SpecifiedBoundaryTendencyError::SpecifiedZoneExceedsBoundaryWidth {
                    boundary_width: parameters.boundary_width,
                    specified_zone_width: parameters.specified_zone_width,
                },
            );
        }
        Ok(())
    }

    fn boundary_vertical_points(
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryTendencyResult<usize> {
        if region.location() == SpecifiedBoundaryFieldLocation::HorizontalMass {
            return Ok(1);
        }
        region
            .mass_domains()
            .2
            .len()
            .checked_add(1)
            .ok_or(SpecifiedBoundaryTendencyError::BoundaryVerticalExtentOverflow)
    }

    fn validate_tendency(
        tendency: &CpuField<f32>,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryTendencyResult<()> {
        if tendency.shape() != region.shape() {
            return Err(SpecifiedBoundaryTendencyError::ShapeMismatch {
                expected: region.shape(),
                actual: tendency.shape(),
            });
        }
        Ok(())
    }

    fn validate_boundary_tendencies(
        tendencies: &SpecifiedBoundaryTendencies<'_, CpuField<f32>>,
        model_shape: GridShape,
        vertical_points: usize,
        boundary_width: usize,
    ) -> SpecifiedBoundaryTendencyResult<()> {
        for (field, tendency, line_points) in [
            (
                "west boundary tendencies",
                tendencies.west,
                model_shape.south_north_points(),
            ),
            (
                "east boundary tendencies",
                tendencies.east,
                model_shape.south_north_points(),
            ),
            (
                "south boundary tendencies",
                tendencies.south,
                model_shape.west_east_points(),
            ),
            (
                "north boundary tendencies",
                tendencies.north,
                model_shape.west_east_points(),
            ),
        ] {
            let shape = tendency.shape();
            if shape.west_east_points() != line_points
                || shape.bottom_top_points() != vertical_points
                || shape.south_north_points() != boundary_width
            {
                return Err(SpecifiedBoundaryTendencyError::BoundaryShapeMismatch {
                    field,
                    expected_line_points: line_points,
                    expected_vertical_points: vertical_points,
                    expected_boundary_width: boundary_width,
                    actual: shape,
                });
            }
        }
        Ok(())
    }

    fn map_parallel_error(
        error: ParallelExecutionError<Infallible>,
    ) -> SpecifiedBoundaryTendencyError {
        match error {
            ParallelExecutionError::Kernel(infallible) => match infallible {},
            ParallelExecutionError::WorkerPanicked => {
                SpecifiedBoundaryTendencyError::WorkerPanicked
            }
            ParallelExecutionError::ZeroBlockLength
            | ParallelExecutionError::IncompleteOutputBlock { .. }
            | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
                SpecifiedBoundaryTendencyError::SchedulerContractViolated
            }
        }
    }
}
