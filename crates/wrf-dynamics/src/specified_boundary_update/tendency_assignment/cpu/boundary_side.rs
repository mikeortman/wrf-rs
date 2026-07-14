use std::ops::Range;

use wrf_compute::CpuField;

use super::super::SpecifiedBoundaryTendencies;

#[derive(Clone, Copy)]
pub(super) enum SpecifiedBoundaryTendencySide {
    South,
    North,
    West,
    East,
}

impl SpecifiedBoundaryTendencySide {
    pub(super) fn values<'a>(
        self,
        tendencies: &'a SpecifiedBoundaryTendencies<'_, CpuField<f32>>,
    ) -> &'a [f32] {
        match self {
            Self::South => tendencies.south.values(),
            Self::North => tendencies.north.values(),
            Self::West => tendencies.west.values(),
            Self::East => tendencies.east.values(),
        }
    }

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

    pub(super) const fn line_point(self, west_east: usize, south_north: usize) -> usize {
        match self {
            Self::South | Self::North => west_east,
            Self::West | Self::East => south_north,
        }
    }

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
}
