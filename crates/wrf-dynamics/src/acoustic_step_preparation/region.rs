use std::ops::Range;

use wrf_compute::GridShape;

use crate::{
    AcousticStepPreparationAxis, AcousticStepPreparationError, AcousticStepPreparationResult,
};

/// Validated horizontal tile and complete vertical acoustic column.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcousticStepPreparationRegion {
    shape: GridShape,
    west_east_domain: Range<usize>,
    south_north_domain: Range<usize>,
    half_levels: Range<usize>,
    west_east_tile: Range<usize>,
    south_north_tile: Range<usize>,
    full_levels: Range<usize>,
}

impl AcousticStepPreparationRegion {
    /// Validates mass-domain ranges, C-grid upper staggers, and full-column coverage.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        west_east_domain: Range<usize>,
        south_north_domain: Range<usize>,
        half_levels: Range<usize>,
        west_east_tile: Range<usize>,
        south_north_tile: Range<usize>,
        full_levels: Range<usize>,
    ) -> AcousticStepPreparationResult<Self> {
        for (axis, range, extent) in [
            (
                AcousticStepPreparationAxis::WestEast,
                &west_east_domain,
                shape.west_east_points(),
            ),
            (
                AcousticStepPreparationAxis::SouthNorth,
                &south_north_domain,
                shape.south_north_points(),
            ),
            (
                AcousticStepPreparationAxis::BottomTop,
                &half_levels,
                shape.bottom_top_points(),
            ),
        ] {
            validate_domain(axis, range, extent)?;
        }
        validate_horizontal_tile(
            AcousticStepPreparationAxis::WestEast,
            &west_east_tile,
            &west_east_domain,
            shape.west_east_points(),
        )?;
        validate_horizontal_tile(
            AcousticStepPreparationAxis::SouthNorth,
            &south_north_tile,
            &south_north_domain,
            shape.south_north_points(),
        )?;
        let expected_full_end = half_levels.end.checked_add(1).ok_or(
            AcousticStepPreparationError::DomainRangeOutOfBounds {
                axis: AcousticStepPreparationAxis::BottomTop,
                range_end: usize::MAX,
                field_extent: shape.bottom_top_points(),
            },
        )?;
        if expected_full_end > shape.bottom_top_points() {
            return Err(AcousticStepPreparationError::DomainRangeOutOfBounds {
                axis: AcousticStepPreparationAxis::BottomTop,
                range_end: expected_full_end,
                field_extent: shape.bottom_top_points(),
            });
        }
        if full_levels.start != half_levels.start || full_levels.end != expected_full_end {
            return Err(AcousticStepPreparationError::IncompleteVerticalColumn {
                expected_start: half_levels.start,
                expected_end: expected_full_end,
                actual_start: full_levels.start,
                actual_end: full_levels.end,
            });
        }
        Ok(Self {
            shape,
            west_east_domain,
            south_north_domain,
            half_levels,
            west_east_tile,
            south_north_tile,
            full_levels,
        })
    }

    /// Returns the common volume field shape.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    pub(crate) fn active_west_east(&self) -> Range<usize> {
        self.west_east_tile.start..self.west_east_tile.end.min(self.west_east_domain.end)
    }

    pub(crate) fn active_south_north(&self) -> Range<usize> {
        self.south_north_tile.start..self.south_north_tile.end.min(self.south_north_domain.end)
    }

    pub(crate) fn west_east_velocity_range(&self) -> Range<usize> {
        self.west_east_tile.clone()
    }
    pub(crate) fn south_north_velocity_range(&self) -> Range<usize> {
        self.south_north_tile.clone()
    }
    pub(crate) fn half_levels(&self) -> Range<usize> {
        self.half_levels.clone()
    }
    pub(crate) fn full_levels(&self) -> Range<usize> {
        self.full_levels.clone()
    }

    pub(crate) fn validate_later_substep_neighbors(&self) -> AcousticStepPreparationResult<()> {
        if self.active_west_east().start == 0 {
            return Err(AcousticStepPreparationError::MissingLowerNeighbor {
                axis: AcousticStepPreparationAxis::WestEast,
            });
        }
        if self.active_south_north().start == 0 {
            return Err(AcousticStepPreparationError::MissingLowerNeighbor {
                axis: AcousticStepPreparationAxis::SouthNorth,
            });
        }
        Ok(())
    }
}

fn validate_domain(
    axis: AcousticStepPreparationAxis,
    range: &Range<usize>,
    extent: usize,
) -> AcousticStepPreparationResult<()> {
    if range.start >= range.end {
        return Err(AcousticStepPreparationError::EmptyDomainRange { axis });
    }
    if range.end > extent {
        return Err(AcousticStepPreparationError::DomainRangeOutOfBounds {
            axis,
            range_end: range.end,
            field_extent: extent,
        });
    }
    Ok(())
}

fn validate_horizontal_tile(
    axis: AcousticStepPreparationAxis,
    tile: &Range<usize>,
    domain: &Range<usize>,
    extent: usize,
) -> AcousticStepPreparationResult<()> {
    if tile.start >= tile.end {
        return Err(AcousticStepPreparationError::EmptyTileRange { axis });
    }
    if tile.end > extent {
        return Err(AcousticStepPreparationError::TileRangeOutOfBounds {
            axis,
            range_end: tile.end,
            field_extent: extent,
        });
    }
    if tile.start < domain.start || tile.end > domain.end.saturating_add(1) {
        return Err(AcousticStepPreparationError::TileOutsideDomain { axis });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_mass_and_staggered_ranges() {
        let region = AcousticStepPreparationRegion::try_new(
            GridShape::try_new(7, 6, 6).unwrap(),
            1..6,
            1..5,
            1..5,
            2..7,
            2..6,
            1..6,
        )
        .unwrap();
        assert_eq!(region.active_west_east(), 2..6);
        assert_eq!(region.active_south_north(), 2..5);
        assert_eq!(region.west_east_velocity_range(), 2..7);
        assert_eq!(region.south_north_velocity_range(), 2..6);
        assert_eq!(region.half_levels(), 1..5);
        assert_eq!(region.full_levels(), 1..6);
    }

    #[test]
    fn requires_complete_vertical_column() {
        assert_eq!(
            AcousticStepPreparationRegion::try_new(
                GridShape::try_new(7, 6, 6).unwrap(),
                1..6,
                1..5,
                1..5,
                2..7,
                2..6,
                1..5,
            ),
            Err(AcousticStepPreparationError::IncompleteVerticalColumn {
                expected_start: 1,
                expected_end: 6,
                actual_start: 1,
                actual_end: 5,
            })
        );
    }
}
