use std::ops::Range;

use wrf_compute::GridShape;

use crate::{DryTendencyAssemblyAxis, DryTendencyAssemblyError, DryTendencyAssemblyResult};

/// Validated physical-domain and active-tile ranges for `rk_addtend_dry`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DryTendencyAssemblyRegion {
    shape: GridShape,
    west_east_domain: Range<usize>,
    south_north_domain: Range<usize>,
    bottom_top_domain: Range<usize>,
    west_east_tile: Range<usize>,
    south_north_tile: Range<usize>,
    bottom_top_tile: Range<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DryTendencyAssemblyActiveRanges {
    pub(crate) west_east: Range<usize>,
    pub(crate) south_north: Range<usize>,
    pub(crate) bottom_top: Range<usize>,
}

impl DryTendencyAssemblyRegion {
    /// Validates zero-based half-open mass-domain and C-grid tile ranges.
    pub fn try_new(
        shape: GridShape,
        west_east_domain: Range<usize>,
        south_north_domain: Range<usize>,
        bottom_top_domain: Range<usize>,
        west_east_tile: Range<usize>,
        south_north_tile: Range<usize>,
        bottom_top_tile: Range<usize>,
    ) -> DryTendencyAssemblyResult<Self> {
        for (axis, range, extent) in [
            (
                DryTendencyAssemblyAxis::WestEast,
                &west_east_domain,
                shape.west_east_points(),
            ),
            (
                DryTendencyAssemblyAxis::SouthNorth,
                &south_north_domain,
                shape.south_north_points(),
            ),
            (
                DryTendencyAssemblyAxis::BottomTop,
                &bottom_top_domain,
                shape.bottom_top_points(),
            ),
        ] {
            validate_domain(axis, range, extent)?;
        }
        for (axis, tile, domain, extent) in [
            (
                DryTendencyAssemblyAxis::WestEast,
                &west_east_tile,
                &west_east_domain,
                shape.west_east_points(),
            ),
            (
                DryTendencyAssemblyAxis::SouthNorth,
                &south_north_tile,
                &south_north_domain,
                shape.south_north_points(),
            ),
            (
                DryTendencyAssemblyAxis::BottomTop,
                &bottom_top_tile,
                &bottom_top_domain,
                shape.bottom_top_points(),
            ),
        ] {
            validate_tile(axis, tile, domain, extent)?;
        }
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

    /// Returns the volume field shape used by this region.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    pub(crate) fn west_east_momentum_ranges(&self) -> DryTendencyAssemblyActiveRanges {
        self.ranges(false, true, true)
    }

    pub(crate) fn south_north_momentum_ranges(&self) -> DryTendencyAssemblyActiveRanges {
        self.ranges(true, false, true)
    }

    pub(crate) fn vertical_ranges(&self) -> DryTendencyAssemblyActiveRanges {
        self.ranges(true, true, false)
    }

    pub(crate) fn mass_ranges(&self) -> DryTendencyAssemblyActiveRanges {
        self.ranges(true, true, true)
    }

    fn ranges(&self, clip_x: bool, clip_y: bool, clip_z: bool) -> DryTendencyAssemblyActiveRanges {
        DryTendencyAssemblyActiveRanges {
            west_east: clipped_if(&self.west_east_tile, self.west_east_domain.end, clip_x),
            south_north: clipped_if(&self.south_north_tile, self.south_north_domain.end, clip_y),
            bottom_top: if clip_z {
                self.bottom_top_tile.start
                    ..self
                        .bottom_top_tile
                        .end
                        .saturating_sub(1)
                        .min(self.bottom_top_domain.end)
            } else {
                self.bottom_top_tile.clone()
            },
        }
    }
}

fn clipped_if(range: &Range<usize>, domain_end: usize, should_clip: bool) -> Range<usize> {
    range.start..if should_clip {
        range.end.min(domain_end)
    } else {
        range.end
    }
}

fn validate_domain(
    axis: DryTendencyAssemblyAxis,
    range: &Range<usize>,
    field_extent: usize,
) -> DryTendencyAssemblyResult<()> {
    if range.start >= range.end {
        return Err(DryTendencyAssemblyError::EmptyMassDomainRange { axis });
    }
    if range.end > field_extent {
        return Err(DryTendencyAssemblyError::MassDomainRangeOutOfBounds {
            axis,
            range_end: range.end,
            field_extent,
        });
    }
    Ok(())
}

fn validate_tile(
    axis: DryTendencyAssemblyAxis,
    tile: &Range<usize>,
    domain: &Range<usize>,
    field_extent: usize,
) -> DryTendencyAssemblyResult<()> {
    if tile.start >= tile.end {
        return Err(DryTendencyAssemblyError::EmptyTileRange { axis });
    }
    if tile.end > field_extent {
        return Err(DryTendencyAssemblyError::TileRangeOutOfBounds {
            axis,
            range_end: tile.end,
            field_extent,
        });
    }
    if tile.start < domain.start || tile.end > domain.end.saturating_add(1) {
        return Err(DryTendencyAssemblyError::TileOutsideMassDomain { axis });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_all_staggered_component_ranges() {
        let region = DryTendencyAssemblyRegion::try_new(
            GridShape::try_new(7, 6, 5).unwrap(),
            1..6,
            1..5,
            1..4,
            2..7,
            2..6,
            1..5,
        )
        .unwrap();
        assert_eq!(
            region.west_east_momentum_ranges(),
            DryTendencyAssemblyActiveRanges {
                west_east: 2..7,
                south_north: 2..5,
                bottom_top: 1..4
            }
        );
        assert_eq!(
            region.south_north_momentum_ranges(),
            DryTendencyAssemblyActiveRanges {
                west_east: 2..6,
                south_north: 2..6,
                bottom_top: 1..4
            }
        );
        assert_eq!(
            region.vertical_ranges(),
            DryTendencyAssemblyActiveRanges {
                west_east: 2..6,
                south_north: 2..5,
                bottom_top: 1..5
            }
        );
        assert_eq!(
            region.mass_ranges(),
            DryTendencyAssemblyActiveRanges {
                west_east: 2..6,
                south_north: 2..5,
                bottom_top: 1..4
            }
        );
    }

    #[test]
    fn removes_inclusive_kte_from_interior_mass_level_loops() {
        let region = DryTendencyAssemblyRegion::try_new(
            GridShape::try_new(7, 6, 5).unwrap(),
            1..6,
            1..5,
            1..4,
            2..4,
            2..4,
            1..3,
        )
        .unwrap();

        assert_eq!(region.mass_ranges().bottom_top, 1..2);
        assert_eq!(region.vertical_ranges().bottom_top, 1..3);
    }
}
