use std::convert::Infallible;
use std::ops::Range;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, GridShape, ParallelExecutionError};

use super::super::{
    SpecifiedBoundaryRelaxationError, SpecifiedBoundaryRelaxationInputs,
    SpecifiedBoundaryRelaxationParameters, SpecifiedBoundaryRelaxationResult,
};
use super::boundary_side::SpecifiedBoundaryRelaxationSide;
use super::update_context::SpecifiedBoundaryRelaxationCpuUpdateContext;
use crate::specified_boundary_update::relaxation::geometry::{
    SpecifiedBoundaryRelaxationCoverage, SpecifiedBoundaryRelaxationRanges,
};
use crate::{
    SpecifiedBoundaryFieldLocation, SpecifiedBoundaryUpdateAxis, SpecifiedBoundaryUpdateRegion,
    SpecifiedBoundaryWestEastPeriodicity,
};

pub(super) struct SpecifiedBoundaryRelaxationCpuKernel<'a> {
    tendency: &'a mut CpuField<f32>,
    inputs: SpecifiedBoundaryRelaxationInputs<'a, CpuField<f32>>,
    parameters: SpecifiedBoundaryRelaxationParameters,
    half_level_domain_start: usize,
    boundary_vertical_points: usize,
    ranges: SpecifiedBoundaryRelaxationRanges,
    has_updates: bool,
}

