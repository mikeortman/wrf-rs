use std::ops::Range;

use wrf_compute::GridShape;

use super::{
    SpecifiedBoundaryFinalizationError, SpecifiedBoundaryFinalizationFieldLocation,
    SpecifiedBoundaryFinalizationResult,
};
use crate::SpecifiedBoundaryUpdateRegion;
use crate::specified_boundary_update::region::SpecifiedBoundaryActiveRanges;

/// Validated field location, physical domain, and tile for boundary finalization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpecifiedBoundaryFinalizationRegion {
    location: SpecifiedBoundaryFinalizationFieldLocation,
    update_region: SpecifiedBoundaryUpdateRegion,
    active_bottom_top: Range<usize>,
}

impl SpecifiedBoundaryFinalizationRegion {
    /// Validates storage, physical domains, location-specific staggers, and tile ranges.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        location: SpecifiedBoundaryFinalizationFieldLocation,
        mass_domain_west_east: Range<usize>,
        mass_domain_south_north: Range<usize>,
        half_level_domain: Range<usize>,
        tile_west_east: Range<usize>,
        tile_south_north: Range<usize>,
        tile_bottom_top: Range<usize>,
    ) -> SpecifiedBoundaryFinalizationResult<Self> {
        let tile_bottom_top_start = tile_bottom_top.start;
        let update_region = SpecifiedBoundaryUpdateRegion::try_new(
            shape,
            location.geometry_location(),
            mass_domain_west_east,
            mass_domain_south_north,
            half_level_domain,
            tile_west_east,
            tile_south_north,
            tile_bottom_top,
        )?;
        let active_bottom_top_end = match location {
            SpecifiedBoundaryFinalizationFieldLocation::MassHalfLevel
            | SpecifiedBoundaryFinalizationFieldLocation::WestEastMomentum
            | SpecifiedBoundaryFinalizationFieldLocation::SouthNorthMomentum
            | SpecifiedBoundaryFinalizationFieldLocation::HorizontalMass => {
                update_region.mass_domains().2.end
            }
            SpecifiedBoundaryFinalizationFieldLocation::VerticalMomentum
            | SpecifiedBoundaryFinalizationFieldLocation::FullLevel => update_region
                .mass_domains()
                .2
                .end
                .checked_add(1)
                .ok_or(SpecifiedBoundaryFinalizationError::BoundaryVerticalExtentOverflow)?,
        };
        let active_bottom_top = tile_bottom_top_start..active_bottom_top_end;
        Ok(Self {
            location,
            update_region,
            active_bottom_top,
        })
    }

    /// Returns the field shape checked before mutation.
    pub const fn shape(&self) -> GridShape {
        self.update_region.shape()
    }

    /// Returns the field location and its WRF normalization policy.
    pub const fn location(&self) -> SpecifiedBoundaryFinalizationFieldLocation {
        self.location
    }

    pub(crate) fn active_ranges(&self) -> SpecifiedBoundaryActiveRanges {
        let mut ranges = self.update_region.active_ranges();
        ranges.bottom_top = self.active_bottom_top.clone();
        ranges
    }

    pub(crate) fn half_level_domain(&self) -> &Range<usize> {
        self.update_region.mass_domains().2
    }

    pub(crate) fn boundary_vertical_points(&self) -> SpecifiedBoundaryFinalizationResult<usize> {
        if self.location == SpecifiedBoundaryFinalizationFieldLocation::HorizontalMass {
            return Ok(1);
        }
        self.half_level_domain()
            .len()
            .checked_add(1)
            .ok_or(SpecifiedBoundaryFinalizationError::BoundaryVerticalExtentOverflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_location_ignores_the_upper_tile_bound_at_its_physical_top() {
        let shape = GridShape::try_new(8, 8, 8).unwrap();
        let scalar = SpecifiedBoundaryFinalizationRegion::try_new(
            shape,
            SpecifiedBoundaryFinalizationFieldLocation::MassHalfLevel,
            1..7,
            1..7,
            1..7,
            1..7,
            1..7,
            2..5,
        )
        .unwrap();
        let full_level = SpecifiedBoundaryFinalizationRegion::try_new(
            shape,
            SpecifiedBoundaryFinalizationFieldLocation::FullLevel,
            1..7,
            1..7,
            1..7,
            1..7,
            1..7,
            2..5,
        )
        .unwrap();

        assert_eq!(scalar.active_ranges().bottom_top, 2..7);
        assert_eq!(full_level.active_ranges().bottom_top, 2..8);
    }
}
