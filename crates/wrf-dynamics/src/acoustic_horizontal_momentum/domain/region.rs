use std::ops::Range;

use wrf_compute::GridShape;

use crate::{
    AcousticHorizontalBoundaryPolicy, AcousticHorizontalMomentumAxis,
    AcousticHorizontalMomentumError, AcousticHorizontalMomentumResult,
};

/// Validated mass domains and C-grid tile for WRF `advance_uv` execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcousticHorizontalMomentumRegion {
    shape: GridShape,
    mass_domain_west_east: Range<usize>,
    mass_domain_south_north: Range<usize>,
    half_level_domain: Range<usize>,
    horizontal_tile_west_east: Range<usize>,
    horizontal_tile_south_north: Range<usize>,
    vertical_tile: Range<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AcousticHorizontalMomentumActiveRanges {
    pub(crate) base_mass_west_east: Range<usize>,
    pub(crate) west_east_tendency_west_east: Range<usize>,
    pub(crate) west_east_pressure_west_east: Range<usize>,
    pub(crate) west_east_south_north: Range<usize>,
    pub(crate) south_north_west_east: Range<usize>,
    pub(crate) south_north_tendency_south_north: Range<usize>,
    pub(crate) south_north_pressure_south_north: Range<usize>,
    pub(crate) half_levels: Range<usize>,
    pub(crate) south_polar_row: Option<usize>,
    pub(crate) north_polar_row: Option<usize>,
}

struct BaseRanges {
    mass_west_east: Range<usize>,
    staggered_west_east: Range<usize>,
    mass_south_north: Range<usize>,
    staggered_south_north: Range<usize>,
    half_levels: Range<usize>,
}

