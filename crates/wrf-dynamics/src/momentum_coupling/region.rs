use std::ops::Range;

use wrf_compute::GridShape;

use crate::{MomentumCouplingAxis, MomentumCouplingError, MomentumCouplingResult};

/// Validated physical-domain and active-tile ranges for momentum coupling.
///
/// All ranges are zero-based, half-open memory offsets. Physical mass-domain
/// ranges exclude each axis's upper staggered boundary. Tile ranges may include
/// that one additional point, matching WRF's inclusive `ite`, `jte`, and `kte`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MomentumCouplingRegion {
    shape: GridShape,
    west_east_domain: Range<usize>,
    south_north_domain: Range<usize>,
    bottom_top_domain: Range<usize>,
    west_east_tile: Range<usize>,
    south_north_tile: Range<usize>,
    bottom_top_tile: Range<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MomentumCouplingActiveRanges {
    pub(crate) west_east: Range<usize>,
    pub(crate) south_north: Range<usize>,
    pub(crate) bottom_top: Range<usize>,
}

impl MomentumCouplingRegion {
    /// Validates physical mass-domain ranges and the active C-grid tile.
    ///
    /// # Errors
    ///
    /// Returns a typed error if a range is empty, outside field storage, or
    /// outside the physical mass domain plus its single upper stagger point.
    pub fn try_new(
        shape: GridShape,
        west_east_domain: Range<usize>,
        south_north_domain: Range<usize>,
        bottom_top_domain: Range<usize>,
        west_east_tile: Range<usize>,
        south_north_tile: Range<usize>,
        bottom_top_tile: Range<usize>,
    ) -> MomentumCouplingResult<Self> {
        validate_domain(
            MomentumCouplingAxis::WestEast,
            &west_east_domain,
            shape.west_east_points(),
        )?;
        validate_domain(
            MomentumCouplingAxis::SouthNorth,
            &south_north_domain,
            shape.south_north_points(),
        )?;
        validate_domain(
            MomentumCouplingAxis::BottomTop,
            &bottom_top_domain,
            shape.bottom_top_points(),
        )?;
        validate_tile(
            MomentumCouplingAxis::WestEast,
            &west_east_tile,
            &west_east_domain,
            shape.west_east_points(),
        )?;
        validate_tile(
            MomentumCouplingAxis::SouthNorth,
            &south_north_tile,
            &south_north_domain,
            shape.south_north_points(),
        )?;
        validate_tile(
            MomentumCouplingAxis::BottomTop,
            &bottom_top_tile,
            &bottom_top_domain,
            shape.bottom_top_points(),
        )?;

        Ok(Self {
            shape,
            west_east_domain,
            south_north_domain,
            bottom_top_domain,
            west_east_tile,
            south_north_tile,
            bottom_top_tile,
        })
    }

    /// Returns the three-dimensional field shape used during validation.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    pub(crate) fn west_east_momentum_ranges(&self) -> MomentumCouplingActiveRanges {
        MomentumCouplingActiveRanges {
            west_east: self.west_east_tile.clone(),
            south_north: clipped(&self.south_north_tile, self.south_north_domain.end),
            bottom_top: clipped(&self.bottom_top_tile, self.bottom_top_domain.end),
        }
    }

    pub(crate) fn south_north_momentum_ranges(&self) -> MomentumCouplingActiveRanges {
        MomentumCouplingActiveRanges {
            west_east: clipped(&self.west_east_tile, self.west_east_domain.end),
            south_north: self.south_north_tile.clone(),
            bottom_top: clipped(&self.bottom_top_tile, self.bottom_top_domain.end),
        }
    }

    pub(crate) fn vertical_momentum_ranges(&self) -> MomentumCouplingActiveRanges {
        MomentumCouplingActiveRanges {
            west_east: clipped(&self.west_east_tile, self.west_east_domain.end),
            south_north: clipped(&self.south_north_tile, self.south_north_domain.end),
            bottom_top: self.bottom_top_tile.clone(),
        }
    }
}

fn clipped(range: &Range<usize>, domain_end: usize) -> Range<usize> {
    range.start..range.end.min(domain_end)
}

fn validate_domain(
    axis: MomentumCouplingAxis,
    range: &Range<usize>,
    field_extent: usize,
) -> MomentumCouplingResult<()> {
    if range.start >= range.end {
        return Err(MomentumCouplingError::EmptyMassDomainRange { axis });
    }
    if range.end > field_extent {
        return Err(MomentumCouplingError::MassDomainRangeOutOfBounds {
            axis,
            range_end: range.end,
            field_extent,
        });
    }
    Ok(())
}

fn validate_tile(
    axis: MomentumCouplingAxis,
    tile: &Range<usize>,
    domain: &Range<usize>,
    field_extent: usize,
) -> MomentumCouplingResult<()> {
    if tile.start >= tile.end {
        return Err(MomentumCouplingError::EmptyTileRange { axis });
    }
    if tile.end > field_extent {
        return Err(MomentumCouplingError::TileRangeOutOfBounds {
            axis,
            range_end: tile.end,
            field_extent,
        });
    }
    if tile.start < domain.start || tile.end > domain.end.saturating_add(1) {
        return Err(MomentumCouplingError::TileOutsideMassDomain { axis });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_wrf_component_specific_clipping() {
        let shape = GridShape::try_new(7, 6, 5).unwrap();
        let region =
            MomentumCouplingRegion::try_new(shape, 1..5, 1..4, 1..4, 1..6, 1..5, 1..5).unwrap();

        assert_eq!(
            region.west_east_momentum_ranges(),
            MomentumCouplingActiveRanges {
                west_east: 1..6,
                south_north: 1..4,
                bottom_top: 1..4,
            }
        );
        assert_eq!(
            region.south_north_momentum_ranges(),
            MomentumCouplingActiveRanges {
                west_east: 1..5,
                south_north: 1..5,
                bottom_top: 1..4,
            }
        );
        assert_eq!(
            region.vertical_momentum_ranges(),
            MomentumCouplingActiveRanges {
                west_east: 1..5,
                south_north: 1..4,
                bottom_top: 1..5,
            }
        );
    }

    #[test]
    fn rejects_invalid_domain_and_tile_ranges() {
        for (axis, field_extent) in [
            (MomentumCouplingAxis::WestEast, 6),
            (MomentumCouplingAxis::SouthNorth, 6),
            (MomentumCouplingAxis::BottomTop, 5),
        ] {
            assert_eq!(
                validate_domain(axis, &(1..1), field_extent),
                Err(MomentumCouplingError::EmptyMassDomainRange { axis })
            );
            assert_eq!(
                validate_domain(axis, &(1..field_extent + 1), field_extent),
                Err(MomentumCouplingError::MassDomainRangeOutOfBounds {
                    axis,
                    range_end: field_extent + 1,
                    field_extent,
                })
            );
            assert_eq!(
                validate_tile(axis, &(2..2), &(1..4), field_extent),
                Err(MomentumCouplingError::EmptyTileRange { axis })
            );
            assert_eq!(
                validate_tile(axis, &(1..field_extent + 1), &(1..4), field_extent),
                Err(MomentumCouplingError::TileRangeOutOfBounds {
                    axis,
                    range_end: field_extent + 1,
                    field_extent,
                })
            );
            assert_eq!(
                validate_tile(axis, &(0..2), &(1..4), field_extent),
                Err(MomentumCouplingError::TileOutsideMassDomain { axis })
            );
        }
    }
}
