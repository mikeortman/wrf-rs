use std::ops::Range;

use wrf_compute::GridShape;

use crate::{
    AcousticVerticalAxis, AcousticVerticalBoundaryPolicy, AcousticVerticalError,
    AcousticVerticalResult,
};

/// Complete-column physical domain and horizontal tile for `advance_w`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcousticVerticalRegion {
    shape: GridShape,
    mass_domain_west_east: Range<usize>,
    mass_domain_south_north: Range<usize>,
    mass_levels: Range<usize>,
    horizontal_tile_west_east: Range<usize>,
    horizontal_tile_south_north: Range<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AcousticVerticalActiveRanges {
    pub(crate) west_east: Range<usize>,
    pub(crate) south_north: Range<usize>,
    pub(crate) mass_levels: Range<usize>,
}

impl AcousticVerticalRegion {
    /// Validates a complete vertical column and horizontal tile.
    ///
    /// `mass_levels` covers the source full-mass levels; its exclusive end is
    /// also the upper staggered vertical-momentum level. `vertical_tile` must
    /// include that upper point because both tridiagonal sweeps span it.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        mass_domain_west_east: Range<usize>,
        mass_domain_south_north: Range<usize>,
        mass_levels: Range<usize>,
        horizontal_tile_west_east: Range<usize>,
        horizontal_tile_south_north: Range<usize>,
        vertical_tile: Range<usize>,
    ) -> AcousticVerticalResult<Self> {
        validate_horizontal_domain(
            AcousticVerticalAxis::WestEast,
            &mass_domain_west_east,
            shape.west_east_points(),
        )?;
        validate_horizontal_domain(
            AcousticVerticalAxis::SouthNorth,
            &mass_domain_south_north,
            shape.south_north_points(),
        )?;
        validate_horizontal_tile(
            AcousticVerticalAxis::WestEast,
            &horizontal_tile_west_east,
            &mass_domain_west_east,
            shape.west_east_points(),
        )?;
        validate_horizontal_tile(
            AcousticVerticalAxis::SouthNorth,
            &horizontal_tile_south_north,
            &mass_domain_south_north,
            shape.south_north_points(),
        )?;
        if mass_levels.len() < 3 {
            return Err(AcousticVerticalError::InsufficientVerticalLevels {
                required: 3,
                actual: mass_levels.len(),
            });
        }
        if mass_levels.end >= shape.bottom_top_points() {
            return Err(AcousticVerticalError::MissingUpperNeighbor {
                axis: AcousticVerticalAxis::BottomTop,
                boundary_index: mass_levels.end,
                field_extent: shape.bottom_top_points(),
            });
        }
        let required_vertical_tile = mass_levels.start..(mass_levels.end + 1);
        if vertical_tile != required_vertical_tile {
            return Err(AcousticVerticalError::IncompleteVerticalColumn {
                expected: required_vertical_tile,
                actual: vertical_tile,
            });
        }
        Ok(Self {
            shape,
            mass_domain_west_east,
            mass_domain_south_north,
            mass_levels,
            horizontal_tile_west_east,
            horizontal_tile_south_north,
        })
    }

    /// Returns the common volume-field shape.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    pub(crate) const fn surface_level(&self) -> usize {
        self.mass_levels.start
    }

    pub(crate) const fn top_level(&self) -> usize {
        self.mass_levels.end
    }

    pub(crate) fn active_ranges(
        &self,
        policy: AcousticVerticalBoundaryPolicy,
    ) -> AcousticVerticalResult<AcousticVerticalActiveRanges> {
        let mut west_east = self.horizontal_tile_west_east.start
            ..self
                .horizontal_tile_west_east
                .end
                .min(self.mass_domain_west_east.end);
        let mut south_north = self.horizontal_tile_south_north.start
            ..self
                .horizontal_tile_south_north
                .end
                .min(self.mass_domain_south_north.end);
        if policy.lateral_domain.excludes_edge_points() {
            if !policy.west_east_periodicity.is_periodic() {
                west_east = clipped_inner_range(&west_east, &self.mass_domain_west_east);
            }
            south_north = clipped_inner_range(&south_north, &self.mass_domain_south_north);
        }
        validate_stencil_neighbors(
            AcousticVerticalAxis::WestEast,
            &west_east,
            self.shape.west_east_points(),
        )?;
        validate_stencil_neighbors(
            AcousticVerticalAxis::SouthNorth,
            &south_north,
            self.shape.south_north_points(),
        )?;
        Ok(AcousticVerticalActiveRanges {
            west_east,
            south_north,
            mass_levels: self.mass_levels.clone(),
        })
    }
}