impl AcousticHorizontalMomentumRegion {
    /// Validates mass domains, upper C-grid points, the full-level neighbor,
    /// and horizontal/vertical tile ranges.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        mass_domain_west_east: Range<usize>,
        mass_domain_south_north: Range<usize>,
        half_level_domain: Range<usize>,
        horizontal_tile_west_east: Range<usize>,
        horizontal_tile_south_north: Range<usize>,
        vertical_tile: Range<usize>,
    ) -> AcousticHorizontalMomentumResult<Self> {
        validate_domain_with_upper_point(
            AcousticHorizontalMomentumAxis::WestEast,
            &mass_domain_west_east,
            shape.west_east_points(),
        )?;
        validate_domain_with_upper_point(
            AcousticHorizontalMomentumAxis::SouthNorth,
            &mass_domain_south_north,
            shape.south_north_points(),
        )?;
        validate_domain_with_upper_point(
            AcousticHorizontalMomentumAxis::BottomTop,
            &half_level_domain,
            shape.bottom_top_points(),
        )?;
        validate_tile(
            AcousticHorizontalMomentumAxis::WestEast,
            &horizontal_tile_west_east,
            &mass_domain_west_east,
            shape.west_east_points(),
        )?;
        validate_tile(
            AcousticHorizontalMomentumAxis::SouthNorth,
            &horizontal_tile_south_north,
            &mass_domain_south_north,
            shape.south_north_points(),
        )?;
        validate_tile(
            AcousticHorizontalMomentumAxis::BottomTop,
            &vertical_tile,
            &half_level_domain,
            shape.bottom_top_points(),
        )?;
        Ok(Self {
            shape,
            mass_domain_west_east,
            mass_domain_south_north,
            half_level_domain,
            horizontal_tile_west_east,
            horizontal_tile_south_north,
            vertical_tile,
        })
    }

    /// Returns the common three-dimensional field shape.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    pub(crate) fn active_ranges(
        &self,
        policy: AcousticHorizontalBoundaryPolicy,
    ) -> AcousticHorizontalMomentumResult<AcousticHorizontalMomentumActiveRanges> {
        let west_contact = self.horizontal_tile_west_east.start == self.mass_domain_west_east.start;
        let east_contact = self.horizontal_tile_west_east.end == self.mass_domain_west_east.end + 1;
        let south_contact =
            self.horizontal_tile_south_north.start == self.mass_domain_south_north.start;
        let north_contact =
            self.horizontal_tile_south_north.end == self.mass_domain_south_north.end + 1;
        let BaseRanges {
            mass_west_east: base_mass_west_east,
            staggered_west_east: base_west_east_stagger,
            mass_south_north: base_mass_south_north,
            staggered_south_north: base_south_north_stagger,
            half_levels,
        } = self.base_ranges(policy)?;
        let west_east_tendency_west_east = adjusted_range(
            &base_west_east_stagger,
            west_contact && policy.west.excludes_tendency(),
            east_contact && policy.east.excludes_tendency(),
        );
        let west_east_pressure_west_east = adjusted_range(
            &base_west_east_stagger,
            west_contact && policy.west.excludes_pressure_gradient(),
            east_contact && policy.east.excludes_pressure_gradient(),
        );
        let south_north_tendency_south_north = adjusted_range(
            &base_south_north_stagger,
            south_contact && policy.south.excludes_tendency(),
            north_contact && policy.north.excludes_tendency(),
        );
        let south_north_pressure_south_north = adjusted_range(
            &base_south_north_stagger,
            south_contact && policy.south.excludes_pressure_gradient(),
            north_contact && policy.north.excludes_pressure_gradient(),
        );
        if !west_east_pressure_west_east.is_empty() && west_east_pressure_west_east.start == 0 {
            return Err(AcousticHorizontalMomentumError::MissingLowerNeighbor {
                axis: AcousticHorizontalMomentumAxis::WestEast,
            });
        }
        if !south_north_pressure_south_north.is_empty()
            && south_north_pressure_south_north.start == 0
        {
            return Err(AcousticHorizontalMomentumError::MissingLowerNeighbor {
                axis: AcousticHorizontalMomentumAxis::SouthNorth,
            });
        }
        Ok(AcousticHorizontalMomentumActiveRanges {
            base_mass_west_east: base_mass_west_east.clone(),
            west_east_tendency_west_east,
            west_east_pressure_west_east,
            west_east_south_north: base_mass_south_north.clone(),
            south_north_west_east: base_mass_west_east.clone(),
            south_north_tendency_south_north,
            south_north_pressure_south_north,
            half_levels,
            south_polar_row: (south_contact && policy.south.is_polar())
                .then_some(self.mass_domain_south_north.start),
            north_polar_row: (north_contact && policy.north.is_polar())
                .then_some(self.mass_domain_south_north.end),
        })
    }

    pub(crate) fn half_level_domain(&self) -> Range<usize> {
        self.half_level_domain.clone()
    }

    fn base_ranges(
        &self,
        policy: AcousticHorizontalBoundaryPolicy,
    ) -> AcousticHorizontalMomentumResult<BaseRanges> {
        let Some(width) = policy.relaxation_zone.width() else {
            return Ok(BaseRanges {
                mass_west_east: normalized_range(
                    self.horizontal_tile_west_east.start,
                    self.horizontal_tile_west_east
                        .end
                        .min(self.mass_domain_west_east.end),
                ),
                staggered_west_east: self.horizontal_tile_west_east.clone(),
                mass_south_north: normalized_range(
                    self.horizontal_tile_south_north.start,
                    self.horizontal_tile_south_north
                        .end
                        .min(self.mass_domain_south_north.end),
                ),
                staggered_south_north: self.horizontal_tile_south_north.clone(),
                half_levels: normalized_range(
                    self.vertical_tile.start,
                    self.vertical_tile
                        .end
                        .saturating_sub(1)
                        .min(self.half_level_domain.end),
                ),
            });
        };
        validate_relaxation_width(
            AcousticHorizontalMomentumAxis::WestEast,
            width,
            self.mass_domain_west_east.len(),
        )?;
        validate_relaxation_width(
            AcousticHorizontalMomentumAxis::SouthNorth,
            width,
            self.mass_domain_south_north.len(),
        )?;
        let (mass_west_east, staggered_west_east) = if policy.west_east_periodicity.is_periodic() {
            (
                normalized_range(
                    self.horizontal_tile_west_east.start,
                    self.horizontal_tile_west_east
                        .end
                        .min(self.mass_domain_west_east.end),
                ),
                self.horizontal_tile_west_east.clone(),
            )
        } else {
            (
                normalized_range(
                    self.horizontal_tile_west_east
                        .start
                        .max(self.mass_domain_west_east.start + width),
                    self.horizontal_tile_west_east
                        .end
                        .min(self.mass_domain_west_east.end - width),
                ),
                normalized_range(
                    self.horizontal_tile_west_east
                        .start
                        .max(self.mass_domain_west_east.start + width),
                    self.horizontal_tile_west_east
                        .end
                        .min(self.mass_domain_west_east.end - width + 1),
                ),
            )
        };
        Ok(BaseRanges {
            mass_west_east,
            staggered_west_east,
            mass_south_north: normalized_range(
                self.horizontal_tile_south_north
                    .start
                    .max(self.mass_domain_south_north.start + width),
                self.horizontal_tile_south_north
                    .end
                    .min(self.mass_domain_south_north.end - width),
            ),
            staggered_south_north: normalized_range(
                self.horizontal_tile_south_north
                    .start
                    .max(self.mass_domain_south_north.start + width),
                self.horizontal_tile_south_north
                    .end
                    .min(self.mass_domain_south_north.end - width + 1),
            ),
            half_levels: normalized_range(
                self.vertical_tile.start,
                self.vertical_tile.end.min(self.half_level_domain.end),
            ),
        })
    }
}