impl<'a> SpecifiedBoundaryRelaxationCpuKernel<'a> {
    pub(super) fn try_new(
        tendency: &'a mut CpuField<f32>,
        inputs: SpecifiedBoundaryRelaxationInputs<'a, CpuField<f32>>,
        parameters: SpecifiedBoundaryRelaxationParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryRelaxationResult<Self> {
        Self::validate_parameters(parameters)?;
        Self::validate_tendency(tendency, region)?;
        Self::validate_field_view(&inputs)?;
        let boundary_vertical_points = Self::boundary_vertical_points(region)?;
        Self::validate_boundary_fields(
            &inputs,
            region.shape(),
            boundary_vertical_points,
            parameters.boundary_width,
        )?;
        Self::validate_coefficients(&inputs, parameters.boundary_width)?;
        let ranges = SpecifiedBoundaryRelaxationRanges::new(
            region.active_ranges(),
            parameters.specified_zone_width,
            parameters.relaxation_zone_width,
            west_east_periodicity,
        );
        let has_updates = match ranges.required_field_coverage() {
            Some(coverage) => {
                Self::validate_field_coverage(&inputs, coverage)?;
                true
            }
            None => false,
        };
        Ok(Self {
            tendency,
            inputs,
            parameters,
            half_level_domain_start: region.mass_domains().2.start,
            boundary_vertical_points,
            ranges,
            has_updates,
        })
    }

    pub(super) fn execute(self, backend: &CpuBackend) -> SpecifiedBoundaryRelaxationResult<()> {
        if !self.has_updates {
            return Ok(());
        }
        let Self {
            tendency,
            inputs,
            parameters,
            half_level_domain_start,
            boundary_vertical_points,
            ranges,
            has_updates: _,
        } = self;
        let shape = tendency.shape();
        let plane_length = shape.west_east_points() * shape.bottom_top_points();
        let update_context = SpecifiedBoundaryRelaxationCpuUpdateContext::new(
            &inputs,
            shape,
            &ranges,
            half_level_domain_start,
            boundary_vertical_points,
            parameters.boundary_time_offset,
        );
        backend
            .try_for_each_output_block::<_, Infallible, _>(
                tendency.values_mut(),
                plane_length,
                |south_north, output_plane| {
                    let row_ranges = ranges.ranges_for_row(south_north);
                    if let Some(west_east_range) = row_ranges.south {
                        update_context.update_range(
                            output_plane,
                            south_north,
                            west_east_range,
                            SpecifiedBoundaryRelaxationSide::South,
                        );
                    }
                    if let Some(west_east_range) = row_ranges.north {
                        update_context.update_range(
                            output_plane,
                            south_north,
                            west_east_range,
                            SpecifiedBoundaryRelaxationSide::North,
                        );
                    }
                    if let Some(west_east_range) = row_ranges.west {
                        update_context.update_range(
                            output_plane,
                            south_north,
                            west_east_range,
                            SpecifiedBoundaryRelaxationSide::West,
                        );
                    }
                    if let Some(west_east_range) = row_ranges.east {
                        update_context.update_range(
                            output_plane,
                            south_north,
                            west_east_range,
                            SpecifiedBoundaryRelaxationSide::East,
                        );
                    }
                    Ok(())
                },
            )
            .map_err(Self::map_parallel_error)
    }

    fn validate_parameters(
        parameters: SpecifiedBoundaryRelaxationParameters,
    ) -> SpecifiedBoundaryRelaxationResult<()> {
        if parameters.boundary_width == 0 {
            return Err(SpecifiedBoundaryRelaxationError::InvalidBoundaryWidth);
        }
        if parameters.specified_zone_width == 0 {
            return Err(SpecifiedBoundaryRelaxationError::InvalidSpecifiedZoneWidth);
        }
        for zone_width in [
            parameters.specified_zone_width,
            parameters.relaxation_zone_width,
        ] {
            if zone_width > parameters.boundary_width {
                return Err(SpecifiedBoundaryRelaxationError::ZoneExceedsBoundaryWidth {
                    boundary_width: parameters.boundary_width,
                    zone_width,
                });
            }
        }
        if parameters.relaxation_zone_width > parameters.specified_zone_width
            && parameters.relaxation_zone_width >= parameters.boundary_width
        {
            return Err(
                SpecifiedBoundaryRelaxationError::RelaxationNeighborOutsideBoundary {
                    boundary_width: parameters.boundary_width,
                    relaxation_zone_width: parameters.relaxation_zone_width,
                },
            );
        }
        Ok(())
    }

    fn validate_tendency(
        tendency: &CpuField<f32>,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryRelaxationResult<()> {
        if tendency.shape() != region.shape() {
            return Err(SpecifiedBoundaryRelaxationError::ShapeMismatch {
                expected: region.shape(),
                actual: tendency.shape(),
            });
        }
        Ok(())
    }

    fn validate_field_view(
        inputs: &SpecifiedBoundaryRelaxationInputs<'_, CpuField<f32>>,
    ) -> SpecifiedBoundaryRelaxationResult<()> {
        let expected_west_east_points = inputs.field.west_east.len();
        let expected_south_north_points = inputs.field.south_north.len();
        let expected_bottom_top_points = inputs.field.bottom_top.len();
        let actual = inputs.field.field.shape();
        if actual.west_east_points() != expected_west_east_points
            || actual.south_north_points() != expected_south_north_points
            || actual.bottom_top_points() != expected_bottom_top_points
        {
            return Err(SpecifiedBoundaryRelaxationError::FieldViewShapeMismatch {
                expected_west_east_points,
                expected_south_north_points,
                expected_bottom_top_points,
                actual,
            });
        }
        Ok(())
    }

    fn validate_field_coverage(
        inputs: &SpecifiedBoundaryRelaxationInputs<'_, CpuField<f32>>,
        coverage: SpecifiedBoundaryRelaxationCoverage,
    ) -> SpecifiedBoundaryRelaxationResult<()> {
        for (axis, required, actual) in [
            (
                SpecifiedBoundaryUpdateAxis::WestEast,
                coverage.west_east,
                inputs.field.west_east.clone(),
            ),
            (
                SpecifiedBoundaryUpdateAxis::SouthNorth,
                coverage.south_north,
                inputs.field.south_north.clone(),
            ),
            (
                SpecifiedBoundaryUpdateAxis::BottomTop,
                coverage.bottom_top,
                inputs.field.bottom_top.clone(),
            ),
        ] {
            if !contains_range(&actual, &required) {
                return Err(SpecifiedBoundaryRelaxationError::FieldCoverageMissing {
                    axis,
                    required,
                    actual,
                });
            }
        }
        Ok(())
    }

    fn boundary_vertical_points(
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryRelaxationResult<usize> {
        if region.location() == SpecifiedBoundaryFieldLocation::HorizontalMass {
            return Ok(1);
        }
        region
            .mass_domains()
            .2
            .len()
            .checked_add(1)
            .ok_or(SpecifiedBoundaryRelaxationError::BoundaryVerticalExtentOverflow)
    }

    fn validate_boundary_fields(
        inputs: &SpecifiedBoundaryRelaxationInputs<'_, CpuField<f32>>,
        model_shape: GridShape,
        vertical_points: usize,
        boundary_width: usize,
    ) -> SpecifiedBoundaryRelaxationResult<()> {
        for (field, values, line_points) in [
            (
                "west boundary values",
                inputs.boundary_values.west,
                model_shape.south_north_points(),
            ),
            (
                "east boundary values",
                inputs.boundary_values.east,
                model_shape.south_north_points(),
            ),
            (
                "south boundary values",
                inputs.boundary_values.south,
                model_shape.west_east_points(),
            ),
            (
                "north boundary values",
                inputs.boundary_values.north,
                model_shape.west_east_points(),
            ),
            (
                "west boundary tendencies",
                inputs.boundary_tendencies.west,
                model_shape.south_north_points(),
            ),
            (
                "east boundary tendencies",
                inputs.boundary_tendencies.east,
                model_shape.south_north_points(),
            ),
            (
                "south boundary tendencies",
                inputs.boundary_tendencies.south,
                model_shape.west_east_points(),
            ),
            (
                "north boundary tendencies",
                inputs.boundary_tendencies.north,
                model_shape.west_east_points(),
            ),
        ] {
            let shape = values.shape();
            if shape.west_east_points() != line_points
                || shape.bottom_top_points() != vertical_points
                || shape.south_north_points() != boundary_width
            {
                return Err(SpecifiedBoundaryRelaxationError::BoundaryShapeMismatch {
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

    fn validate_coefficients(
        inputs: &SpecifiedBoundaryRelaxationInputs<'_, CpuField<f32>>,
        boundary_width: usize,
    ) -> SpecifiedBoundaryRelaxationResult<()> {
        for (coefficient, actual) in [
            ("forcing", inputs.coefficients.forcing.len()),
            ("smoothing", inputs.coefficients.smoothing.len()),
        ] {
            if actual != boundary_width {
                return Err(
                    SpecifiedBoundaryRelaxationError::CoefficientLengthMismatch {
                        coefficient,
                        expected: boundary_width,
                        actual,
                    },
                );
            }
        }
        Ok(())
    }

    fn map_parallel_error(
        error: ParallelExecutionError<Infallible>,
    ) -> SpecifiedBoundaryRelaxationError {
        match error {
            ParallelExecutionError::Kernel(infallible) => match infallible {},
            ParallelExecutionError::WorkerPanicked => {
                SpecifiedBoundaryRelaxationError::WorkerPanicked
            }
            ParallelExecutionError::ZeroBlockLength
            | ParallelExecutionError::IncompleteOutputBlock { .. }
            | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
                SpecifiedBoundaryRelaxationError::SchedulerContractViolated
            }
        }
    }
}

fn contains_range(actual: &Range<usize>, required: &Range<usize>) -> bool {
    actual.start <= required.start && actual.end >= required.end
}
