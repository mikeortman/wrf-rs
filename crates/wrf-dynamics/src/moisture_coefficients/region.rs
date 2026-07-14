use std::ops::Range;

use wrf_compute::GridShape;

use crate::{MoistureCoefficientAxis, MoistureCoefficientError, MoistureCoefficientResult};

/// Validated physical domain and active tile for moisture coefficients.
///
/// Ranges are zero-based, half-open storage offsets. Physical ranges exclude
/// each upper stagger point. Tiles may contain that one additional point,
/// matching WRF's inclusive `ite`, `jte`, and `kte` bounds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MoistureCoefficientRegion {
    shape: GridShape,
    west_east_domain: Range<usize>,
    south_north_domain: Range<usize>,
    bottom_top_domain: Range<usize>,
    west_east_tile: Range<usize>,
    south_north_tile: Range<usize>,
    bottom_top_tile: Range<usize>,
}

impl MoistureCoefficientRegion {
    /// Validates domain, stagger clipping, and storage bounds.
    ///
    /// # Errors
    ///
    /// Returns a typed error for empty or out-of-storage ranges, tiles outside
    /// their domain plus one stagger point.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        west_east_domain: Range<usize>,
        south_north_domain: Range<usize>,
        bottom_top_domain: Range<usize>,
        west_east_tile: Range<usize>,
        south_north_tile: Range<usize>,
        bottom_top_tile: Range<usize>,
    ) -> MoistureCoefficientResult<Self> {
        validate_domain(
            MoistureCoefficientAxis::WestEast,
            &west_east_domain,
            shape.west_east_points(),
        )?;
        validate_domain(
            MoistureCoefficientAxis::SouthNorth,
            &south_north_domain,
            shape.south_north_points(),
        )?;
        validate_domain(
            MoistureCoefficientAxis::BottomTop,
            &bottom_top_domain,
            shape.bottom_top_points(),
        )?;
        validate_tile(
            MoistureCoefficientAxis::WestEast,
            &west_east_tile,
            &west_east_domain,
            shape.west_east_points(),
        )?;
        validate_tile(
            MoistureCoefficientAxis::SouthNorth,
            &south_north_tile,
            &south_north_domain,
            shape.south_north_points(),
        )?;
        validate_tile(
            MoistureCoefficientAxis::BottomTop,
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

    pub(crate) fn west_east_output_ranges(&self) -> (Range<usize>, Range<usize>, Range<usize>) {
        (
            self.west_east_tile.clone(),
            clipped(&self.south_north_tile, self.south_north_domain.end),
            clipped(&self.bottom_top_tile, self.bottom_top_domain.end),
        )
    }

    pub(crate) fn south_north_output_ranges(&self) -> (Range<usize>, Range<usize>, Range<usize>) {
        (
            clipped(&self.west_east_tile, self.west_east_domain.end),
            self.south_north_tile.clone(),
            clipped(&self.bottom_top_tile, self.bottom_top_domain.end),
        )
    }

    pub(crate) fn vertical_output_ranges(&self) -> (Range<usize>, Range<usize>, Range<usize>) {
        let bottom_top_end = self.bottom_top_tile.end.min(self.bottom_top_domain.end);
        (
            clipped(&self.west_east_tile, self.west_east_domain.end),
            clipped(&self.south_north_tile, self.south_north_domain.end),
            self.bottom_top_tile
                .start
                .saturating_add(1)
                .min(bottom_top_end)..bottom_top_end,
        )
    }

    pub(crate) fn validate_active_species_neighbors(&self) -> MoistureCoefficientResult<()> {
        validate_lower_neighbor(MoistureCoefficientAxis::WestEast, &self.west_east_tile)?;
        validate_lower_neighbor(MoistureCoefficientAxis::SouthNorth, &self.south_north_tile)
    }
}

fn clipped(range: &Range<usize>, domain_end: usize) -> Range<usize> {
    range.start.min(domain_end)..range.end.min(domain_end)
}

fn validate_domain(
    axis: MoistureCoefficientAxis,
    range: &Range<usize>,
    field_extent: usize,
) -> MoistureCoefficientResult<()> {
    if range.start >= range.end {
        return Err(MoistureCoefficientError::EmptyDomainRange { axis });
    }
    if range.end > field_extent {
        return Err(MoistureCoefficientError::DomainRangeOutOfBounds {
            axis,
            range_end: range.end,
            field_extent,
        });
    }
    Ok(())
}

fn validate_tile(
    axis: MoistureCoefficientAxis,
    tile: &Range<usize>,
    domain: &Range<usize>,
    field_extent: usize,
) -> MoistureCoefficientResult<()> {
    if tile.start >= tile.end {
        return Err(MoistureCoefficientError::EmptyTileRange { axis });
    }
    if tile.end > field_extent {
        return Err(MoistureCoefficientError::TileRangeOutOfBounds {
            axis,
            range_end: tile.end,
            field_extent,
        });
    }
    if tile.start < domain.start || tile.end > domain.end.saturating_add(1) {
        return Err(MoistureCoefficientError::TileOutsideDomain { axis });
    }
    Ok(())
}

fn validate_lower_neighbor(
    axis: MoistureCoefficientAxis,
    tile: &Range<usize>,
) -> MoistureCoefficientResult<()> {
    if tile.start == 0 {
        return Err(MoistureCoefficientError::MissingLowerNeighbor {
            axis,
            tile_start: tile.start,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_each_wrf_component_range() {
        let shape = GridShape::try_new(8, 7, 7).unwrap();
        let region =
            MoistureCoefficientRegion::try_new(shape, 1..6, 1..5, 2..5, 1..7, 1..6, 2..6).unwrap();

        assert_eq!(region.west_east_output_ranges(), (1..7, 1..5, 2..5));
        assert_eq!(region.south_north_output_ranges(), (1..6, 1..6, 2..5));
        assert_eq!(region.vertical_output_ranges(), (1..6, 1..5, 3..5));
    }

    #[test]
    fn reports_missing_west_and_south_neighbors_for_active_species() {
        let shape = GridShape::try_new(8, 7, 7).unwrap();
        let missing_west =
            MoistureCoefficientRegion::try_new(shape, 0..6, 1..5, 1..5, 0..3, 1..4, 1..4).unwrap();
        assert_eq!(
            missing_west.validate_active_species_neighbors(),
            Err(MoistureCoefficientError::MissingLowerNeighbor {
                axis: MoistureCoefficientAxis::WestEast,
                tile_start: 0,
            })
        );
        let missing_south =
            MoistureCoefficientRegion::try_new(shape, 1..6, 0..5, 1..5, 1..4, 0..4, 1..4).unwrap();
        assert_eq!(
            missing_south.validate_active_species_neighbors(),
            Err(MoistureCoefficientError::MissingLowerNeighbor {
                axis: MoistureCoefficientAxis::SouthNorth,
                tile_start: 0,
            })
        );
    }

    #[test]
    fn rejects_every_range_failure_category() {
        for (axis, extent) in [
            (MoistureCoefficientAxis::WestEast, 8),
            (MoistureCoefficientAxis::SouthNorth, 7),
            (MoistureCoefficientAxis::BottomTop, 7),
        ] {
            assert_eq!(
                validate_domain(axis, &(1..1), extent),
                Err(MoistureCoefficientError::EmptyDomainRange { axis })
            );
            assert_eq!(
                validate_domain(axis, &(1..extent + 1), extent),
                Err(MoistureCoefficientError::DomainRangeOutOfBounds {
                    axis,
                    range_end: extent + 1,
                    field_extent: extent,
                })
            );
            assert_eq!(
                validate_tile(axis, &(2..2), &(1..5), extent),
                Err(MoistureCoefficientError::EmptyTileRange { axis })
            );
            assert_eq!(
                validate_tile(axis, &(1..extent + 1), &(1..extent), extent),
                Err(MoistureCoefficientError::TileRangeOutOfBounds {
                    axis,
                    range_end: extent + 1,
                    field_extent: extent,
                })
            );
            assert_eq!(
                validate_tile(axis, &(0..2), &(1..5), extent),
                Err(MoistureCoefficientError::TileOutsideDomain { axis })
            );
        }
    }
}
