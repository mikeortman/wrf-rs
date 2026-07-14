use std::convert::Infallible;
use std::ops::Range;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, ParallelExecutionError};

use super::south_north_side::SouthNorthSide;
use super::west_east_side::WestEastSide;
use crate::specified_boundary_update::geometry::{
    SpecifiedBoundaryRanges, clamp_to_interior, volume_index,
};
use crate::{
    SpecifiedBoundaryFlowError, SpecifiedBoundaryFlowField, SpecifiedBoundaryFlowInputs,
    SpecifiedBoundaryFlowParameters, SpecifiedBoundaryFlowRegion, SpecifiedBoundaryFlowResult,
    SpecifiedBoundaryUpdateAxis, SpecifiedBoundaryWestEastPeriodicity,
};

pub(super) struct SpecifiedBoundaryFlowCpuKernel<'a> {
    scalar: &'a mut CpuField<f32>,
    west_east_velocity: &'a CpuField<f32>,
    south_north_velocity: &'a CpuField<f32>,
    specified_zone_width: usize,
    periodic_west_east: bool,
    bottom_top_range: Range<usize>,
    boundary_ranges: SpecifiedBoundaryRanges,
}

impl<'a> SpecifiedBoundaryFlowCpuKernel<'a> {
    pub(super) fn try_new(
        scalar: &'a mut CpuField<f32>,
        inputs: SpecifiedBoundaryFlowInputs<'a, CpuField<f32>>,
        parameters: SpecifiedBoundaryFlowParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryFlowRegion,
    ) -> SpecifiedBoundaryFlowResult<Self> {
        Self::validate_shape(SpecifiedBoundaryFlowField::Scalar, scalar, region.shape())?;
        Self::validate_shape(
            SpecifiedBoundaryFlowField::WestEastVelocity,
            inputs.west_east_velocity,
            region.shape(),
        )?;
        Self::validate_shape(
            SpecifiedBoundaryFlowField::SouthNorthVelocity,
            inputs.south_north_velocity,
            region.shape(),
        )?;
        let inner_region = region.inner();
        let boundary_ranges = SpecifiedBoundaryRanges::new(
            inner_region.active_ranges(),
            parameters.specified_zone_width(),
            west_east_periodicity,
        );
        Self::validate_interior_sources(
            &boundary_ranges,
            parameters.specified_zone_width(),
            west_east_periodicity,
        )?;
        Self::validate_velocity_neighbors(&boundary_ranges, region.shape())?;
        let active_bottom_top_start = inner_region.active_ranges().bottom_top.start;
        let bottom_top_end = inner_region.mass_domains().2.end;
        Ok(Self {
            scalar,
            west_east_velocity: inputs.west_east_velocity,
            south_north_velocity: inputs.south_north_velocity,
            specified_zone_width: parameters.specified_zone_width(),
            periodic_west_east: west_east_periodicity.is_periodic(),
            bottom_top_range: active_bottom_top_start..bottom_top_end,
            boundary_ranges,
        })
    }

    pub(super) fn execute(mut self, backend: &CpuBackend) -> SpecifiedBoundaryFlowResult<()> {
        if self.specified_zone_width == 0 {
            return Ok(());
        }
        self.apply_south(backend)?;
        self.apply_north(backend)?;
        if self.periodic_west_east {
            return Ok(());
        }
        self.apply_west_east();
        Ok(())
    }

    fn apply_south(&mut self, backend: &CpuBackend) -> SpecifiedBoundaryFlowResult<()> {
        let domain = self.boundary_ranges.south_north_domain();
        let source_south_north = domain.start + self.specified_zone_width;
        let west_east_points = self.scalar.shape().west_east_points();
        let bottom_top_points = self.scalar.shape().bottom_top_points();
        let plane_length = self.plane_length();
        let split_index = source_south_north * plane_length;
        let (before_source, source_and_after) = self.scalar.values_mut().split_at_mut(split_index);
        let source_plane = &source_and_after[..plane_length];
        let destination_start = domain.start * plane_length;
        let destination = &mut before_source[destination_start..];
        Self::apply_horizontal_boundary_planes(
            backend,
            destination,
            source_plane,
            self.south_north_velocity.values(),
            plane_length,
            west_east_points,
            bottom_top_points,
            domain.start,
            &self.boundary_ranges,
            SouthNorthSide::South,
            self.periodic_west_east,
            self.specified_zone_width,
            self.bottom_top_range.clone(),
        )
    }

