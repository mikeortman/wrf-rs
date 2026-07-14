use std::convert::Infallible;
use std::ops::Range;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, ParallelExecutionError};

use crate::specified_boundary_update::geometry::SpecifiedBoundaryRanges;
use crate::{
    SpecifiedBoundaryGeopotentialError, SpecifiedBoundaryGeopotentialInputs,
    SpecifiedBoundaryGeopotentialResult, SpecifiedBoundaryUpdateParameters,
    SpecifiedBoundaryUpdateRegion, SpecifiedBoundaryWestEastPeriodicity,
};

pub(super) struct SpecifiedBoundaryGeopotentialCpuKernel<'a> {
    geopotential: &'a mut CpuField<f32>,
    inputs: SpecifiedBoundaryGeopotentialInputs<'a, CpuField<f32>>,
    parameters: SpecifiedBoundaryUpdateParameters,
    boundary_ranges: SpecifiedBoundaryRanges,
}

impl<'a> SpecifiedBoundaryGeopotentialCpuKernel<'a> {
    pub(super) fn try_new(
        geopotential: &'a mut CpuField<f32>,
        inputs: SpecifiedBoundaryGeopotentialInputs<'a, CpuField<f32>>,
        parameters: SpecifiedBoundaryUpdateParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryGeopotentialResult<Self> {
        Self::validate_fields(geopotential, &inputs, region)?;
        Self::validate_coefficients(&inputs, region)?;
        Ok(Self {
            geopotential,
            inputs,
            parameters,
            boundary_ranges: SpecifiedBoundaryRanges::new(
                region.active_ranges(),
                parameters.specified_zone_width,
                west_east_periodicity,
            ),
        })
    }

    pub(super) fn execute(self, backend: &CpuBackend) -> SpecifiedBoundaryGeopotentialResult<()> {
        if self.parameters.specified_zone_width == 0 {
            return Ok(());
        }
        let shape = self.geopotential.shape();
        let west_east_points = shape.west_east_points();
        let bottom_top_points = shape.bottom_top_points();
        let plane_length = west_east_points * bottom_top_points;
        let saved_geopotential = self.inputs.saved_geopotential.values();
        let geopotential_tendency = self.inputs.geopotential_tendency.values();
        let column_mass_tendency = self.inputs.column_mass_tendency.values();
        let current_column_mass = self.inputs.current_column_mass.values();
        let column_mass_multiplier = self.inputs.column_mass_multiplier;
        let column_mass_offset = self.inputs.column_mass_offset;
        let parameters = self.parameters;
        let boundary_ranges = self.boundary_ranges;

        backend
            .try_for_each_output_block::<_, Infallible, _>(
                self.geopotential.values_mut(),
                plane_length,
                |south_north, output_plane| {
                    let row_ranges = boundary_ranges.ranges_for_row(south_north);
                    let plane_start = south_north * plane_length;
                    let saved_plane = &saved_geopotential[plane_start..plane_start + plane_length];
                    let tendency_plane =
                        &geopotential_tendency[plane_start..plane_start + plane_length];
                    for west_east_range in [
                        row_ranges.south,
                        row_ranges.north,
                        row_ranges.west,
                        row_ranges.east,
                    ]
                    .into_iter()
                    .flatten()
                    {
                        Self::update_range(
                            output_plane,
                            saved_plane,
                            tendency_plane,
                            column_mass_tendency,
                            current_column_mass,
                            column_mass_multiplier,
                            column_mass_offset,
                            west_east_points,
                            south_north,
                            boundary_ranges.bottom_top_range(),
                            west_east_range,
                            parameters.time_step,
                        );
                    }
                    Ok(())
                },
            )
            .map_err(Self::map_parallel_error)
    }

    #[allow(clippy::too_many_arguments)]
    fn update_range(
        output_plane: &mut [f32],
        saved_plane: &[f32],
        tendency_plane: &[f32],
        column_mass_tendency: &[f32],
        current_column_mass: &[f32],
        column_mass_multiplier: &[f32],
        column_mass_offset: &[f32],
        west_east_points: usize,
        south_north: usize,
        bottom_top_range: Range<usize>,
        west_east_range: Range<usize>,
        time_step: f32,
    ) {
        for bottom_top in bottom_top_range {
            let row_start = bottom_top * west_east_points;
            for west_east in west_east_range.clone() {
                let plane_index = row_start + west_east;
                let horizontal_index = south_north * west_east_points + west_east;
                let previous_column_mass = current_column_mass[horizontal_index]
                    - time_step * column_mass_tendency[horizontal_index];
                let previous_mass = column_mass_multiplier[bottom_top] * previous_column_mass
                    + column_mass_offset[bottom_top];
                let current_mass = column_mass_multiplier[bottom_top]
                    * current_column_mass[horizontal_index]
                    + column_mass_offset[bottom_top];
                output_plane[plane_index] = output_plane[plane_index] * previous_mass
                    / current_mass
                    + time_step * tendency_plane[plane_index] / current_mass
                    + saved_plane[plane_index] * (previous_mass / current_mass - 1.0);
            }
        }
    }

    fn validate_fields(
        geopotential: &CpuField<f32>,
        inputs: &SpecifiedBoundaryGeopotentialInputs<'_, CpuField<f32>>,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryGeopotentialResult<()> {
        Self::validate_shape("geopotential", geopotential, region.shape())?;
        Self::validate_shape(
            "saved geopotential",
            inputs.saved_geopotential,
            region.shape(),
        )?;
        Self::validate_shape(
            "geopotential tendency",
            inputs.geopotential_tendency,
            region.shape(),
        )?;
        let horizontal_shape = region.shape().horizontal_shape();
        Self::validate_shape(
            "column mass tendency",
            inputs.column_mass_tendency,
            horizontal_shape,
        )?;
        Self::validate_shape(
            "current column mass",
            inputs.current_column_mass,
            horizontal_shape,
        )
    }

    fn validate_shape(
        field_name: &'static str,
        field: &CpuField<f32>,
        expected: wrf_compute::GridShape,
    ) -> SpecifiedBoundaryGeopotentialResult<()> {
        if field.shape() != expected {
            return Err(SpecifiedBoundaryGeopotentialError::ShapeMismatch {
                field: field_name,
                expected,
                actual: field.shape(),
            });
        }
        Ok(())
    }

    fn validate_coefficients(
        inputs: &SpecifiedBoundaryGeopotentialInputs<'_, CpuField<f32>>,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryGeopotentialResult<()> {
        let expected = region.shape().bottom_top_points();
        Self::validate_coefficient_length(
            "column mass multiplier",
            inputs.column_mass_multiplier,
            expected,
        )?;
        Self::validate_coefficient_length("column mass offset", inputs.column_mass_offset, expected)
    }

    fn validate_coefficient_length(
        coefficient: &'static str,
        values: &[f32],
        expected: usize,
    ) -> SpecifiedBoundaryGeopotentialResult<()> {
        if values.len() != expected {
            return Err(
                SpecifiedBoundaryGeopotentialError::CoefficientLengthMismatch {
                    coefficient,
                    expected,
                    actual: values.len(),
                },
            );
        }
        Ok(())
    }

    fn map_parallel_error(
        error: ParallelExecutionError<Infallible>,
    ) -> SpecifiedBoundaryGeopotentialError {
        match error {
            ParallelExecutionError::Kernel(infallible) => match infallible {},
            ParallelExecutionError::WorkerPanicked => {
                SpecifiedBoundaryGeopotentialError::WorkerPanicked
            }
            ParallelExecutionError::ZeroBlockLength
            | ParallelExecutionError::IncompleteOutputBlock { .. }
            | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
                SpecifiedBoundaryGeopotentialError::SchedulerContractViolated
            }
        }
    }
}
