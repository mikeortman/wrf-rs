use std::ops::Range;

use wrf_compute::GridShape;

use crate::{
    AcousticStepFinalizationAxis, AcousticStepFinalizationError, AcousticStepFinalizationResult,
};

/// Validated physical mass domain and horizontal tile for `small_step_finish`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcousticStepFinalizationRegion {
    shape: GridShape,
    west_east_domain: Range<usize>,
    south_north_domain: Range<usize>,
    bottom_top_domain: Range<usize>,
    west_east_tile: Range<usize>,
    south_north_tile: Range<usize>,
}

impl AcousticStepFinalizationRegion {
    /// Validates mass ranges, one upper full level, and C-grid tile staggers.
    ///
    /// Horizontal tile ranges may extend one point beyond the mass domain for
    /// U or V staggering. Bottom-top storage must contain one full level above
    /// the half-open mass-level domain.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        west_east_domain: Range<usize>,
        south_north_domain: Range<usize>,
        bottom_top_domain: Range<usize>,
        west_east_tile: Range<usize>,
        south_north_tile: Range<usize>,
    ) -> AcousticStepFinalizationResult<Self> {
        for (axis, range, extent) in [
            (
                AcousticStepFinalizationAxis::WestEast,
                &west_east_domain,
                shape.west_east_points(),
            ),
            (
                AcousticStepFinalizationAxis::SouthNorth,
                &south_north_domain,
                shape.south_north_points(),
            ),
            (
                AcousticStepFinalizationAxis::BottomTop,
                &bottom_top_domain,
                shape.bottom_top_points(),
            ),
        ] {
            validate_domain(axis, range, extent)?;
        }
        if bottom_top_domain.end >= shape.bottom_top_points() {
            return Err(AcousticStepFinalizationError::MissingUpperFullLevel {
                required_index: bottom_top_domain.end,
                field_extent: shape.bottom_top_points(),
            });
        }
        validate_tile(
            AcousticStepFinalizationAxis::WestEast,
            &west_east_tile,
            &west_east_domain,
            shape.west_east_points(),
        )?;
        validate_tile(
            AcousticStepFinalizationAxis::SouthNorth,
            &south_north_tile,
            &south_north_domain,
            shape.south_north_points(),
        )?;
        Ok(Self {
            shape,
            west_east_domain,
            south_north_domain,
            bottom_top_domain,
            west_east_tile,
            south_north_tile,
        })
    }

    /// Returns the shared volume-field shape.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    pub(crate) fn mass_west_east_range(&self) -> Range<usize> {
        self.west_east_tile.start..self.west_east_tile.end.min(self.west_east_domain.end)
    }

    pub(crate) fn mass_south_north_range(&self) -> Range<usize> {
        self.south_north_tile.start..self.south_north_tile.end.min(self.south_north_domain.end)
    }

    pub(crate) fn west_east_velocity_range(&self) -> Range<usize> {
        self.west_east_tile.clone()
    }

    pub(crate) fn south_north_velocity_range(&self) -> Range<usize> {
        self.south_north_tile.clone()
    }

    pub(crate) fn mass_bottom_top_range(&self) -> Range<usize> {
        self.bottom_top_domain.clone()
    }

    pub(crate) fn full_bottom_top_range(&self) -> Range<usize> {
        self.bottom_top_domain.start..self.bottom_top_domain.end + 1
    }
}

fn validate_domain(
    axis: AcousticStepFinalizationAxis,
    range: &Range<usize>,
    field_extent: usize,
) -> AcousticStepFinalizationResult<()> {
    if range.is_empty() {
        return Err(AcousticStepFinalizationError::EmptyMassDomainRange { axis });
    }
    if range.end > field_extent {
        return Err(AcousticStepFinalizationError::MassDomainRangeOutOfBounds {
            axis,
            range: range.clone(),
            field_extent,
        });
    }
    Ok(())
}

fn validate_tile(
    axis: AcousticStepFinalizationAxis,
    tile: &Range<usize>,
    domain: &Range<usize>,
    field_extent: usize,
) -> AcousticStepFinalizationResult<()> {
    if tile.is_empty() {
        return Err(AcousticStepFinalizationError::EmptyTileRange { axis });
    }
    let permitted_end = domain.end.saturating_add(1).min(field_extent);
    if tile.start < domain.start || tile.end > permitted_end {
        return Err(AcousticStepFinalizationError::TileRangeOutOfBounds {
            axis,
            range: tile.clone(),
            permitted_end,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_one_mass_cell_with_upper_staggers() {
        let shape = GridShape::try_new(2, 2, 2).unwrap();
        let region =
            AcousticStepFinalizationRegion::try_new(shape, 0..1, 0..1, 0..1, 0..2, 0..2).unwrap();

        assert_eq!(region.mass_west_east_range(), 0..1);
        assert_eq!(region.west_east_velocity_range(), 0..2);
        assert_eq!(region.mass_south_north_range(), 0..1);
        assert_eq!(region.south_north_velocity_range(), 0..2);
        assert_eq!(region.mass_bottom_top_range(), 0..1);
        assert_eq!(region.full_bottom_top_range(), 0..2);
    }

    #[test]
    fn rejects_missing_full_level_and_bad_tiles() {
        let shape = GridShape::try_new(3, 3, 2).unwrap();
        assert_eq!(
            AcousticStepFinalizationRegion::try_new(shape, 0..2, 0..2, 0..2, 0..2, 0..2,),
            Err(AcousticStepFinalizationError::MissingUpperFullLevel {
                required_index: 2,
                field_extent: 2,
            })
        );
        assert_eq!(
            AcousticStepFinalizationRegion::try_new(shape, 0..2, 0..2, 0..1, 0..4, 0..2,),
            Err(AcousticStepFinalizationError::TileRangeOutOfBounds {
                axis: AcousticStepFinalizationAxis::WestEast,
                range: 0..4,
                permitted_end: 3,
            })
        );
    }
}