    fn apply_north(&mut self, backend: &CpuBackend) -> SpecifiedBoundaryFlowResult<()> {
        let domain = self.boundary_ranges.south_north_domain();
        let source_south_north = domain.end - 1 - self.specified_zone_width;
        let west_east_points = self.scalar.shape().west_east_points();
        let bottom_top_points = self.scalar.shape().bottom_top_points();
        let plane_length = self.plane_length();
        let split_index = (source_south_north + 1) * plane_length;
        let (through_source, after_source) = self.scalar.values_mut().split_at_mut(split_index);
        let source_start = source_south_north * plane_length;
        let source_plane = &through_source[source_start..source_start + plane_length];
        Self::apply_horizontal_boundary_planes(
            backend,
            after_source,
            source_plane,
            self.south_north_velocity.values(),
            plane_length,
            west_east_points,
            bottom_top_points,
            source_south_north + 1,
            &self.boundary_ranges,
            SouthNorthSide::North,
            self.periodic_west_east,
            self.specified_zone_width,
            self.bottom_top_range.clone(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_horizontal_boundary_planes(
        backend: &CpuBackend,
        destination: &mut [f32],
        source_plane: &[f32],
        south_north_velocity: &[f32],
        plane_length: usize,
        west_east_points: usize,
        bottom_top_points: usize,
        first_south_north: usize,
        boundary_ranges: &SpecifiedBoundaryRanges,
        side: SouthNorthSide,
        periodic_west_east: bool,
        specified_zone_width: usize,
        bottom_top_range: Range<usize>,
    ) -> SpecifiedBoundaryFlowResult<()> {
        let west_east_domain = boundary_ranges.west_east_domain();
        backend
            .try_for_each_output_block::<_, Infallible, _>(
                destination,
                plane_length,
                |local_south_north, output_plane| {
                    let south_north = first_south_north + local_south_north;
                    let row_ranges = boundary_ranges.ranges_for_row(south_north);
                    let selected_range = match side {
                        SouthNorthSide::South => row_ranges.south,
                        SouthNorthSide::North => row_ranges.north,
                    };
                    let Some(west_east_range) = selected_range else {
                        return Ok(());
                    };
                    let velocity_south_north = match side {
                        SouthNorthSide::South => south_north,
                        SouthNorthSide::North => south_north + 1,
                    };
                    for bottom_top in bottom_top_range.clone() {
                        let row_start = bottom_top * west_east_points;
                        for west_east in west_east_range.clone() {
                            let velocity_index = volume_index(
                                west_east,
                                bottom_top,
                                velocity_south_north,
                                west_east_points,
                                bottom_top_points,
                            );
                            let outflow = match side {
                                SouthNorthSide::South => south_north_velocity[velocity_index] < 0.0,
                                SouthNorthSide::North => south_north_velocity[velocity_index] > 0.0,
                            };
                            if !outflow {
                                output_plane[row_start + west_east] = 0.0;
                                continue;
                            }
                            let source_west_east = if periodic_west_east {
                                west_east
                            } else {
                                clamp_to_interior(
                                    west_east,
                                    west_east_domain.start + specified_zone_width,
                                    west_east_domain.end - 1 - specified_zone_width,
                                )
                            };
                            output_plane[row_start + west_east] =
                                source_plane[row_start + source_west_east];
                        }
                    }
                    Ok(())
                },
            )
            .map_err(Self::map_parallel_error)
    }

    fn apply_west_east(&mut self) {
        let shape = self.scalar.shape();
        let west_east_points = shape.west_east_points();
        let bottom_top_points = shape.bottom_top_points();
        let west_east_domain = self.boundary_ranges.west_east_domain();
        let south_north_domain = self.boundary_ranges.south_north_domain();
        let west_source = west_east_domain.start + self.specified_zone_width;
        let east_source = west_east_domain.end - 1 - self.specified_zone_width;
        let south_source = south_north_domain.start + self.specified_zone_width;
        let north_source = south_north_domain.end - 1 - self.specified_zone_width;
        let west_east_velocity = self.west_east_velocity.values();
        let values = self.scalar.values_mut();
        for south_north in 0..shape.south_north_points() {
            let row_ranges = self.boundary_ranges.ranges_for_row(south_north);
            let source_south_north = clamp_to_interior(south_north, south_source, north_source);
            for (side, destination_range, source_west_east) in [
                (WestEastSide::West, row_ranges.west, west_source),
                (WestEastSide::East, row_ranges.east, east_source),
            ] {
                let Some(destination_range) = destination_range else {
                    continue;
                };
                for bottom_top in self.bottom_top_range.clone() {
                    let source_index = volume_index(
                        source_west_east,
                        bottom_top,
                        source_south_north,
                        west_east_points,
                        bottom_top_points,
                    );
                    let source_value = values[source_index];
                    for west_east in destination_range.clone() {
                        let velocity_west_east = match side {
                            WestEastSide::West => west_east,
                            WestEastSide::East => west_east + 1,
                        };
                        let velocity_index = volume_index(
                            velocity_west_east,
                            bottom_top,
                            south_north,
                            west_east_points,
                            bottom_top_points,
                        );
                        let outflow = match side {
                            WestEastSide::West => west_east_velocity[velocity_index] < 0.0,
                            WestEastSide::East => west_east_velocity[velocity_index] > 0.0,
                        };
                        let destination_index = volume_index(
                            west_east,
                            bottom_top,
                            south_north,
                            west_east_points,
                            bottom_top_points,
                        );
                        values[destination_index] = if outflow { source_value } else { 0.0 };
                    }
                }
            }
        }
    }

    fn plane_length(&self) -> usize {
        self.scalar.shape().west_east_points() * self.scalar.shape().bottom_top_points()
    }

    fn validate_shape(
        field_role: SpecifiedBoundaryFlowField,
        field: &CpuField<f32>,
        expected: wrf_compute::GridShape,
    ) -> SpecifiedBoundaryFlowResult<()> {
        let actual = field.shape();
        if actual == expected {
            return Ok(());
        }
        Err(SpecifiedBoundaryFlowError::ShapeMismatch {
            field: field_role,
            expected,
            actual,
        })
    }

    fn validate_velocity_neighbors(
        boundary_ranges: &SpecifiedBoundaryRanges,
        shape: wrf_compute::GridShape,
    ) -> SpecifiedBoundaryFlowResult<()> {
        if boundary_ranges.touches_north_boundary() {
            let required_index = boundary_ranges.south_north_domain().end;
            if required_index >= shape.south_north_points() {
                return Err(SpecifiedBoundaryFlowError::MissingVelocityNeighbor {
                    axis: SpecifiedBoundaryUpdateAxis::SouthNorth,
                    required_index,
                    extent: shape.south_north_points(),
                });
            }
        }
        if boundary_ranges.touches_east_boundary() {
            let required_index = boundary_ranges.west_east_domain().end;
            if required_index >= shape.west_east_points() {
                return Err(SpecifiedBoundaryFlowError::MissingVelocityNeighbor {
                    axis: SpecifiedBoundaryUpdateAxis::WestEast,
                    required_index,
                    extent: shape.west_east_points(),
                });
            }
        }
        Ok(())
    }

    fn validate_interior_sources(
        boundary_ranges: &SpecifiedBoundaryRanges,
        specified_zone_width: usize,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
    ) -> SpecifiedBoundaryFlowResult<()> {
        if specified_zone_width == 0 {
            return Ok(());
        }
        Self::validate_axis_source(
            SpecifiedBoundaryUpdateAxis::SouthNorth,
            boundary_ranges.south_north_domain(),
            specified_zone_width,
        )?;
        if west_east_periodicity.is_periodic() {
            return Ok(());
        }
        Self::validate_axis_source(
            SpecifiedBoundaryUpdateAxis::WestEast,
            boundary_ranges.west_east_domain(),
            specified_zone_width,
        )
    }

    fn validate_axis_source(
        axis: SpecifiedBoundaryUpdateAxis,
        domain: Range<usize>,
        specified_zone_width: usize,
    ) -> SpecifiedBoundaryFlowResult<()> {
        let domain_points = domain.len();
        if specified_zone_width.saturating_mul(2) >= domain_points {
            return Err(SpecifiedBoundaryFlowError::MissingInteriorSource {
                axis,
                specified_zone_width,
                domain_points,
            });
        }
        Ok(())
    }

    fn map_parallel_error(error: ParallelExecutionError<Infallible>) -> SpecifiedBoundaryFlowError {
        match error {
            ParallelExecutionError::Kernel(infallible) => match infallible {},
            ParallelExecutionError::WorkerPanicked => SpecifiedBoundaryFlowError::WorkerPanicked,
            ParallelExecutionError::ZeroBlockLength
            | ParallelExecutionError::IncompleteOutputBlock { .. }
            | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
                SpecifiedBoundaryFlowError::SchedulerContractViolated
            }
        }
    }
}
