use std::convert::Infallible;
use std::ops::Range;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, ParallelExecutionError};

use super::horizontal_boundary_side::HorizontalBoundarySide;
use crate::specified_boundary_update::geometry::SpecifiedBoundaryRanges;
use crate::{
    SpecifiedBoundaryUpdateAxis, SpecifiedBoundaryUpdateRegion,
    SpecifiedBoundaryWestEastPeriodicity, SpecifiedBoundaryZeroGradientError,
    SpecifiedBoundaryZeroGradientParameters, SpecifiedBoundaryZeroGradientResult,
};

pub(super) struct SpecifiedBoundaryZeroGradientCpuKernel<'a> {
    field: &'a mut CpuField<f32>,
    specified_zone_width: usize,
    periodic_west_east: bool,
    bottom_top_range: Range<usize>,
    boundary_ranges: SpecifiedBoundaryRanges,
}

impl<'a> SpecifiedBoundaryZeroGradientCpuKernel<'a> {
    pub(super) fn try_new(
        field: &'a mut CpuField<f32>,
        parameters: SpecifiedBoundaryZeroGradientParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryZeroGradientResult<Self> {
        if field.shape() != region.shape() {
            return Err(SpecifiedBoundaryZeroGradientError::ShapeMismatch {
                expected: region.shape(),
                actual: field.shape(),
            });
        }
        let boundary_ranges = SpecifiedBoundaryRanges::new(
            region.active_ranges(),
            parameters.specified_zone_width(),
            west_east_periodicity,
        );
        Self::validate_interior_sources(
            &boundary_ranges,
            parameters.specified_zone_width(),
            west_east_periodicity,
        )?;
        let active_bottom_top_start = region.active_ranges().bottom_top.start;
        let half_level_domain_end = region.mass_domains().2.end;
        let bottom_top_end =
            half_level_domain_end + usize::from(region.location().has_upper_vertical_point());
        Ok(Self {
            field,
            specified_zone_width: parameters.specified_zone_width(),
            periodic_west_east: west_east_periodicity.is_periodic(),
            bottom_top_range: active_bottom_top_start..bottom_top_end,
            boundary_ranges,
        })
    }

    pub(super) fn execute(
        mut self,
        backend: &CpuBackend,
    ) -> SpecifiedBoundaryZeroGradientResult<()> {
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

    fn apply_south(&mut self, backend: &CpuBackend) -> SpecifiedBoundaryZeroGradientResult<()> {
        let domain = self.boundary_ranges.south_north_domain();
        let source_south_north = domain.start + self.specified_zone_width;
        let west_east_points = self.field.shape().west_east_points();
        let plane_length = self.plane_length();
        let split_index = source_south_north * plane_length;
        let (before_source, source_and_after) = self.field.values_mut().split_at_mut(split_index);
        let source_plane = &source_and_after[..plane_length];
        let destination_start = domain.start * plane_length;
        let destination = &mut before_source[destination_start..];
        Self::apply_horizontal_boundary_planes(
            backend,
            destination,
            source_plane,
            plane_length,
            west_east_points,
            domain.start,
            &self.boundary_ranges,
            HorizontalBoundarySide::South,
            self.periodic_west_east,
            self.specified_zone_width,
            self.bottom_top_range.clone(),
        )
    }

    fn apply_north(&mut self, backend: &CpuBackend) -> SpecifiedBoundaryZeroGradientResult<()> {
        let domain = self.boundary_ranges.south_north_domain();
        let source_south_north = domain.end - 1 - self.specified_zone_width;
        let west_east_points = self.field.shape().west_east_points();
        let plane_length = self.plane_length();
        let split_index = (source_south_north + 1) * plane_length;
        let (through_source, after_source) = self.field.values_mut().split_at_mut(split_index);
        let source_start = source_south_north * plane_length;
        let source_plane = &through_source[source_start..source_start + plane_length];
        Self::apply_horizontal_boundary_planes(
            backend,
            after_source,
            source_plane,
            plane_length,
            west_east_points,
            source_south_north + 1,
            &self.boundary_ranges,
            HorizontalBoundarySide::North,
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
        plane_length: usize,
        west_east_points: usize,
        first_south_north: usize,
        boundary_ranges: &SpecifiedBoundaryRanges,
        side: HorizontalBoundarySide,
        periodic_west_east: bool,
        specified_zone_width: usize,
        bottom_top_range: Range<usize>,
    ) -> SpecifiedBoundaryZeroGradientResult<()> {
        let west_east_domain = boundary_ranges.west_east_domain();
        backend
            .try_for_each_output_block::<_, Infallible, _>(
                destination,
                plane_length,
                |local_south_north, output_plane| {
                    let south_north = first_south_north + local_south_north;
                    let row_ranges = boundary_ranges.ranges_for_row(south_north);
                    let selected_range = match side {
                        HorizontalBoundarySide::South => row_ranges.south,
                        HorizontalBoundarySide::North => row_ranges.north,
                    };
                    let Some(west_east_range) = selected_range else {
                        return Ok(());
                    };
                    for bottom_top in bottom_top_range.clone() {
                        let row_start = bottom_top * west_east_points;
                        for west_east in west_east_range.clone() {
                            let source_west_east = if periodic_west_east {
                                west_east
                            } else {
                                source_coordinate(
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
        let shape = self.field.shape();
        let west_east_points = shape.west_east_points();
        let bottom_top_points = shape.bottom_top_points();
        let west_east_domain = self.boundary_ranges.west_east_domain();
        let south_north_domain = self.boundary_ranges.south_north_domain();
        let west_source = west_east_domain.start + self.specified_zone_width;
        let east_source = west_east_domain.end - 1 - self.specified_zone_width;
        let south_source = south_north_domain.start + self.specified_zone_width;
        let north_source = south_north_domain.end - 1 - self.specified_zone_width;
        let values = self.field.values_mut();
        for south_north in 0..shape.south_north_points() {
            let row_ranges = self.boundary_ranges.ranges_for_row(south_north);
            let source_south_north = source_coordinate(south_north, south_source, north_source);
            for (destination_range, source_west_east) in [
                (row_ranges.west, west_source),
                (row_ranges.east, east_source),
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
                        let destination_index = volume_index(
                            west_east,
                            bottom_top,
                            south_north,
                            west_east_points,
                            bottom_top_points,
                        );
                        values[destination_index] = source_value;
                    }
                }
            }
        }
    }

    fn plane_length(&self) -> usize {
        self.field.shape().west_east_points() * self.field.shape().bottom_top_points()
    }

    fn validate_interior_sources(
        boundary_ranges: &SpecifiedBoundaryRanges,
        specified_zone_width: usize,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
    ) -> SpecifiedBoundaryZeroGradientResult<()> {
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
    ) -> SpecifiedBoundaryZeroGradientResult<()> {
        let domain_points = domain.len();
        if specified_zone_width.saturating_mul(2) >= domain_points {
            return Err(SpecifiedBoundaryZeroGradientError::MissingInteriorSource {
                axis,
                specified_zone_width,
                domain_points,
            });
        }
        Ok(())
    }

    fn map_parallel_error(
        error: ParallelExecutionError<Infallible>,
    ) -> SpecifiedBoundaryZeroGradientError {
        match error {
            ParallelExecutionError::Kernel(infallible) => match infallible {},
            ParallelExecutionError::WorkerPanicked => {
                SpecifiedBoundaryZeroGradientError::WorkerPanicked
            }
            ParallelExecutionError::ZeroBlockLength
            | ParallelExecutionError::IncompleteOutputBlock { .. }
            | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
                SpecifiedBoundaryZeroGradientError::SchedulerContractViolated
            }
        }
    }
}

fn source_coordinate(value: usize, lower: usize, upper: usize) -> usize {
    value.max(lower).min(upper)
}

fn volume_index(
    west_east: usize,
    bottom_top: usize,
    south_north: usize,
    west_east_points: usize,
    bottom_top_points: usize,
) -> usize {
    west_east + west_east_points * (bottom_top + bottom_top_points * south_north)
}