fn clipped_inner_range(tile: &Range<usize>, domain: &Range<usize>) -> Range<usize> {
    let start = tile.start.max(domain.start + 1);
    let end = tile.end.min(domain.end.saturating_sub(1));
    start..end.max(start)
}

fn validate_horizontal_domain(
    axis: AcousticVerticalAxis,
    range: &Range<usize>,
    field_extent: usize,
) -> AcousticVerticalResult<()> {
    if range.is_empty() {
        return Err(AcousticVerticalError::EmptyDomainRange { axis });
    }
    if range.end >= field_extent {
        return Err(AcousticVerticalError::MissingUpperNeighbor {
            axis,
            boundary_index: range.end,
            field_extent,
        });
    }
    Ok(())
}

fn validate_horizontal_tile(
    axis: AcousticVerticalAxis,
    tile: &Range<usize>,
    domain: &Range<usize>,
    field_extent: usize,
) -> AcousticVerticalResult<()> {
    if tile.is_empty() {
        return Err(AcousticVerticalError::EmptyTileRange { axis });
    }
    if tile.end > field_extent {
        return Err(AcousticVerticalError::TileRangeOutOfBounds {
            axis,
            range_end: tile.end,
            field_extent,
        });
    }
    if tile.start < domain.start || tile.end > domain.end + 1 {
        return Err(AcousticVerticalError::TileOutsideDomain { axis });
    }
    Ok(())
}

fn validate_stencil_neighbors(
    axis: AcousticVerticalAxis,
    active: &Range<usize>,
    field_extent: usize,
) -> AcousticVerticalResult<()> {
    if active.is_empty() {
        return Ok(());
    }
    if active.start == 0 {
        return Err(AcousticVerticalError::MissingLowerNeighbor { axis });
    }
    if active.end >= field_extent {
        return Err(AcousticVerticalError::MissingUpperNeighbor {
            axis,
            boundary_index: active.end,
            field_extent,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AcousticVerticalLateralDomain, AcousticVerticalWestEastPeriodicity};

    #[test]
    fn nested_periodic_policy_restores_only_west_east_edges() {
        let ranges = region()
            .active_ranges(AcousticVerticalBoundaryPolicy::new(
                AcousticVerticalLateralDomain::SpecifiedOrNested,
                AcousticVerticalWestEastPeriodicity::Periodic,
            ))
            .unwrap();

        assert_eq!(ranges.west_east, 1..5);
        assert_eq!(ranges.south_north, 2..4);
        assert_eq!(ranges.mass_levels, 1..5);
    }

    #[test]
    fn rejects_partial_vertical_column() {
        assert_eq!(
            AcousticVerticalRegion::try_new(
                GridShape::try_new(6, 6, 6).unwrap(),
                1..5,
                1..5,
                1..5,
                1..6,
                1..6,
                2..6,
            ),
            Err(AcousticVerticalError::IncompleteVerticalColumn {
                expected: 1..6,
                actual: 2..6,
            })
        );
    }

    fn region() -> AcousticVerticalRegion {
        AcousticVerticalRegion::try_new(
            GridShape::try_new(6, 6, 6).unwrap(),
            1..5,
            1..5,
            1..5,
            1..6,
            1..6,
            1..6,
        )
        .unwrap()
    }
}
