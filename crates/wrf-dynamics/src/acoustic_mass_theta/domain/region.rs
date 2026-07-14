use std::ops::Range;

use wrf_compute::GridShape;

use crate::{
    AcousticMassThetaAxis, AcousticMassThetaBoundaryPolicy, AcousticMassThetaError,
    AcousticMassThetaResult,
};

/// Validated mass domains and complete-column tile for WRF `advance_mu_t`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcousticMassThetaRegion {
    shape: GridShape,
    mass_domain_west_east: Range<usize>,
    mass_domain_south_north: Range<usize>,
    half_level_domain: Range<usize>,
    horizontal_tile_west_east: Range<usize>,
    horizontal_tile_south_north: Range<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AcousticMassThetaActiveRanges {
    pub(crate) west_east: Range<usize>,
    pub(crate) south_north: Range<usize>,
    pub(crate) half_levels: Range<usize>,
}

impl AcousticMassThetaRegion {
    /// Validates horizontal mass domains, upper U/V neighbors, and the complete column.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        mass_domain_west_east: Range<usize>,
        mass_domain_south_north: Range<usize>,
        half_level_domain: Range<usize>,
        horizontal_tile_west_east: Range<usize>,
        horizontal_tile_south_north: Range<usize>,
        vertical_tile: Range<usize>,
    ) -> AcousticMassThetaResult<Self> {
        validate_domain_with_upper_point(
            AcousticMassThetaAxis::WestEast,
            &mass_domain_west_east,
            shape.west_east_points(),
        )?;
        validate_domain_with_upper_point(
            AcousticMassThetaAxis::SouthNorth,
            &mass_domain_south_north,
            shape.south_north_points(),
        )?;
        validate_domain_with_upper_point(
            AcousticMassThetaAxis::BottomTop,
            &half_level_domain,
            shape.bottom_top_points(),
        )?;
        validate_horizontal_tile(
            AcousticMassThetaAxis::WestEast,
            &horizontal_tile_west_east,
            &mass_domain_west_east,
            shape.west_east_points(),
        )?;
        validate_horizontal_tile(
            AcousticMassThetaAxis::SouthNorth,
            &horizontal_tile_south_north,
            &mass_domain_south_north,
            shape.south_north_points(),
        )?;
        let required_vertical_tile = half_level_domain.start..(half_level_domain.end + 1);
        if vertical_tile != required_vertical_tile {
            return Err(AcousticMassThetaError::IncompleteVerticalColumn {
                expected: required_vertical_tile,
                actual: vertical_tile,
            });
        }
        Ok(Self {
            shape,
            mass_domain_west_east,
            mass_domain_south_north,
            half_level_domain,
            horizontal_tile_west_east,
            horizontal_tile_south_north,
        })
    }

    /// Returns the common three-dimensional storage shape.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    pub(crate) fn active_ranges(
        &self,
        policy: AcousticMassThetaBoundaryPolicy,
    ) -> AcousticMassThetaResult<AcousticMassThetaActiveRanges> {
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
        if !west_east.is_empty() && west_east.start == 0 {
            return Err(AcousticMassThetaError::MissingLowerNeighbor {
                axis: AcousticMassThetaAxis::WestEast,
            });
        }
        if !south_north.is_empty() && south_north.start == 0 {
            return Err(AcousticMassThetaError::MissingLowerNeighbor {
                axis: AcousticMassThetaAxis::SouthNorth,
            });
        }
        Ok(AcousticMassThetaActiveRanges {
            west_east,
            south_north,
            half_levels: self.half_level_domain.clone(),
        })
    }
}

fn clipped_inner_range(tile: &Range<usize>, domain: &Range<usize>) -> Range<usize> {
    let start = tile.start.max(domain.start + 1);
    let end = tile.end.min(domain.end.saturating_sub(1));
    start..end.max(start)
}

fn validate_domain_with_upper_point(
    axis: AcousticMassThetaAxis,
    range: &Range<usize>,
    field_extent: usize,
) -> AcousticMassThetaResult<()> {
    if range.is_empty() {
        return Err(AcousticMassThetaError::EmptyDomainRange { axis });
    }
    if range.end >= field_extent {
        return Err(AcousticMassThetaError::MissingUpperNeighbor {
            axis,
            boundary_index: range.end,
            field_extent,
        });
    }
    Ok(())
}

fn validate_horizontal_tile(
    axis: AcousticMassThetaAxis,
    tile: &Range<usize>,
    domain: &Range<usize>,
    field_extent: usize,
) -> AcousticMassThetaResult<()> {
    if tile.is_empty() {
        return Err(AcousticMassThetaError::EmptyTileRange { axis });
    }
    if tile.end > field_extent {
        return Err(AcousticMassThetaError::TileRangeOutOfBounds {
            axis,
            range_end: tile.end,
            field_extent,
        });
    }
    if tile.start < domain.start || tile.end > domain.end + 1 {
        return Err(AcousticMassThetaError::TileOutsideDomain { axis });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AcousticMassThetaLateralDomain, AcousticMassThetaWestEastPeriodicity};

    #[test]
    fn nested_periodic_policy_restores_only_the_west_east_edges() {
        let region = region();
        let ranges = region
            .active_ranges(AcousticMassThetaBoundaryPolicy::new(
                AcousticMassThetaLateralDomain::SpecifiedOrNested,
                AcousticMassThetaWestEastPeriodicity::Periodic,
            ))
            .unwrap();

        assert_eq!(ranges.west_east, 1..5);
        assert_eq!(ranges.south_north, 2..4);
        assert_eq!(ranges.half_levels, 1..5);
    }

    #[test]
    fn rejects_a_partial_vertical_column() {
        assert_eq!(
            AcousticMassThetaRegion::try_new(
                GridShape::try_new(6, 6, 6).unwrap(),
                1..5,
                1..5,
                1..5,
                1..6,
                1..6,
                2..6,
            ),
            Err(AcousticMassThetaError::IncompleteVerticalColumn {
                expected: 1..6,
                actual: 2..6,
            })
        );
    }

    fn region() -> AcousticMassThetaRegion {
        AcousticMassThetaRegion::try_new(
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
