use std::ops::Range;

use wrf_compute::GridShape;

use crate::{HeldSuarezDampingAxis, HeldSuarezDampingError, HeldSuarezDampingResult};

/// Validated memory offsets and active ranges for Held-Suarez damping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeldSuarezDampingRegion {
    shape: GridShape,
    west_east_range: Range<usize>,
    bottom_top_range: Range<usize>,
    west_east_momentum_south_north_range: Range<usize>,
    south_north_momentum_south_north_range: Range<usize>,
    surface_level: usize,
}

impl HeldSuarezDampingRegion {
    /// Validates ranges, staggered neighbors, and the pressure reference level.
    ///
    /// Ranges use zero-based half-open memory offsets. The west-east range must
    /// have a preceding point for the west-east staggered pressure average;
    /// the south-north momentum range needs the analogous preceding row.
    ///
    /// # Errors
    ///
    /// Returns a typed error for empty/out-of-bounds ranges, missing staggered
    /// neighbors, or an out-of-bounds surface level.
    pub fn try_new(
        shape: GridShape,
        west_east_range: Range<usize>,
        bottom_top_range: Range<usize>,
        west_east_momentum_south_north_range: Range<usize>,
        south_north_momentum_south_north_range: Range<usize>,
        surface_level: usize,
    ) -> HeldSuarezDampingResult<Self> {
        validate_range(
            HeldSuarezDampingAxis::WestEast,
            &west_east_range,
            shape.west_east_points(),
        )?;
        validate_range(
            HeldSuarezDampingAxis::BottomTop,
            &bottom_top_range,
            shape.bottom_top_points(),
        )?;
        validate_range(
            HeldSuarezDampingAxis::WestEastMomentumSouthNorth,
            &west_east_momentum_south_north_range,
            shape.south_north_points(),
        )?;
        validate_range(
            HeldSuarezDampingAxis::SouthNorthMomentumSouthNorth,
            &south_north_momentum_south_north_range,
            shape.south_north_points(),
        )?;
        if west_east_range.start == 0 {
            return Err(HeldSuarezDampingError::MissingPrecedingNeighbor {
                axis: HeldSuarezDampingAxis::WestEast,
            });
        }
        if south_north_momentum_south_north_range.start == 0 {
            return Err(HeldSuarezDampingError::MissingPrecedingNeighbor {
                axis: HeldSuarezDampingAxis::SouthNorthMomentumSouthNorth,
            });
        }
        if surface_level >= shape.bottom_top_points() {
            return Err(HeldSuarezDampingError::SurfaceLevelOutOfBounds {
                surface_level,
                bottom_top_points: shape.bottom_top_points(),
            });
        }

        Ok(Self {
            shape,
            west_east_range,
            bottom_top_range,
            west_east_momentum_south_north_range,
            south_north_momentum_south_north_range,
            surface_level,
        })
    }

    /// Returns the field shape used for validation.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    pub(crate) fn west_east_range(&self) -> Range<usize> {
        self.west_east_range.clone()
    }

    pub(crate) fn bottom_top_range(&self) -> Range<usize> {
        self.bottom_top_range.clone()
    }

    pub(crate) fn west_east_momentum_south_north_range(&self) -> Range<usize> {
        self.west_east_momentum_south_north_range.clone()
    }

    pub(crate) fn south_north_momentum_south_north_range(&self) -> Range<usize> {
        self.south_north_momentum_south_north_range.clone()
    }

    pub(crate) const fn surface_level(&self) -> usize {
        self.surface_level
    }
}

fn validate_range(
    axis: HeldSuarezDampingAxis,
    range: &Range<usize>,
    field_extent: usize,
) -> HeldSuarezDampingResult<()> {
    if range.start >= range.end {
        return Err(HeldSuarezDampingError::EmptyRange { axis });
    }
    if range.end > field_extent {
        return Err(HeldSuarezDampingError::RangeOutOfBounds {
            axis,
            range_end: range.end,
            field_extent,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_rejects_empty_and_out_of_bounds_ranges() {
        let shape = GridShape::try_new(6, 6, 4).unwrap();

        assert_eq!(
            HeldSuarezDampingRegion::try_new(shape, 2..2, 1..4, 1..5, 1..5, 1),
            Err(HeldSuarezDampingError::EmptyRange {
                axis: HeldSuarezDampingAxis::WestEast,
            })
        );
        assert_eq!(
            HeldSuarezDampingRegion::try_new(shape, 1..5, 1..5, 1..5, 1..5, 1),
            Err(HeldSuarezDampingError::RangeOutOfBounds {
                axis: HeldSuarezDampingAxis::BottomTop,
                range_end: 5,
                field_extent: 4,
            })
        );
        assert_eq!(
            HeldSuarezDampingRegion::try_new(shape, 1..5, 1..4, 1..7, 1..5, 1),
            Err(HeldSuarezDampingError::RangeOutOfBounds {
                axis: HeldSuarezDampingAxis::WestEastMomentumSouthNorth,
                range_end: 7,
                field_extent: 6,
            })
        );
    }

    #[test]
    fn try_new_requires_staggered_preceding_neighbors() {
        let shape = GridShape::try_new(6, 6, 4).unwrap();

        assert_eq!(
            HeldSuarezDampingRegion::try_new(shape, 0..4, 1..4, 1..5, 1..5, 1),
            Err(HeldSuarezDampingError::MissingPrecedingNeighbor {
                axis: HeldSuarezDampingAxis::WestEast,
            })
        );
        assert_eq!(
            HeldSuarezDampingRegion::try_new(shape, 1..5, 1..4, 1..5, 0..4, 1),
            Err(HeldSuarezDampingError::MissingPrecedingNeighbor {
                axis: HeldSuarezDampingAxis::SouthNorthMomentumSouthNorth,
            })
        );
    }

    #[test]
    fn try_new_rejects_an_out_of_bounds_surface_level() {
        let shape = GridShape::try_new(6, 6, 4).unwrap();

        assert_eq!(
            HeldSuarezDampingRegion::try_new(shape, 1..5, 1..4, 1..5, 1..5, 4),
            Err(HeldSuarezDampingError::SurfaceLevelOutOfBounds {
                surface_level: 4,
                bottom_top_points: 4,
            })
        );
    }
}
