use std::fmt;
use std::ops::Range;

use wrf_compute::GridShape;

use crate::{
    SpecifiedBoundaryFieldLocation, SpecifiedBoundaryUpdateError, SpecifiedBoundaryUpdateResult,
};

/// Logical axis used in specified-boundary validation errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpecifiedBoundaryUpdateAxis {
    /// West–east storage axis.
    WestEast,
    /// South–north storage axis.
    SouthNorth,
    /// Bottom–top storage axis.
    BottomTop,
}

impl fmt::Display for SpecifiedBoundaryUpdateAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WestEast => formatter.write_str("west-east"),
            Self::SouthNorth => formatter.write_str("south-north"),
            Self::BottomTop => formatter.write_str("bottom-top"),
        }
    }
}

/// Validated physical domain and one location-specific boundary tile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpecifiedBoundaryUpdateRegion {
    shape: GridShape,
    location: SpecifiedBoundaryFieldLocation,
    mass_domain_west_east: Range<usize>,
    mass_domain_south_north: Range<usize>,
    half_level_domain: Range<usize>,
    active_west_east: Range<usize>,
    active_south_north: Range<usize>,
    active_bottom_top: Range<usize>,
    effective_west_east_domain: Range<usize>,
    effective_south_north_domain: Range<usize>,
}