fn adjusted_range(range: &Range<usize>, exclude_lower: bool, exclude_upper: bool) -> Range<usize> {
    let start = if exclude_lower {
        (range.start + 1).min(range.end)
    } else {
        range.start
    };
    let end = if exclude_upper {
        range.end.saturating_sub(1).max(start)
    } else {
        range.end
    };
    start..end
}

fn normalized_range(start: usize, end: usize) -> Range<usize> {
    start..end.max(start)
}

fn validate_relaxation_width(
    axis: AcousticHorizontalMomentumAxis,
    width: usize,
    domain_points: usize,
) -> AcousticHorizontalMomentumResult<()> {
    if width > domain_points {
        return Err(AcousticHorizontalMomentumError::RelaxationZoneTooWide {
            axis,
            width,
            domain_points,
        });
    }
    Ok(())
}

fn validate_domain_with_upper_point(
    axis: AcousticHorizontalMomentumAxis,
    range: &Range<usize>,
    field_extent: usize,
) -> AcousticHorizontalMomentumResult<()> {
    if range.start >= range.end {
        return Err(AcousticHorizontalMomentumError::EmptyDomainRange { axis });
    }
    if range.end >= field_extent {
        return Err(AcousticHorizontalMomentumError::MissingUpperStaggerPoint {
            axis,
            boundary_index: range.end,
            field_extent,
        });
    }
    Ok(())
}

fn validate_tile(
    axis: AcousticHorizontalMomentumAxis,
    tile: &Range<usize>,
    domain: &Range<usize>,
    field_extent: usize,
) -> AcousticHorizontalMomentumResult<()> {
    if tile.start >= tile.end {
        return Err(AcousticHorizontalMomentumError::EmptyTileRange { axis });
    }
    if tile.end > field_extent {
        return Err(AcousticHorizontalMomentumError::TileRangeOutOfBounds {
            axis,
            range_end: tile.end,
            field_extent,
        });
    }
    if tile.start < domain.start || tile.end > domain.end + 1 {
        return Err(AcousticHorizontalMomentumError::TileOutsideDomain { axis });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AcousticRelaxationZone, AcousticSouthNorthBoundary, AcousticWestEastBoundary,
        AcousticWestEastPeriodicity,
    };

    fn policy() -> AcousticHorizontalBoundaryPolicy {
        AcousticHorizontalBoundaryPolicy::new(
            AcousticRelaxationZone::Disabled,
            AcousticWestEastPeriodicity::Nonperiodic,
            AcousticWestEastBoundary::Open,
            AcousticWestEastBoundary::Symmetric,
            AcousticSouthNorthBoundary::Polar,
            AcousticSouthNorthBoundary::Open,
        )
    }

    #[test]
    fn derives_distinct_tendency_pressure_and_polar_ranges() {
        let region = AcousticHorizontalMomentumRegion::try_new(
            GridShape::try_new(8, 7, 6).unwrap(),
            1..7,
            1..6,
            1..5,
            1..8,
            1..7,
            1..6,
        )
        .unwrap();

        let ranges = region.active_ranges(policy()).unwrap();

        assert_eq!(ranges.west_east_tendency_west_east, 1..7);
        assert_eq!(ranges.west_east_pressure_west_east, 2..7);
        assert_eq!(ranges.south_north_tendency_south_north, 1..7);
        assert_eq!(ranges.south_north_pressure_south_north, 2..6);
        assert_eq!(ranges.half_levels, 1..5);
        assert_eq!(ranges.south_polar_row, Some(1));
        assert_eq!(ranges.north_polar_row, None);
    }

    #[test]
    fn periodic_relaxation_restores_full_west_east_tile() {
        let region = AcousticHorizontalMomentumRegion::try_new(
            GridShape::try_new(10, 9, 7).unwrap(),
            1..9,
            1..8,
            1..6,
            1..10,
            1..9,
            1..7,
        )
        .unwrap();
        let policy = AcousticHorizontalBoundaryPolicy::new(
            AcousticRelaxationZone::Active { width: 2 },
            AcousticWestEastPeriodicity::Periodic,
            AcousticWestEastBoundary::Closed,
            AcousticWestEastBoundary::Closed,
            AcousticSouthNorthBoundary::Closed,
            AcousticSouthNorthBoundary::Closed,
        );

        let ranges = region.active_ranges(policy).unwrap();

        assert_eq!(ranges.base_mass_west_east, 1..9);
        assert_eq!(ranges.west_east_pressure_west_east, 1..10);
        assert_eq!(ranges.south_north_west_east, 1..9);
        assert_eq!(ranges.west_east_south_north, 3..6);
        assert_eq!(ranges.half_levels, 1..6);
    }
}
