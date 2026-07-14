use std::ops::Range;

use wrf_compute::GridShape;

use crate::{
    PressurePointGeopotentialAxis, PressurePointGeopotentialError, PressurePointGeopotentialResult,
};

/// Validated mass domain and active tile for pressure-point geopotential.
///
/// Ranges use zero-based, half-open storage offsets. Calculation clips one
/// upper stagger point on every axis. The bottom-top domain additionally
/// requires one stored full level above its last active mass level.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PressurePointGeopotentialRegion {
    shape: GridShape,
    west_east_domain: Range<usize>,
    south_north_domain: Range<usize>,
    bottom_top_domain: Range<usize>,
    west_east_tile: Range<usize>,
    south_north_tile: Range<usize>,
    bottom_top_tile: Range<usize>,
}

impl PressurePointGeopotentialRegion {
    /// Validates mass-domain, tile, storage, and vertical-neighbor bounds.
    ///
    /// # Errors
    ///
    /// Returns a typed error when a range is empty, exceeds storage, places a
    /// tile outside its physical domain plus upper stagger, or omits the full
    /// level read by WRF at `k + 1`.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        west_east_domain: Range<usize>,
        south_north_domain: Range<usize>,
        bottom_top_domain: Range<usize>,
        west_east_tile: Range<usize>,
        south_north_tile: Range<usize>,
        bottom_top_tile: Range<usize>,
    ) -> PressurePointGeopotentialResult<Self> {
        for (axis, range, extent) in [
            (
                PressurePointGeopotentialAxis::WestEast,
                &west_east_domain,
                shape.west_east_points(),
            ),
            (
                PressurePointGeopotentialAxis::SouthNorth,
                &south_north_domain,
                shape.south_north_points(),
            ),
            (
                PressurePointGeopotentialAxis::BottomTop,
                &bottom_top_domain,
                shape.bottom_top_points(),
            ),
        ] {
            validate_domain(axis, range, extent)?;
        }
        validate_upper_full_level(&bottom_top_domain, shape.bottom_top_points())?;

        for (axis, tile, domain, extent) in [
            (
                PressurePointGeopotentialAxis::WestEast,
                &west_east_tile,
                &west_east_domain,
                shape.west_east_points(),
            ),
            (
                PressurePointGeopotentialAxis::SouthNorth,
                &south_north_tile,
                &south_north_domain,
                shape.south_north_points(),
            ),
            (
                PressurePointGeopotentialAxis::BottomTop,
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
    axis: PressurePointGeopotentialAxis,
    range: &Range<usize>,
    field_extent: usize,
) -> PressurePointGeopotentialResult<()> {
    if range.start >= range.end {
        return Err(PressurePointGeopotentialError::EmptyMassDomainRange { axis });
    }
    if range.end > field_extent {
        return Err(PressurePointGeopotentialError::MassDomainRangeOutOfBounds {
            axis,
            range_end: range.end,
            field_extent,
        });
    }
    Ok(())
}

fn validate_upper_full_level(
    bottom_top_domain: &Range<usize>,
    field_extent: usize,
) -> PressurePointGeopotentialResult<()> {
    if bottom_top_domain.end >= field_extent {
        return Err(PressurePointGeopotentialError::MissingUpperFullLevel {
            required_index: bottom_top_domain.end,
            field_extent,
        });
    }
    Ok(())
}

fn validate_tile(
    axis: PressurePointGeopotentialAxis,
    tile: &Range<usize>,
    domain: &Range<usize>,
    field_extent: usize,
) -> PressurePointGeopotentialResult<()> {
    if tile.start >= tile.end {
        return Err(PressurePointGeopotentialError::EmptyTileRange { axis });
    }
    if tile.end > field_extent {
        return Err(PressurePointGeopotentialError::TileRangeOutOfBounds {
            axis,
            range_end: tile.end,
            field_extent,
        });
    }
    if tile.start < domain.start || tile.end > domain.end.saturating_add(1) {
        return Err(PressurePointGeopotentialError::TileOutsideMassDomain { axis });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clips_upper_staggers_and_retains_the_vertical_input_neighbor() {
        let shape = GridShape::try_new(8, 7, 7).unwrap();
        let region =
            PressurePointGeopotentialRegion::try_new(shape, 1..6, 1..5, 2..5, 1..7, 1..6, 2..6)
                .unwrap();

        assert_eq!(region.output_ranges(), (1..6, 1..5, 2..5));
        assert!(region.bottom_top_domain.end < region.shape.bottom_top_points());
    }

    #[test]
    fn rejects_a_mass_domain_without_the_upper_full_level() {
        let shape = GridShape::try_new(8, 7, 7).unwrap();

        assert_eq!(
            PressurePointGeopotentialRegion::try_new(shape, 1..6, 1..5, 2..7, 1..6, 1..5, 2..7,),
            Err(PressurePointGeopotentialError::MissingUpperFullLevel {
                required_index: 7,
                field_extent: 7,
            })
        );
    }

    #[test]
    fn rejects_every_range_failure_category() {
        for (axis, extent) in [
            (PressurePointGeopotentialAxis::WestEast, 8),
            (PressurePointGeopotentialAxis::SouthNorth, 7),
            (PressurePointGeopotentialAxis::BottomTop, 7),
        ] {
            assert_eq!(
                validate_domain(axis, &(1..1), extent),
                Err(PressurePointGeopotentialError::EmptyMassDomainRange { axis })
            );
            assert_eq!(
                validate_domain(axis, &(1..extent + 1), extent),
                Err(PressurePointGeopotentialError::MassDomainRangeOutOfBounds {
                    axis,
                    range_end: extent + 1,
                    field_extent: extent,
                })
            );
            assert_eq!(
                validate_tile(axis, &(2..2), &(1..5), extent),
                Err(PressurePointGeopotentialError::EmptyTileRange { axis })
            );
            assert_eq!(
                validate_tile(axis, &(1..extent + 1), &(1..extent), extent),
                Err(PressurePointGeopotentialError::TileRangeOutOfBounds {
                    axis,
                    range_end: extent + 1,
                    field_extent: extent,
                })
            );
            assert_eq!(
                validate_tile(axis, &(0..2), &(1..5), extent),
                Err(PressurePointGeopotentialError::TileOutsideMassDomain { axis })
            );
        }
    }
}
