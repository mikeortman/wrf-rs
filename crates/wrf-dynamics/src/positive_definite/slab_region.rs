use std::ops::Range;

use wrf_compute::GridShape;

use crate::{PositiveDefiniteError, PositiveDefiniteResult, PositiveDefiniteSlabAxis};

/// Validated active ranges for WRF's three-dimensional slab correction.
///
/// Ranges are zero-based and half-open in Rust. They refer to offsets within
/// the field's memory extents, so callers with non-one Fortran lower bounds
/// translate indices once while constructing the region.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PositiveDefiniteSlabRegion {
    shape: GridShape,
    west_east_range: Range<usize>,
    bottom_top_range: Range<usize>,
    south_north_range: Range<usize>,
}

impl PositiveDefiniteSlabRegion {
    /// Validates active slab ranges against a field shape.
    ///
    /// # Errors
    ///
    /// Returns an error when any range is empty or extends beyond its logical
    /// field dimension.
    pub fn try_new(
        shape: GridShape,
        west_east_range: Range<usize>,
        bottom_top_range: Range<usize>,
        south_north_range: Range<usize>,
    ) -> PositiveDefiniteResult<Self> {
        validate_range(
            PositiveDefiniteSlabAxis::WestEast,
            &west_east_range,
            shape.west_east_points(),
        )?;
        validate_range(
            PositiveDefiniteSlabAxis::BottomTop,
            &bottom_top_range,
            shape.bottom_top_points(),
        )?;
        validate_range(
            PositiveDefiniteSlabAxis::SouthNorth,
            &south_north_range,
            shape.south_north_points(),
        )?;

        Ok(Self {
            shape,
            west_east_range,
            bottom_top_range,
            south_north_range,
        })
    }

    /// Returns the field shape against which this region was validated.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    /// Returns the active west-east offsets.
    pub fn west_east_range(&self) -> Range<usize> {
        self.west_east_range.clone()
    }

    /// Returns the active bottom-top offsets.
    pub fn bottom_top_range(&self) -> Range<usize> {
        self.bottom_top_range.clone()
    }

    /// Returns the active south-north offsets.
    pub fn south_north_range(&self) -> Range<usize> {
        self.south_north_range.clone()
    }
}

fn validate_range(
    axis: PositiveDefiniteSlabAxis,
    range: &Range<usize>,
    field_extent: usize,
) -> PositiveDefiniteResult<()> {
    if range.start >= range.end {
        return Err(PositiveDefiniteError::EmptySlabRange { axis });
    }
    if range.end > field_extent {
        return Err(PositiveDefiniteError::SlabRangeOutOfBounds {
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
    fn try_new_validates_empty_and_out_of_bounds_ranges() {
        let shape = GridShape::try_new(6, 4, 5).unwrap();

        assert_eq!(
            PositiveDefiniteSlabRegion::try_new(shape, 1..1, 1..3, 1..3),
            Err(PositiveDefiniteError::EmptySlabRange {
                axis: PositiveDefiniteSlabAxis::WestEast,
            })
        );
        assert_eq!(
            PositiveDefiniteSlabRegion::try_new(shape, 1..7, 1..3, 1..3),
            Err(PositiveDefiniteError::SlabRangeOutOfBounds {
                axis: PositiveDefiniteSlabAxis::WestEast,
                range_end: 7,
                field_extent: 6,
            })
        );
    }
}
