use std::ops::Range;

use wrf_compute::GridShape;

use crate::{InverseDensityAxis, InverseDensityError, InverseDensityResult};

/// Validated physical mass domain and active tile for full inverse density.
///
/// Ranges use zero-based, half-open storage offsets. A tile may contain the
/// single upper stagger point accepted by WRF; calculation clips that point on
/// every axis because `alt`, `al`, and `alb` are mass-grid fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InverseDensityRegion {
    shape: GridShape,
    west_east_domain: Range<usize>,
    south_north_domain: Range<usize>,
    bottom_top_domain: Range<usize>,
    west_east_tile: Range<usize>,
    south_north_tile: Range<usize>,
    bottom_top_tile: Range<usize>,
}

impl InverseDensityRegion {
    /// Validates mass-domain, tile, and storage bounds.
    ///
    /// # Errors
    ///
    /// Returns a typed error when a range is empty, exceeds storage, or places
    /// a tile outside its physical domain plus one upper stagger point.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        west_east_domain: Range<usize>,
        south_north_domain: Range<usize>,
        bottom_top_domain: Range<usize>,
        west_east_tile: Range<usize>,
        south_north_tile: Range<usize>,
        bottom_top_tile: Range<usize>,
    ) -> InverseDensityResult<Self> {
        for (axis, range, extent) in [
            (
                InverseDensityAxis::WestEast,
                &west_east_domain,
                shape.west_east_points(),
            ),
            (
                InverseDensityAxis::SouthNorth,
                &south_north_domain,
                shape.south_north_points(),
            ),
            (
                InverseDensityAxis::BottomTop,
                &bottom_top_domain,
                shape.bottom_top_points(),
            ),
        ] {
            validate_domain(axis, range, extent)?;
        }
        for (axis, tile, domain, extent) in [
            (
                InverseDensityAxis::WestEast,
                &west_east_tile,
                &west_east_domain,
                shape.west_east_points(),
            ),
            (
                InverseDensityAxis::SouthNorth,
                &south_north_tile,
                &south_north_domain,
                shape.south_north_points(),
            ),
            (
                InverseDensityAxis::BottomTop,
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

    /// Returns the three-dimensional field shape used during validation.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    pub(crate) fn output_ranges(&self) -> (Range<usize>, Range<usize>, Range<usize>) {
        (
            clipped(&self.west_east_tile, self.west_east_domain.end),
            clipped(&self.south_north_tile, self.south_north_domain.end),
            clipped(&self.bottom_top_tile, self.bottom_top_domain.end),
        )
    }
}

fn clipped(tile: &Range<usize>, domain_end: usize) -> Range<usize> {
    tile.start.min(domain_end)..tile.end.min(domain_end)
}

fn validate_domain(
    axis: InverseDensityAxis,
    range: &Range<usize>,
    field_extent: usize,
) -> InverseDensityResult<()> {
    if range.start >= range.end {
        return Err(InverseDensityError::EmptyMassDomainRange { axis });
    }
    if range.end > field_extent {
        return Err(InverseDensityError::MassDomainRangeOutOfBounds {
            axis,
            range_end: range.end,
            field_extent,
        });
    }
    Ok(())
}

fn validate_tile(
    axis: InverseDensityAxis,
    tile: &Range<usize>,
    domain: &Range<usize>,
    field_extent: usize,
) -> InverseDensityResult<()> {
    if tile.start >= tile.end {
        return Err(InverseDensityError::EmptyTileRange { axis });
    }
    if tile.end > field_extent {
        return Err(InverseDensityError::TileRangeOutOfBounds {
            axis,
            range_end: tile.end,
            field_extent,
        });
    }
    if tile.start < domain.start || tile.end > domain.end.saturating_add(1) {
        return Err(InverseDensityError::TileOutsideMassDomain { axis });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clips_every_upper_stagger_to_the_mass_domain() {
        let shape = GridShape::try_new(8, 7, 7).unwrap();
        let region =
            InverseDensityRegion::try_new(shape, 1..6, 1..5, 2..5, 1..7, 1..6, 2..6).unwrap();

        assert_eq!(region.output_ranges(), (1..6, 1..5, 2..5));
    }

    #[test]
    fn rejects_every_range_failure_category() {
        for (axis, extent) in [
            (InverseDensityAxis::WestEast, 8),
            (InverseDensityAxis::SouthNorth, 7),
            (InverseDensityAxis::BottomTop, 7),
        ] {
            assert_eq!(
                validate_domain(axis, &(1..1), extent),
                Err(InverseDensityError::EmptyMassDomainRange { axis })
            );
            assert_eq!(
                validate_domain(axis, &(1..extent + 1), extent),
                Err(InverseDensityError::MassDomainRangeOutOfBounds {
                    axis,
                    range_end: extent + 1,
                    field_extent: extent,
                })
            );
            assert_eq!(
                validate_tile(axis, &(2..2), &(1..5), extent),
                Err(InverseDensityError::EmptyTileRange { axis })
            );
            assert_eq!(
                validate_tile(axis, &(1..extent + 1), &(1..extent), extent),
                Err(InverseDensityError::TileRangeOutOfBounds {
                    axis,
                    range_end: extent + 1,
                    field_extent: extent,
                })
            );
            assert_eq!(
                validate_tile(axis, &(0..2), &(1..5), extent),
                Err(InverseDensityError::TileOutsideMassDomain { axis })
            );
        }
    }
}
