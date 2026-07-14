use std::ops::Range;

use wrf_compute::GridShape;

use crate::{KesslerMicrophysicsAxis, KesslerMicrophysicsError, KesslerMicrophysicsResult};

/// Validated allocation and active tile ranges for Kessler microphysics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KesslerMicrophysicsRegion {
    field_shape: GridShape,
    precipitation_shape: GridShape,
    west_east_range: Range<usize>,
    south_north_range: Range<usize>,
    bottom_top_range: Range<usize>,
}

impl KesslerMicrophysicsRegion {
    /// Validates the three-dimensional field shape and active half-open ranges.
    ///
    /// WRF's pinned routine indexes sedimentation scratch from level one rather
    /// than `kts`; therefore the zero-based vertical range must begin at zero.
    pub fn try_new(
        field_shape: GridShape,
        west_east_range: Range<usize>,
        south_north_range: Range<usize>,
        bottom_top_range: Range<usize>,
    ) -> KesslerMicrophysicsResult<Self> {
        validate_range(
            KesslerMicrophysicsAxis::WestEast,
            &west_east_range,
            field_shape.west_east_points(),
        )?;
        validate_range(
            KesslerMicrophysicsAxis::SouthNorth,
            &south_north_range,
            field_shape.south_north_points(),
        )?;
        validate_range(
            KesslerMicrophysicsAxis::BottomTop,
            &bottom_top_range,
            field_shape.bottom_top_points(),
        )?;
        if bottom_top_range.start != 0 {
            return Err(KesslerMicrophysicsError::BottomTopRangeMustStartAtSurface {
                range_start: bottom_top_range.start,
            });
        }
        if bottom_top_range.len() < 2 {
            return Err(KesslerMicrophysicsError::RequiresTwoVerticalLevels {
                level_count: bottom_top_range.len(),
            });
        }

        let precipitation_shape = field_shape.horizontal_shape();

        Ok(Self {
            field_shape,
            precipitation_shape,
            west_east_range,
            south_north_range,
            bottom_top_range,
        })
    }

    /// Returns the required three-dimensional field shape.
    pub const fn field_shape(&self) -> GridShape {
        self.field_shape
    }

    /// Returns the required two-dimensional precipitation field shape.
    pub const fn precipitation_shape(&self) -> GridShape {
        self.precipitation_shape
    }

    pub(crate) fn west_east_range(&self) -> Range<usize> {
        self.west_east_range.clone()
    }

    pub(crate) fn south_north_range(&self) -> Range<usize> {
        self.south_north_range.clone()
    }

    pub(crate) fn bottom_top_range(&self) -> Range<usize> {
        self.bottom_top_range.clone()
    }
}

fn validate_range(
    axis: KesslerMicrophysicsAxis,
    range: &Range<usize>,
    field_extent: usize,
) -> KesslerMicrophysicsResult<()> {
    if range.is_empty() {
        return Err(KesslerMicrophysicsError::EmptyRange { axis });
    }
    if range.end > field_extent {
        return Err(KesslerMicrophysicsError::RangeOutOfBounds {
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
    fn derives_precipitation_shape_from_valid_active_ranges() {
        let shape = GridShape::try_new(6, 5, 4).unwrap();
        let region = KesslerMicrophysicsRegion::try_new(shape, 1..5, 1..4, 0..4).unwrap();

        assert_eq!(region.field_shape(), shape);
        assert_eq!(
            region.precipitation_shape(),
            GridShape::try_new(6, 5, 1).unwrap()
        );
    }

    #[test]
    fn rejects_empty_out_of_bounds_and_elevated_vertical_ranges() {
        let shape = GridShape::try_new(6, 5, 4).unwrap();

        assert_eq!(
            KesslerMicrophysicsRegion::try_new(shape, 2..2, 1..4, 0..4),
            Err(KesslerMicrophysicsError::EmptyRange {
                axis: KesslerMicrophysicsAxis::WestEast,
            })
        );
        assert_eq!(
            KesslerMicrophysicsRegion::try_new(shape, 1..7, 1..4, 0..4),
            Err(KesslerMicrophysicsError::RangeOutOfBounds {
                axis: KesslerMicrophysicsAxis::WestEast,
                range_end: 7,
                field_extent: 6,
            })
        );
        assert_eq!(
            KesslerMicrophysicsRegion::try_new(shape, 1..5, 1..4, 1..4),
            Err(KesslerMicrophysicsError::BottomTopRangeMustStartAtSurface { range_start: 1 })
        );
        assert_eq!(
            KesslerMicrophysicsRegion::try_new(shape, 1..5, 1..4, 0..1),
            Err(KesslerMicrophysicsError::RequiresTwoVerticalLevels { level_count: 1 })
        );
    }
}
