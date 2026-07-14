use std::ops::Range;

use wrf_compute::GridShape;

use crate::{AcousticPressureAxis, AcousticPressureError, AcousticPressureResult};

/// Validated physical domains and clipped mass-point tile for `calc_p_rho`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcousticPressureRegion {
    shape: GridShape,
    west_east_domain: Range<usize>,
    south_north_domain: Range<usize>,
    half_level_domain: Range<usize>,
    west_east_tile: Range<usize>,
    south_north_tile: Range<usize>,
    half_level_tile: Range<usize>,
}

impl AcousticPressureRegion {
    /// Validates domain/tile separation and the geopotential level at `k + 1`.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        west_east_domain: Range<usize>,
        south_north_domain: Range<usize>,
        half_level_domain: Range<usize>,
        west_east_tile: Range<usize>,
        south_north_tile: Range<usize>,
        half_level_tile: Range<usize>,
    ) -> AcousticPressureResult<Self> {
        for (axis, range, extent) in [
            (
                AcousticPressureAxis::WestEast,
                &west_east_domain,
                shape.west_east_points(),
            ),
            (
                AcousticPressureAxis::SouthNorth,
                &south_north_domain,
                shape.south_north_points(),
            ),
            (
                AcousticPressureAxis::BottomTop,
                &half_level_domain,
                shape.bottom_top_points(),
            ),
        ] {
            validate_domain(axis, range, extent)?;
        }
        for (axis, tile, domain, extent) in [
            (
                AcousticPressureAxis::WestEast,
                &west_east_tile,
                &west_east_domain,
                shape.west_east_points(),
            ),
            (
                AcousticPressureAxis::SouthNorth,
                &south_north_tile,
                &south_north_domain,
                shape.south_north_points(),
            ),
            (
                AcousticPressureAxis::BottomTop,
                &half_level_tile,
                &half_level_domain,
                shape.bottom_top_points(),
            ),
        ] {
            validate_tile(axis, tile, domain, extent)?;
        }
        let active_half_level_end = half_level_tile.end.min(half_level_domain.end);
        let required_end = active_half_level_end.checked_add(1).ok_or(
            AcousticPressureError::MissingUpperFullLevel {
                required_end: usize::MAX,
                field_extent: shape.bottom_top_points(),
            },
        )?;
        if required_end > shape.bottom_top_points() {
            return Err(AcousticPressureError::MissingUpperFullLevel {
                required_end,
                field_extent: shape.bottom_top_points(),
            });
        }
        Ok(Self {
            shape,
            west_east_domain,
            south_north_domain,
            half_level_domain,
            west_east_tile,
            south_north_tile,
            half_level_tile,
        })
    }

    /// Returns the common three-dimensional field shape.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    pub(crate) fn active_west_east(&self) -> Range<usize> {
        self.west_east_tile.start..self.west_east_tile.end.min(self.west_east_domain.end)
    }

    pub(crate) fn active_south_north(&self) -> Range<usize> {
        self.south_north_tile.start..self.south_north_tile.end.min(self.south_north_domain.end)
    }

    pub(crate) fn active_half_levels(&self) -> Range<usize> {
        self.half_level_tile.start..self.half_level_tile.end.min(self.half_level_domain.end)
    }
}

fn validate_domain(
    axis: AcousticPressureAxis,
    range: &Range<usize>,
    extent: usize,
) -> AcousticPressureResult<()> {
    if range.start >= range.end {
        return Err(AcousticPressureError::EmptyDomainRange { axis });
    }
    if range.end > extent {
        return Err(AcousticPressureError::DomainRangeOutOfBounds {
            axis,
            range_end: range.end,
            field_extent: extent,
        });
    }
    Ok(())
}

fn validate_tile(
    axis: AcousticPressureAxis,
    tile: &Range<usize>,
    domain: &Range<usize>,
    extent: usize,
) -> AcousticPressureResult<()> {
    if tile.start >= tile.end {
        return Err(AcousticPressureError::EmptyTileRange { axis });
    }
    if tile.end > extent {
        return Err(AcousticPressureError::TileRangeOutOfBounds {
            axis,
            range_end: tile.end,
            field_extent: extent,
        });
    }
    if tile.start < domain.start || tile.end > domain.end.saturating_add(1) {
        return Err(AcousticPressureError::TileOutsideDomain { axis });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clips_tile_endpoints_and_retains_upper_full_level() {
        let region = AcousticPressureRegion::try_new(
            GridShape::try_new(7, 6, 6).unwrap(),
            1..6,
            1..5,
            1..5,
            2..7,
            2..6,
            2..6,
        )
        .unwrap();
        assert_eq!(region.active_west_east(), 2..6);
        assert_eq!(region.active_south_north(), 2..5);
        assert_eq!(region.active_half_levels(), 2..5);
    }

    #[test]
    fn rejects_a_half_level_without_its_upper_geopotential_level() {
        assert_eq!(
            AcousticPressureRegion::try_new(
                GridShape::try_new(7, 6, 5).unwrap(),
                1..6,
                1..5,
                1..5,
                1..6,
                1..5,
                1..5,
            ),
            Err(AcousticPressureError::MissingUpperFullLevel {
                required_end: 6,
                field_extent: 5,
            })
        );
    }
}