impl SpecifiedBoundaryUpdateRegion {
    /// Validates physical domains, location-specific staggers, and tile ranges.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        location: SpecifiedBoundaryFieldLocation,
        mass_domain_west_east: Range<usize>,
        mass_domain_south_north: Range<usize>,
        half_level_domain: Range<usize>,
        tile_west_east: Range<usize>,
        tile_south_north: Range<usize>,
        tile_bottom_top: Range<usize>,
    ) -> SpecifiedBoundaryUpdateResult<Self> {
        validate_range(
            SpecifiedBoundaryUpdateAxis::WestEast,
            &mass_domain_west_east,
            shape.west_east_points(),
        )?;
        validate_range(
            SpecifiedBoundaryUpdateAxis::SouthNorth,
            &mass_domain_south_north,
            shape.south_north_points(),
        )?;
        validate_range(
            SpecifiedBoundaryUpdateAxis::BottomTop,
            &half_level_domain,
            shape.bottom_top_points(),
        )?;
        let effective_west_east_domain = extend_upper_stagger(
            SpecifiedBoundaryUpdateAxis::WestEast,
            mass_domain_west_east.clone(),
            shape.west_east_points(),
            location.has_upper_west_east_point(),
        )?;
        let effective_south_north_domain = extend_upper_stagger(
            SpecifiedBoundaryUpdateAxis::SouthNorth,
            mass_domain_south_north.clone(),
            shape.south_north_points(),
            location.has_upper_south_north_point(),
        )?;
        let effective_bottom_top_domain = extend_upper_stagger(
            SpecifiedBoundaryUpdateAxis::BottomTop,
            half_level_domain.clone(),
            shape.bottom_top_points(),
            location.has_upper_vertical_point(),
        )?;
        validate_tile(
            SpecifiedBoundaryUpdateAxis::WestEast,
            &tile_west_east,
            &mass_domain_west_east,
            shape.west_east_points(),
        )?;
        validate_tile(
            SpecifiedBoundaryUpdateAxis::SouthNorth,
            &tile_south_north,
            &mass_domain_south_north,
            shape.south_north_points(),
        )?;
        validate_tile(
            SpecifiedBoundaryUpdateAxis::BottomTop,
            &tile_bottom_top,
            &half_level_domain,
            shape.bottom_top_points(),
        )?;
        Ok(Self {
            shape,
            location,
            mass_domain_west_east,
            mass_domain_south_north,
            half_level_domain,
            active_west_east: tile_west_east.start
                ..tile_west_east.end.min(effective_west_east_domain.end),
            active_south_north: tile_south_north.start
                ..tile_south_north.end.min(effective_south_north_domain.end),
            active_bottom_top: tile_bottom_top.start
                ..tile_bottom_top.end.min(effective_bottom_top_domain.end),
            effective_west_east_domain,
            effective_south_north_domain,
        })
    }

    /// Returns the common field shape checked before mutation.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    /// Returns the field location captured by validation.
    pub const fn location(&self) -> SpecifiedBoundaryFieldLocation {
        self.location
    }

    pub(crate) fn active_ranges(&self) -> SpecifiedBoundaryActiveRanges {
        SpecifiedBoundaryActiveRanges {
            west_east: self.active_west_east.clone(),
            south_north: self.active_south_north.clone(),
            bottom_top: self.active_bottom_top.clone(),
            effective_west_east_domain: self.effective_west_east_domain.clone(),
            effective_south_north_domain: self.effective_south_north_domain.clone(),
        }
    }

    /// Returns the unstaggered physical domain used to derive this region.
    pub fn mass_domains(&self) -> (&Range<usize>, &Range<usize>, &Range<usize>) {
        (
            &self.mass_domain_west_east,
            &self.mass_domain_south_north,
            &self.half_level_domain,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SpecifiedBoundaryActiveRanges {
    pub(crate) west_east: Range<usize>,
    pub(crate) south_north: Range<usize>,
    pub(crate) bottom_top: Range<usize>,
    pub(crate) effective_west_east_domain: Range<usize>,
    pub(crate) effective_south_north_domain: Range<usize>,
}

fn validate_range(
    axis: SpecifiedBoundaryUpdateAxis,
    range: &Range<usize>,
    extent: usize,
) -> SpecifiedBoundaryUpdateResult<()> {
    if range.is_empty() || range.end > extent {
        return Err(SpecifiedBoundaryUpdateError::InvalidRange {
            axis,
            range: range.clone(),
            extent,
        });
    }
    Ok(())
}

fn extend_upper_stagger(
    axis: SpecifiedBoundaryUpdateAxis,
    range: Range<usize>,
    extent: usize,
    extend: bool,
) -> SpecifiedBoundaryUpdateResult<Range<usize>> {
    if !extend {
        return Ok(range);
    }
    let required_end =
        range
            .end
            .checked_add(1)
            .ok_or(SpecifiedBoundaryUpdateError::MissingUpperStagger {
                axis,
                required_end: usize::MAX,
                extent,
            })?;
    if required_end > extent {
        return Err(SpecifiedBoundaryUpdateError::MissingUpperStagger {
            axis,
            required_end,
            extent,
        });
    }
    Ok(range.start..required_end)
}

fn validate_tile(
    axis: SpecifiedBoundaryUpdateAxis,
    tile: &Range<usize>,
    domain: &Range<usize>,
    extent: usize,
) -> SpecifiedBoundaryUpdateResult<()> {
    validate_range(axis, tile, extent)?;
    let permitted_end = domain.end.saturating_add(1).min(extent);
    let permitted_domain = domain.start..permitted_end;
    if tile.start < permitted_domain.start
        || tile.start >= permitted_domain.end
        || tile.end > permitted_domain.end
    {
        return Err(SpecifiedBoundaryUpdateError::TileOutsideDomain {
            axis,
            tile: tile.clone(),
            domain: permitted_domain,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_extends_only_its_named_stagger() {
        let shape = GridShape::try_new(6, 6, 6).unwrap();
        let west_east = SpecifiedBoundaryUpdateRegion::try_new(
            shape,
            SpecifiedBoundaryFieldLocation::WestEastFace,
            1..5,
            1..5,
            1..5,
            1..6,
            1..5,
            1..5,
        )
        .unwrap();
        let full_level = SpecifiedBoundaryUpdateRegion::try_new(
            shape,
            SpecifiedBoundaryFieldLocation::FullLevel,
            1..5,
            1..5,
            1..5,
            1..5,
            1..5,
            1..6,
        )
        .unwrap();

        assert_eq!(west_east.active_ranges().west_east, 1..6);
        assert_eq!(west_east.active_ranges().bottom_top, 1..5);
        assert_eq!(full_level.active_ranges().west_east, 1..5);
        assert_eq!(full_level.active_ranges().bottom_top, 1..6);
    }

    #[test]
    fn rejects_missing_upper_stagger_storage() {
        let result = SpecifiedBoundaryUpdateRegion::try_new(
            GridShape::try_new(5, 6, 6).unwrap(),
            SpecifiedBoundaryFieldLocation::WestEastFace,
            1..5,
            1..5,
            1..5,
            1..5,
            1..5,
            1..5,
        );

        assert!(matches!(
            result,
            Err(SpecifiedBoundaryUpdateError::MissingUpperStagger {
                axis: SpecifiedBoundaryUpdateAxis::WestEast,
                ..
            })
        ));
    }
}
