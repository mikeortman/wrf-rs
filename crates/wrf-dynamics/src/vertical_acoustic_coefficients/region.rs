use std::ops::Range;

use wrf_compute::GridShape;

use crate::{
    VerticalAcousticCoefficientAxis, VerticalAcousticCoefficientError,
    VerticalAcousticCoefficientResult,
};

/// Validated mass domains and horizontal tile for WRF `calc_coef_w`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerticalAcousticCoefficientRegion {
    shape: GridShape,
    west_east_domain: Range<usize>,
    south_north_domain: Range<usize>,
    half_level_domain: Range<usize>,
    west_east_tile: Range<usize>,
    south_north_tile: Range<usize>,
}

impl VerticalAcousticCoefficientRegion {
    /// Validates horizontal clipping and the upper full level required by the
    /// complete-column recurrence.
    ///
    /// WRF ignores `kts` and `kte` in this routine. The half-level domain is
    /// therefore the complete physical column, and its exclusive end names the
    /// top full-level output point.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        west_east_domain: Range<usize>,
        south_north_domain: Range<usize>,
        half_level_domain: Range<usize>,
        west_east_tile: Range<usize>,
        south_north_tile: Range<usize>,
    ) -> VerticalAcousticCoefficientResult<Self> {
        for (axis, range, extent) in [
            (
                VerticalAcousticCoefficientAxis::WestEast,
                &west_east_domain,
                shape.west_east_points(),
            ),
            (
                VerticalAcousticCoefficientAxis::SouthNorth,
                &south_north_domain,
                shape.south_north_points(),
            ),
            (
                VerticalAcousticCoefficientAxis::BottomTop,
                &half_level_domain,
                shape.bottom_top_points(),
            ),
        ] {
            validate_domain(axis, range, extent)?;
        }
        for (axis, tile, domain, extent) in [
            (
                VerticalAcousticCoefficientAxis::WestEast,
                &west_east_tile,
                &west_east_domain,
                shape.west_east_points(),
            ),
            (
                VerticalAcousticCoefficientAxis::SouthNorth,
                &south_north_tile,
                &south_north_domain,
                shape.south_north_points(),
            ),
        ] {
            validate_tile(axis, tile, domain, extent)?;
        }
        let required_end = half_level_domain.end.checked_add(1).ok_or(
            VerticalAcousticCoefficientError::MissingUpperFullLevel {
                required_end: usize::MAX,
                field_extent: shape.bottom_top_points(),
            },
        )?;
        if required_end > shape.bottom_top_points() {
            return Err(VerticalAcousticCoefficientError::MissingUpperFullLevel {
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

    pub(crate) const fn lower_full_level(&self) -> usize {
        self.half_level_domain.start
    }

    pub(crate) const fn top_full_level(&self) -> usize {
        self.half_level_domain.end
    }
}

fn validate_domain(
    axis: VerticalAcousticCoefficientAxis,
    range: &Range<usize>,
    extent: usize,
) -> VerticalAcousticCoefficientResult<()> {
    if range.start >= range.end {
        return Err(VerticalAcousticCoefficientError::EmptyDomainRange { axis });
    }
    if range.end > extent {
        return Err(VerticalAcousticCoefficientError::DomainRangeOutOfBounds {
            axis,
            range_end: range.end,
            field_extent: extent,
        });
    }
    Ok(())
}

fn validate_tile(
    axis: VerticalAcousticCoefficientAxis,
    tile: &Range<usize>,
    domain: &Range<usize>,
    extent: usize,
) -> VerticalAcousticCoefficientResult<()> {
    if tile.start >= tile.end {
        return Err(VerticalAcousticCoefficientError::EmptyTileRange { axis });
    }
    if tile.end > extent {
        return Err(VerticalAcousticCoefficientError::TileRangeOutOfBounds {
            axis,
            range_end: tile.end,
            field_extent: extent,
        });
    }
    if tile.start < domain.start || tile.end > domain.end.saturating_add(1) {
        return Err(VerticalAcousticCoefficientError::TileOutsideDomain { axis });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clips_upper_horizontal_tile_points_and_retains_complete_column() {
        let region = VerticalAcousticCoefficientRegion::try_new(
            GridShape::try_new(7, 6, 8).unwrap(),
            1..6,
            1..5,
            2..7,
            2..7,
            2..6,
        )
        .unwrap();

        assert_eq!(region.active_west_east(), 2..6);
        assert_eq!(region.active_south_north(), 2..5);
        assert_eq!(region.lower_full_level(), 2);
        assert_eq!(region.top_full_level(), 7);
    }

    #[test]
    fn rejects_half_levels_without_the_upper_full_level() {
        assert_eq!(
            VerticalAcousticCoefficientRegion::try_new(
                GridShape::try_new(7, 6, 7).unwrap(),
                1..6,
                1..5,
                2..7,
                1..6,
                1..5,
            ),
            Err(VerticalAcousticCoefficientError::MissingUpperFullLevel {
                required_end: 8,
                field_extent: 7,
            })
        );
    }
}
