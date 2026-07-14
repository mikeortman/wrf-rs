use std::fmt;
use std::ops::Range;

use wrf_compute::GridShape;

use crate::{PHYSICAL_BOUNDARY_ZONE, PhysicalBoundaryError, PhysicalBoundaryResult};

/// Logical axis used in physical-boundary validation errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhysicalBoundaryAxis {
    /// West-east storage axis.
    WestEast,
    /// South-north storage axis.
    SouthNorth,
    /// Bottom-top storage axis.
    BottomTop,
}

impl fmt::Display for PhysicalBoundaryAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WestEast => formatter.write_str("west-east"),
            Self::SouthNorth => formatter.write_str("south-north"),
            Self::BottomTop => formatter.write_str("bottom-top"),
        }
    }
}

/// Validated storage, physical-domain, and tile geometry for one patch.
///
/// Storage covers WRF memory bounds (`ims:ime`, `kms:kme`, `jms:jme`) mapped
/// to zero-based indices. The mass ranges map Fortran `ids..ide-1` and
/// `jds..jde-1`; their exclusive ends are therefore the staggered `ide`/`jde`
/// indices. Tiles use the same staggered-inclusive convention, so a tile that
/// reaches `ide` has `tile.end == mass.end + 1`. Both horizontal halos must
/// hold WRF's fixed four-point boundary zone, and the patch is the whole
/// domain (single rank), which keeps WRF's on-processor periodic test true.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalBoundaryRegion {
    shape: GridShape,
    mass_west_east: Range<usize>,
    mass_south_north: Range<usize>,
    half_level: Range<usize>,
    tile_west_east: Range<usize>,
    tile_south_north: Range<usize>,
    tile_bottom_top: Range<usize>,
}

impl PhysicalBoundaryRegion {
    /// Validates domains, four-point halos, staggered storage, and tiles.
    pub fn try_new(
        shape: GridShape,
        mass_west_east: Range<usize>,
        mass_south_north: Range<usize>,
        half_level: Range<usize>,
        tile_west_east: Range<usize>,
        tile_south_north: Range<usize>,
        tile_bottom_top: Range<usize>,
    ) -> PhysicalBoundaryResult<Self> {
        validate_domain_with_halo(
            PhysicalBoundaryAxis::WestEast,
            &mass_west_east,
            shape.west_east_points(),
        )?;
        validate_domain_with_halo(
            PhysicalBoundaryAxis::SouthNorth,
            &mass_south_north,
            shape.south_north_points(),
        )?;
        validate_half_level_domain(&half_level, shape.bottom_top_points())?;
        validate_tile(
            PhysicalBoundaryAxis::WestEast,
            &tile_west_east,
            &mass_west_east,
        )?;
        validate_tile(
            PhysicalBoundaryAxis::SouthNorth,
            &tile_south_north,
            &mass_south_north,
        )?;
        validate_tile(
            PhysicalBoundaryAxis::BottomTop,
            &tile_bottom_top,
            &half_level,
        )?;
        Ok(Self {
            shape,
            mass_west_east,
            mass_south_north,
            half_level,
            tile_west_east,
            tile_south_north,
            tile_bottom_top,
        })
    }

    /// Returns the common volume storage shape checked before mutation.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    /// Returns the matching horizontal storage shape with one level.
    pub const fn horizontal_shape(&self) -> GridShape {
        self.shape.horizontal_shape()
    }

    /// Returns the unstaggered physical domains used to derive this region.
    pub fn mass_domains(&self) -> (&Range<usize>, &Range<usize>, &Range<usize>) {
        (
            &self.mass_west_east,
            &self.mass_south_north,
            &self.half_level,
        )
    }

    pub(crate) fn mass_west_east(&self) -> &Range<usize> {
        &self.mass_west_east
    }

    pub(crate) fn mass_south_north(&self) -> &Range<usize> {
        &self.mass_south_north
    }

    pub(crate) fn half_level(&self) -> &Range<usize> {
        &self.half_level
    }

