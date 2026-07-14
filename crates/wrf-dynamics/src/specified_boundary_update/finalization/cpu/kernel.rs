use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, GridShape, ParallelExecutionError};

use super::super::{
    SpecifiedBoundaryFinalizationBoundaryFields, SpecifiedBoundaryFinalizationError,
    SpecifiedBoundaryFinalizationFieldLocation, SpecifiedBoundaryFinalizationInputs,
    SpecifiedBoundaryFinalizationParameters, SpecifiedBoundaryFinalizationRegion,
    SpecifiedBoundaryFinalizationResult,
};
use super::boundary_side::SpecifiedBoundaryFinalizationSide;
use super::update_context::SpecifiedBoundaryFinalizationCpuUpdateContext;
use crate::SpecifiedBoundaryWestEastPeriodicity;
use crate::specified_boundary_update::geometry::SpecifiedBoundaryRanges;

pub(super) struct SpecifiedBoundaryFinalizationCpuKernel<'a> {
    field: &'a mut CpuField<f32>,
    inputs: SpecifiedBoundaryFinalizationInputs<'a, CpuField<f32>>,
    parameters: SpecifiedBoundaryFinalizationParameters,
    location: SpecifiedBoundaryFinalizationFieldLocation,
    half_level_domain_start: usize,
    boundary_vertical_points: usize,
    boundary_ranges: SpecifiedBoundaryRanges,
}

impl<'a> SpecifiedBoundaryFinalizationCpuKernel<'a> {
    pub(super) fn try_new(
        field: &'a mut CpuField<f32>,
        inputs: SpecifiedBoundaryFinalizationInputs<'a, CpuField<f32>>,
        parameters: SpecifiedBoundaryFinalizationParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryFinalizationRegion,
    ) -> SpecifiedBoundaryFinalizationResult<Self> {
        Self::validate_parameters(parameters)?;
        let boundary_vertical_points = region.boundary_vertical_points()?;
        Self::validate_fields(field, &inputs, region)?;
        Self::validate_coefficients(&inputs, region.shape())?;
        Self::validate_boundary_fields(
            &inputs.boundary_values,
            "boundary values",
            region.shape(),
            boundary_vertical_points,
            parameters.boundary_width,
        )?;
        Self::validate_boundary_fields(
            &inputs.boundary_tendencies,
            "boundary tendencies",
            region.shape(),
            boundary_vertical_points,
            parameters.boundary_width,
        )?;
        Ok(Self {
            field,
            inputs,
            parameters,
            location: region.location(),
            half_level_domain_start: region.half_level_domain().start,
            boundary_vertical_points,
            boundary_ranges: SpecifiedBoundaryRanges::new(
                region.active_ranges(),
                parameters.specified_zone_width,
                west_east_periodicity,
            ),
        })
    }

