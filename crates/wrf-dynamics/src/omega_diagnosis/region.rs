use std::ops::Range;

use wrf_compute::GridShape;

use crate::{OmegaDiagnosisAxis, OmegaDiagnosisError, OmegaDiagnosisResult};

/// Validated physical domain and complete-column tile for omega diagnosis.
///
/// Ranges are zero-based, half-open memory offsets. Horizontal physical-domain
/// ranges exclude their upper stagger points. The half-level range excludes
/// the top full level, while `full_level_tile` must include it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OmegaDiagnosisRegion {
    shape: GridShape,
    west_east_domain: Range<usize>,
    south_north_domain: Range<usize>,
    half_levels: Range<usize>,
    west_east_tile: Range<usize>,
    south_north_tile: Range<usize>,
    full_level_tile: Range<usize>,
}

impl OmegaDiagnosisRegion {
    /// Validates C-grid neighbors, horizontal clipping, and full-column coverage.
    ///
    /// # Errors
    ///
    /// Returns a typed error for empty/out-of-storage ranges, a horizontal tile
    /// outside its domain plus one stagger point, missing C-grid neighbors, or
    /// a vertical tile that does not cover every half level and the top face.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        west_east_domain: Range<usize>,
        south_north_domain: Range<usize>,
        half_levels: Range<usize>,
        west_east_tile: Range<usize>,
        south_north_tile: Range<usize>,
        full_level_tile: Range<usize>,
    ) -> OmegaDiagnosisResult<Self> {
        validate_domain(
            OmegaDiagnosisAxis::WestEast,
            &west_east_domain,
            shape.west_east_points(),
        )?;
        validate_domain(
            OmegaDiagnosisAxis::SouthNorth,
            &south_north_domain,
            shape.south_north_points(),
        )?;
        validate_domain(
            OmegaDiagnosisAxis::BottomTop,
            &half_levels,
            shape.bottom_top_points(),
        )?;
        validate_horizontal_tile(
            OmegaDiagnosisAxis::WestEast,
            &west_east_tile,
            &west_east_domain,
            shape.west_east_points(),
        )?;
        validate_horizontal_tile(
            OmegaDiagnosisAxis::SouthNorth,
            &south_north_tile,
            &south_north_domain,
            shape.south_north_points(),
        )?;
        validate_vertical_column(&half_levels, &full_level_tile, shape.bottom_top_points())?;

        Ok(Self {
            shape,
            west_east_domain,
            south_north_domain,
            half_levels,
            west_east_tile,
            south_north_tile,
            full_level_tile,
        })
    }

    /// Returns the three-dimensional field shape used during validation.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    pub(crate) fn active_west_east(&self) -> Range<usize> {
        self.west_east_tile.start..self.west_east_tile.end.min(self.west_east_domain.end)
    }

    pub(crate) fn active_south_north(&self) -> Range<usize> {
        self.south_north_tile.start..self.south_north_tile.end.min(self.south_north_domain.end)
    }

    pub(crate) fn west_east_tile(&self) -> Range<usize> {
        self.west_east_tile.clone()
    }

    pub(crate) fn half_levels(&self) -> Range<usize> {
        self.half_levels.clone()
    }

    pub(crate) fn top_full_level(&self) -> usize {
        self.full_level_tile.end - 1
    }
}

fn validate_domain(
    axis: OmegaDiagnosisAxis,
    range: &Range<usize>,
    field_extent: usize,
) -> OmegaDiagnosisResult<()> {
    if range.start >= range.end {
        return Err(OmegaDiagnosisError::EmptyDomainRange { axis });
    }
    if range.end > field_extent {
        return Err(OmegaDiagnosisError::DomainRangeOutOfBounds {
            axis,
            range_end: range.end,
            field_extent,
        });
    }
    Ok(())
}

fn validate_horizontal_tile(
    axis: OmegaDiagnosisAxis,
    tile: &Range<usize>,
    domain: &Range<usize>,
    field_extent: usize,
) -> OmegaDiagnosisResult<()> {
    if tile.start >= tile.end {
        return Err(OmegaDiagnosisError::EmptyTileRange { axis });
    }
    if tile.end > field_extent {
        return Err(OmegaDiagnosisError::TileRangeOutOfBounds {
            axis,
            range_end: tile.end,
            field_extent,
        });
    }
    if tile.start < domain.start || tile.end > domain.end.saturating_add(1) {
        return Err(OmegaDiagnosisError::TileOutsideDomain { axis });
    }

    let active_end = tile.end.min(domain.end);
    if tile.start < active_end && tile.start == 0 {
        return Err(OmegaDiagnosisError::MissingLowerNeighbor {
            axis,
            tile_start: tile.start,
        });
    }
    if tile.start < active_end && active_end >= field_extent {
        return Err(OmegaDiagnosisError::MissingUpperNeighbor {
            axis,
            active_end,
            field_extent,
        });
    }
    Ok(())
}