    pub(crate) fn tile_west_east(&self) -> &Range<usize> {
        &self.tile_west_east
    }

    pub(crate) fn tile_south_north(&self) -> &Range<usize> {
        &self.tile_south_north
    }

    pub(crate) fn tile_bottom_top(&self) -> &Range<usize> {
        &self.tile_bottom_top
    }
}

fn validate_domain_with_halo(
    axis: PhysicalBoundaryAxis,
    domain: &Range<usize>,
    extent: usize,
) -> PhysicalBoundaryResult<()> {
    if domain.is_empty() || domain.end > extent {
        return Err(PhysicalBoundaryError::InvalidRange {
            axis,
            range: domain.clone(),
            extent,
        });
    }
    let has_lower_halo = domain.start >= PHYSICAL_BOUNDARY_ZONE;
    let has_upper_halo = domain.end + PHYSICAL_BOUNDARY_ZONE < extent;
    if !has_lower_halo || !has_upper_halo {
        return Err(PhysicalBoundaryError::MissingBoundaryZone {
            axis,
            domain: domain.clone(),
            extent,
        });
    }
    Ok(())
}

fn validate_half_level_domain(
    half_level: &Range<usize>,
    extent: usize,
) -> PhysicalBoundaryResult<()> {
    if half_level.is_empty() || half_level.end + 1 > extent {
        return Err(PhysicalBoundaryError::InvalidRange {
            axis: PhysicalBoundaryAxis::BottomTop,
            range: half_level.clone(),
            extent,
        });
    }
    Ok(())
}

fn validate_tile(
    axis: PhysicalBoundaryAxis,
    tile: &Range<usize>,
    domain: &Range<usize>,
) -> PhysicalBoundaryResult<()> {
    let staggered = domain.start..domain.end + 1;
    if tile.is_empty() || tile.start < staggered.start || tile.end > staggered.end {
        return Err(PhysicalBoundaryError::TileOutsideDomain {
            axis,
            tile: tile.clone(),
            domain: staggered,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape() -> GridShape {
        GridShape::try_new(15, 15, 7).unwrap()
    }

    #[test]
    fn accepts_a_domain_with_full_boundary_zones() {
        let region =
            PhysicalBoundaryRegion::try_new(shape(), 4..10, 4..10, 1..6, 4..11, 4..11, 1..7);

        assert!(region.is_ok());
    }

    #[test]
    fn rejects_a_shallow_west_halo() {
        let result =
            PhysicalBoundaryRegion::try_new(shape(), 3..10, 4..10, 1..6, 3..11, 4..11, 1..7);

        assert!(matches!(
            result,
            Err(PhysicalBoundaryError::MissingBoundaryZone {
                axis: PhysicalBoundaryAxis::WestEast,
                ..
            })
        ));
    }

    #[test]
    fn rejects_a_shallow_north_halo() {
        let result =
            PhysicalBoundaryRegion::try_new(shape(), 4..10, 4..11, 1..6, 4..11, 4..12, 1..7);

        assert!(matches!(
            result,
            Err(PhysicalBoundaryError::MissingBoundaryZone {
                axis: PhysicalBoundaryAxis::SouthNorth,
                ..
            })
        ));
    }

    #[test]
    fn rejects_a_missing_full_level_point() {
        let result =
            PhysicalBoundaryRegion::try_new(shape(), 4..10, 4..10, 1..7, 4..11, 4..11, 1..7);

        assert!(matches!(
            result,
            Err(PhysicalBoundaryError::InvalidRange {
                axis: PhysicalBoundaryAxis::BottomTop,
                ..
            })
        ));
    }

    #[test]
    fn rejects_a_tile_beyond_the_staggered_domain() {
        let result =
            PhysicalBoundaryRegion::try_new(shape(), 4..10, 4..10, 1..6, 4..12, 4..11, 1..7);

        assert!(matches!(
            result,
            Err(PhysicalBoundaryError::TileOutsideDomain {
                axis: PhysicalBoundaryAxis::WestEast,
                ..
            })
        ));
    }
}
