use std::ops::Range;

use wrf_compute::CpuField;

use super::super::{SpecifiedBoundaryRelaxationBoundaryValues, SpecifiedBoundaryRelaxationInputs};
use crate::SpecifiedBoundaryTendencies;
use crate::specified_boundary_update::geometry::clamp_to_interior;

#[derive(Clone, Copy)]
pub(super) enum SpecifiedBoundaryRelaxationSide {
    South,
    North,
    West,
    East,
}

impl SpecifiedBoundaryRelaxationSide {
    #[inline(always)]
    pub(super) fn boundary_values<'a>(
        self,
        inputs: &'a SpecifiedBoundaryRelaxationInputs<'_, CpuField<f32>>,
    ) -> &'a [f32] {
        let values: &SpecifiedBoundaryRelaxationBoundaryValues<'_, CpuField<f32>> =
            &inputs.boundary_values;
        match self {
            Self::South => values.south.values(),
            Self::North => values.north.values(),
            Self::West => values.west.values(),
            Self::East => values.east.values(),
        }
    }

    #[inline(always)]
    pub(super) fn boundary_tendencies<'a>(
        self,
        inputs: &'a SpecifiedBoundaryRelaxationInputs<'_, CpuField<f32>>,
    ) -> &'a [f32] {
        let tendencies: &SpecifiedBoundaryTendencies<'_, CpuField<f32>> =
            &inputs.boundary_tendencies;
        match self {
            Self::South => tendencies.south.values(),
            Self::North => tendencies.north.values(),
            Self::West => tendencies.west.values(),
            Self::East => tendencies.east.values(),
        }
    }

    #[inline(always)]
    pub(super) fn distance(
        self,
        west_east: usize,
        south_north: usize,
        west_east_domain: &Range<usize>,
        south_north_domain: &Range<usize>,
    ) -> usize {
        match self {
            Self::South => south_north - south_north_domain.start,
            Self::North => south_north_domain.end - 1 - south_north,
            Self::West => west_east - west_east_domain.start,
            Self::East => west_east_domain.end - 1 - west_east,
        }
    }

    #[inline(always)]
    pub(super) const fn line_point(self, west_east: usize, south_north: usize) -> usize {
        match self {
            Self::South | Self::North => west_east,
            Self::West | Self::East => south_north,
        }
    }

    #[inline(always)]
    pub(super) const fn line_points(
        self,
        west_east_points: usize,
        south_north_points: usize,
    ) -> usize {
        match self {
            Self::South | Self::North => west_east_points,
            Self::West | Self::East => south_north_points,
        }
    }

    #[inline(always)]
    pub(super) fn tangential_predecessor(
        self,
        west_east: usize,
        south_north: usize,
        west_east_domain: &Range<usize>,
    ) -> (usize, usize) {
        match self {
            Self::South | Self::North => (
                clamp_to_interior(
                    west_east.saturating_sub(1),
                    west_east_domain.start,
                    west_east_domain.end - 1,
                ),
                south_north,
            ),
            Self::West | Self::East => (west_east, south_north - 1),
        }
    }

    #[inline(always)]
    pub(super) fn tangential_successor(
        self,
        west_east: usize,
        south_north: usize,
        west_east_domain: &Range<usize>,
    ) -> (usize, usize) {
        match self {
            Self::South | Self::North => (
                clamp_to_interior(
                    west_east.saturating_add(1),
                    west_east_domain.start,
                    west_east_domain.end - 1,
                ),
                south_north,
            ),
            Self::West | Self::East => (west_east, south_north + 1),
        }
    }

    #[inline(always)]
    pub(super) const fn outer_neighbor(
        self,
        west_east: usize,
        south_north: usize,
    ) -> (usize, usize) {
        match self {
            Self::South => (west_east, south_north - 1),
            Self::North => (west_east, south_north + 1),
            Self::West => (west_east - 1, south_north),
            Self::East => (west_east + 1, south_north),
        }
    }

    #[inline(always)]
    pub(super) const fn inner_neighbor(
        self,
        west_east: usize,
        south_north: usize,
    ) -> (usize, usize) {
        match self {
            Self::South => (west_east, south_north + 1),
            Self::North => (west_east, south_north - 1),
            Self::West => (west_east + 1, south_north),
            Self::East => (west_east - 1, south_north),
        }
    }
}