fn validate_vertical_column(
    half_levels: &Range<usize>,
    full_level_tile: &Range<usize>,
    field_extent: usize,
) -> OmegaDiagnosisResult<()> {
    let expected_end =
        half_levels
            .end
            .checked_add(1)
            .ok_or(OmegaDiagnosisError::DomainRangeOutOfBounds {
                axis: OmegaDiagnosisAxis::BottomTop,
                range_end: usize::MAX,
                field_extent,
            })?;
    if expected_end > field_extent {
        return Err(OmegaDiagnosisError::DomainRangeOutOfBounds {
            axis: OmegaDiagnosisAxis::BottomTop,
            range_end: expected_end,
            field_extent,
        });
    }
    if full_level_tile.start != half_levels.start || full_level_tile.end != expected_end {
        return Err(OmegaDiagnosisError::IncompleteVerticalColumn {
            expected_start: half_levels.start,
            expected_end,
            actual_start: full_level_tile.start,
            actual_end: full_level_tile.end,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_clipped_horizontal_ranges_and_complete_vertical_levels() {
        let shape = GridShape::try_new(8, 7, 7).unwrap();
        let region =
            OmegaDiagnosisRegion::try_new(shape, 1..6, 1..5, 2..5, 1..7, 1..6, 2..6).unwrap();

        assert_eq!(region.active_west_east(), 1..6);
        assert_eq!(region.active_south_north(), 1..5);
        assert_eq!(region.west_east_tile(), 1..7);
        assert_eq!(region.half_levels(), 2..5);
        assert_eq!(region.top_full_level(), 5);
    }

    #[test]
    fn rejects_incomplete_vertical_columns_and_missing_horizontal_neighbors() {
        let shape = GridShape::try_new(8, 7, 7).unwrap();
        assert_eq!(
            OmegaDiagnosisRegion::try_new(shape, 1..6, 1..5, 2..5, 1..7, 1..6, 2..5,),
            Err(OmegaDiagnosisError::IncompleteVerticalColumn {
                expected_start: 2,
                expected_end: 6,
                actual_start: 2,
                actual_end: 5,
            })
        );
        assert_eq!(
            validate_horizontal_tile(OmegaDiagnosisAxis::WestEast, &(0..3), &(0..6), 8),
            Err(OmegaDiagnosisError::MissingLowerNeighbor {
                axis: OmegaDiagnosisAxis::WestEast,
                tile_start: 0,
            })
        );
        assert_eq!(
            validate_horizontal_tile(OmegaDiagnosisAxis::SouthNorth, &(1..7), &(1..7), 7),
            Err(OmegaDiagnosisError::MissingUpperNeighbor {
                axis: OmegaDiagnosisAxis::SouthNorth,
                active_end: 7,
                field_extent: 7,
            })
        );
    }

    #[test]
    fn rejects_each_range_failure_category_on_every_axis() {
        for (axis, field_extent) in [
            (OmegaDiagnosisAxis::WestEast, 8),
            (OmegaDiagnosisAxis::SouthNorth, 7),
            (OmegaDiagnosisAxis::BottomTop, 7),
        ] {
            assert_eq!(
                validate_domain(axis, &(1..1), field_extent),
                Err(OmegaDiagnosisError::EmptyDomainRange { axis })
            );
            assert_eq!(
                validate_domain(axis, &(1..field_extent + 1), field_extent),
                Err(OmegaDiagnosisError::DomainRangeOutOfBounds {
                    axis,
                    range_end: field_extent + 1,
                    field_extent,
                })
            );
        }
        for (axis, field_extent) in [
            (OmegaDiagnosisAxis::WestEast, 8),
            (OmegaDiagnosisAxis::SouthNorth, 7),
        ] {
            assert_eq!(
                validate_horizontal_tile(axis, &(2..2), &(1..5), field_extent),
                Err(OmegaDiagnosisError::EmptyTileRange { axis })
            );
            assert_eq!(
                validate_horizontal_tile(
                    axis,
                    &(1..field_extent + 1),
                    &(1..field_extent),
                    field_extent,
                ),
                Err(OmegaDiagnosisError::TileRangeOutOfBounds {
                    axis,
                    range_end: field_extent + 1,
                    field_extent,
                })
            );
            assert_eq!(
                validate_horizontal_tile(axis, &(0..2), &(1..5), field_extent),
                Err(OmegaDiagnosisError::TileOutsideDomain { axis })
            );
        }
    }
}
