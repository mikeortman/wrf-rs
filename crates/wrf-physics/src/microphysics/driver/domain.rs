use std::ops::Range;

use wrf_compute::GridShape;

use crate::{
    KesslerMicrophysicsAxis, MicrophysicsBoundaryPolicy, MicrophysicsDriverError,
    MicrophysicsDriverResult, MicrophysicsTile,
};

/// Validated domain extents and boundary policy for microphysics dispatch.
///
/// The horizontal ranges are the zero-based mass-point domain, so they already
/// carry the call-site `min(i_end, ide-1)` / `min(j_end, jde-1)` staggered-edge
/// clipping that `solve_em` applies before entering the driver. The vertical
/// range carries the call-site `min(k_end, kde-1)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MicrophysicsDriverDomain {
    field_shape: GridShape,
    west_east_range: Range<usize>,
    south_north_range: Range<usize>,
    bottom_top_range: Range<usize>,
    boundary_policy: MicrophysicsBoundaryPolicy,
}

impl MicrophysicsDriverDomain {
    /// Validates the mass-point domain ranges against the allocated shape.
    ///
    /// # Errors
    ///
    /// Returns an error if any range is empty or extends beyond the allocated
    /// field extent on its axis.
    pub fn try_new(
        field_shape: GridShape,
        west_east_range: Range<usize>,
        south_north_range: Range<usize>,
        bottom_top_range: Range<usize>,
        boundary_policy: MicrophysicsBoundaryPolicy,
    ) -> MicrophysicsDriverResult<Self> {
        validate_domain_range(
            KesslerMicrophysicsAxis::WestEast,
            &west_east_range,
            field_shape.west_east_points(),
        )?;
        validate_domain_range(
            KesslerMicrophysicsAxis::SouthNorth,
            &south_north_range,
            field_shape.south_north_points(),
        )?;
        validate_domain_range(
            KesslerMicrophysicsAxis::BottomTop,
            &bottom_top_range,
            field_shape.bottom_top_points(),
        )?;

        Ok(Self {
            field_shape,
            west_east_range,
            south_north_range,
            bottom_top_range,
            boundary_policy,
        })
    }

    /// Returns the allocated shape every three-dimensional field must match.
    pub const fn field_shape(&self) -> GridShape {
        self.field_shape
    }

    pub(crate) fn bottom_top_range(&self) -> Range<usize> {
        self.bottom_top_range.clone()
    }

    pub(crate) fn west_east_range(&self) -> Range<usize> {
        self.west_east_range.clone()
    }

    pub(crate) fn south_north_range(&self) -> Range<usize> {
        self.south_north_range.clone()
    }

    /// Clips one tile exactly like the pinned driver's per-tile preamble.
    ///
    /// Outside a channel configuration both horizontal axes skip the
    /// effective boundary zone; a channel keeps the full west-east domain.
    /// Returns `None` when the clipped tile holds no active points, matching
    /// a Fortran tile loop that never executes.
    pub(crate) fn clip_tile(
        &self,
        tile: &MicrophysicsTile,
    ) -> Option<(Range<usize>, Range<usize>)> {
        let zone_width = self.boundary_policy.effective_zone_width();
        let west_east_active = if self.boundary_policy.is_channel() {
            self.west_east_range.clone()
        } else {
            shrink_range(&self.west_east_range, zone_width)
        };
        let south_north_active = shrink_range(&self.south_north_range, zone_width);

        let west_east = intersect_ranges(&west_east_active, &tile.west_east_range());
        let south_north = intersect_ranges(&south_north_active, &tile.south_north_range());
        if west_east.is_empty() || south_north.is_empty() {
            return None;
        }
        Some((west_east, south_north))
    }
}

fn shrink_range(range: &Range<usize>, zone_width: usize) -> Range<usize> {
    let start = range.start.saturating_add(zone_width);
    let end = range.end.saturating_sub(zone_width);
    start..end.max(start)
}

fn intersect_ranges(left: &Range<usize>, right: &Range<usize>) -> Range<usize> {
    let start = left.start.max(right.start);
    let end = left.end.min(right.end);
    start..end.max(start)
}

fn validate_domain_range(
    axis: KesslerMicrophysicsAxis,
    range: &Range<usize>,
    field_extent: usize,
) -> MicrophysicsDriverResult<()> {
    if range.is_empty() {
        return Err(MicrophysicsDriverError::EmptyDomainRange { axis });
    }
    if range.end > field_extent {
        return Err(MicrophysicsDriverError::DomainRangeOutOfBounds {
            axis,
            range_end: range.end,
            field_extent,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_domain(policy: MicrophysicsBoundaryPolicy) -> MicrophysicsDriverDomain {
        let shape = GridShape::try_new(8, 7, 5).unwrap();
        MicrophysicsDriverDomain::try_new(shape, 1..7, 1..6, 0..5, policy).unwrap()
    }

    #[test]
    fn clips_both_axes_by_the_specified_zone() {
        let domain = create_domain(MicrophysicsBoundaryPolicy::new(true, false, 1));

        let (west_east, south_north) = domain
            .clip_tile(&MicrophysicsTile::new(1..7, 1..3))
            .unwrap();

        assert_eq!(west_east, 2..6);
        assert_eq!(south_north, 2..3);
    }

    #[test]
    fn channel_keeps_the_full_west_east_domain() {
        let domain = create_domain(MicrophysicsBoundaryPolicy::new(true, true, 1));

        let (west_east, south_north) = domain
            .clip_tile(&MicrophysicsTile::new(1..7, 1..6))
            .unwrap();

        assert_eq!(west_east, 1..7);
        assert_eq!(south_north, 2..5);
    }

    #[test]
    fn open_boundaries_keep_the_full_mass_domain() {
        let domain = create_domain(MicrophysicsBoundaryPolicy::open());

        let (west_east, south_north) = domain
            .clip_tile(&MicrophysicsTile::new(0..8, 0..7))
            .unwrap();

        assert_eq!(west_east, 1..7);
        assert_eq!(south_north, 1..6);
    }

    #[test]
    fn tile_inside_the_boundary_zone_is_inactive() {
        let domain = create_domain(MicrophysicsBoundaryPolicy::new(true, false, 2));

        assert_eq!(domain.clip_tile(&MicrophysicsTile::new(1..7, 1..2)), None);
    }

    #[test]
    fn oversized_zone_clips_every_tile_to_nothing() {
        let domain = create_domain(MicrophysicsBoundaryPolicy::new(true, false, 4));

        assert_eq!(domain.clip_tile(&MicrophysicsTile::new(1..7, 1..6)), None);
    }

    #[test]
    fn rejects_empty_and_out_of_bounds_domain_ranges() {
        let shape = GridShape::try_new(8, 7, 5).unwrap();
        let policy = MicrophysicsBoundaryPolicy::open();

        assert_eq!(
            MicrophysicsDriverDomain::try_new(shape, 3..3, 1..6, 0..5, policy),
            Err(MicrophysicsDriverError::EmptyDomainRange {
                axis: KesslerMicrophysicsAxis::WestEast,
            })
        );
        assert_eq!(
            MicrophysicsDriverDomain::try_new(shape, 1..7, 1..8, 0..5, policy),
            Err(MicrophysicsDriverError::DomainRangeOutOfBounds {
                axis: KesslerMicrophysicsAxis::SouthNorth,
                range_end: 8,
                field_extent: 7,
            })
        );
    }
}