    pub(super) fn execute(self, backend: &CpuBackend) -> SpecifiedBoundaryFinalizationResult<()> {
        if self.parameters.specified_zone_width == 0 {
            return Ok(());
        }
        let shape = self.field.shape();
        let west_east_points = shape.west_east_points();
        let bottom_top_points = shape.bottom_top_points();
        let plane_length = west_east_points * bottom_top_points;
        let boundary_ranges = self.boundary_ranges;
        let update_context = SpecifiedBoundaryFinalizationCpuUpdateContext::new(
            &self.inputs,
            self.parameters,
            self.location,
            shape,
            &boundary_ranges,
            self.half_level_domain_start,
            self.boundary_vertical_points,
        );

        backend
            .try_for_each_output_block::<_, Infallible, _>(
                self.field.values_mut(),
                plane_length,
                |south_north, output_plane| {
                    let row_ranges = boundary_ranges.ranges_for_row(south_north);
                    for (side, west_east_range) in [
                        (SpecifiedBoundaryFinalizationSide::South, row_ranges.south),
                        (SpecifiedBoundaryFinalizationSide::North, row_ranges.north),
                        (SpecifiedBoundaryFinalizationSide::West, row_ranges.west),
                        (SpecifiedBoundaryFinalizationSide::East, row_ranges.east),
                    ]
                    .into_iter()
                    .filter_map(|(side, range)| range.map(|range| (side, range)))
                    {
                        update_context.update_range(
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
        parameters: SpecifiedBoundaryFinalizationParameters,
    ) -> SpecifiedBoundaryFinalizationResult<()> {
        if parameters.boundary_width == 0 {
            return Err(SpecifiedBoundaryFinalizationError::InvalidBoundaryWidth);
        }
        if parameters.specified_zone_width > parameters.boundary_width {
            return Err(
                SpecifiedBoundaryFinalizationError::SpecifiedZoneExceedsBoundaryWidth {
                    boundary_width: parameters.boundary_width,
                    specified_zone_width: parameters.specified_zone_width,
                },
            );
        }
        Ok(())
    }

    fn validate_fields(
        field: &CpuField<f32>,
        inputs: &SpecifiedBoundaryFinalizationInputs<'_, CpuField<f32>>,
        region: &SpecifiedBoundaryFinalizationRegion,
    ) -> SpecifiedBoundaryFinalizationResult<()> {
        Self::validate_shape("field", field, region.shape())?;
        let horizontal_shape = region.shape().horizontal_shape();
        Self::validate_shape("column mass", inputs.column_mass, horizontal_shape)?;
        Self::validate_shape("map factor", inputs.map_factor, horizontal_shape)
    }

    fn validate_shape(
        field_name: &'static str,
        field: &CpuField<f32>,
        expected: GridShape,
    ) -> SpecifiedBoundaryFinalizationResult<()> {
        if field.shape() != expected {
            return Err(SpecifiedBoundaryFinalizationError::ShapeMismatch {
                field: field_name,
                expected,
                actual: field.shape(),
            });
        }
        Ok(())
    }

    fn validate_coefficients(
        inputs: &SpecifiedBoundaryFinalizationInputs<'_, CpuField<f32>>,
        shape: GridShape,
    ) -> SpecifiedBoundaryFinalizationResult<()> {
        Self::validate_coefficient_length(
            "column mass multiplier",
            inputs.column_mass_multiplier,
            shape.bottom_top_points(),
        )?;
        Self::validate_coefficient_length(
            "column mass offset",
            inputs.column_mass_offset,
            shape.bottom_top_points(),
        )
    }

    fn validate_coefficient_length(
        coefficient: &'static str,
        values: &[f32],
        expected: usize,
    ) -> SpecifiedBoundaryFinalizationResult<()> {
        if values.len() != expected {
            return Err(
                SpecifiedBoundaryFinalizationError::CoefficientLengthMismatch {
                    coefficient,
                    expected,
                    actual: values.len(),
                },
            );
        }
        Ok(())
    }

    fn validate_boundary_fields(
        fields: &SpecifiedBoundaryFinalizationBoundaryFields<'_, CpuField<f32>>,
        role: &'static str,
        model_shape: GridShape,
        vertical_points: usize,
        boundary_width: usize,
    ) -> SpecifiedBoundaryFinalizationResult<()> {
        for (side, field, line_points) in [
            ("west", fields.west, model_shape.south_north_points()),
            ("east", fields.east, model_shape.south_north_points()),
            ("south", fields.south, model_shape.west_east_points()),
            ("north", fields.north, model_shape.west_east_points()),
        ] {
            let shape = field.shape();
            if shape.west_east_points() != line_points
                || shape.bottom_top_points() != vertical_points
                || shape.south_north_points() != boundary_width
            {
                return Err(SpecifiedBoundaryFinalizationError::BoundaryShapeMismatch {
                    field: match (role, side) {
                        ("boundary values", "west") => "west boundary values",
                        ("boundary values", "east") => "east boundary values",
                        ("boundary values", "south") => "south boundary values",
                        ("boundary values", "north") => "north boundary values",
                        ("boundary tendencies", "west") => "west boundary tendencies",
                        ("boundary tendencies", "east") => "east boundary tendencies",
                        ("boundary tendencies", "south") => "south boundary tendencies",
                        ("boundary tendencies", "north") => "north boundary tendencies",
                        _ => "boundary field",
                    },
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
    ) -> SpecifiedBoundaryFinalizationError {
        match error {
            ParallelExecutionError::Kernel(infallible) => match infallible {},
            ParallelExecutionError::WorkerPanicked => {
                SpecifiedBoundaryFinalizationError::WorkerPanicked
            }
            ParallelExecutionError::ZeroBlockLength
            | ParallelExecutionError::IncompleteOutputBlock { .. }
            | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
                SpecifiedBoundaryFinalizationError::SchedulerContractViolated
            }
        }
    }
}
