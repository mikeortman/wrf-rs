use std::ops::Range;

use wrf_compute::GridShape;

use crate::{ColumnMassStaggeringAxis, ColumnMassStaggeringError, ColumnMassStaggeringResult};

/// Validated output ranges for column mass on the two horizontal staggerings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColumnMassStaggeringRegion {
    shape: GridShape,
    west_east_momentum_west_east_range: Range<usize>,
    west_east_momentum_south_north_range: Range<usize>,
    south_north_momentum_west_east_range: Range<usize>,
    south_north_momentum_south_north_range: Range<usize>,
}

impl ColumnMassStaggeringRegion {
    /// Validates the two momentum-grid output rectangles and required halos.
    ///
    /// Ranges are zero-based, half-open memory offsets. West-east momentum
    /// points average with the preceding west-east mass point; south-north
    /// momentum points analogously require a preceding row.
    pub fn try_new(
        shape: GridShape,
        west_east_momentum_west_east_range: Range<usize>,
        west_east_momentum_south_north_range: Range<usize>,
        south_north_momentum_west_east_range: Range<usize>,
        south_north_momentum_south_north_range: Range<usize>,
    ) -> ColumnMassStaggeringResult<Self> {
        if shape.bottom_top_points() != 1 {
            return Err(ColumnMassStaggeringError::RequiresSingleVerticalLevel {
                bottom_top_points: shape.bottom_top_points(),
            });
        }
        validate_range(
            ColumnMassStaggeringAxis::WestEast,
            &west_east_momentum_west_east_range,
            shape.west_east_points(),
        )?;
        validate_range(
            ColumnMassStaggeringAxis::SouthNorth,
            &west_east_momentum_south_north_range,
            shape.south_north_points(),
        )?;
        validate_range(
            ColumnMassStaggeringAxis::WestEast,
            &south_north_momentum_west_east_range,
            shape.west_east_points(),
        )?;
        validate_range(
            ColumnMassStaggeringAxis::SouthNorth,
            &south_north_momentum_south_north_range,
            shape.south_north_points(),
        )?;
        if west_east_momentum_west_east_range.start == 0 {
            return Err(ColumnMassStaggeringError::MissingPrecedingNeighbor {
                axis: ColumnMassStaggeringAxis::WestEast,
            });
        }
        if south_north_momentum_south_north_range.start == 0 {
            return Err(ColumnMassStaggeringError::MissingPrecedingNeighbor {
                axis: ColumnMassStaggeringAxis::SouthNorth,
            });
        }

        Ok(Self {
            shape,
            west_east_momentum_west_east_range,
            west_east_momentum_south_north_range,
            south_north_momentum_west_east_range,
            south_north_momentum_south_north_range,
        })
    }

    /// Returns the field shape used during validation.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    pub(crate) fn west_east_momentum_west_east_range(&self) -> Range<usize> {
        self.west_east_momentum_west_east_range.clone()
    }

    pub(crate) fn west_east_momentum_south_north_range(&self) -> Range<usize> {
        self.west_east_momentum_south_north_range.clone()
    }

    pub(crate) fn south_north_momentum_west_east_range(&self) -> Range<usize> {
        self.south_north_momentum_west_east_range.clone()
    }

    pub(crate) fn south_north_momentum_south_north_range(&self) -> Range<usize> {
        self.south_north_momentum_south_north_range.clone()
    }
}

fn validate_range(
    axis: ColumnMassStaggeringAxis,
    range: &Range<usize>,
    field_extent: usize,
) -> ColumnMassStaggeringResult<()> {
    if range.start >= range.end {
        return Err(ColumnMassStaggeringError::EmptyRange { axis });
    }
    if range.end > field_extent {
        return Err(ColumnMassStaggeringError::RangeOutOfBounds {
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
    fn rejects_non_planar_shapes_and_invalid_ranges() {
        let three_dimensional_shape = GridShape::try_new(4, 4, 2).unwrap();
        assert_eq!(
            ColumnMassStaggeringRegion::try_new(three_dimensional_shape, 1..4, 1..4, 1..4, 1..4,),
            Err(ColumnMassStaggeringError::RequiresSingleVerticalLevel {
                bottom_top_points: 2,
            })
        );

        let shape = GridShape::try_new(4, 4, 1).unwrap();
        assert_eq!(
            ColumnMassStaggeringRegion::try_new(shape, 1..1, 1..4, 1..4, 1..4),
            Err(ColumnMassStaggeringError::EmptyRange {
                axis: ColumnMassStaggeringAxis::WestEast,
            })
        );
        assert_eq!(
            ColumnMassStaggeringRegion::try_new(shape, 1..5, 1..4, 1..4, 1..4),
            Err(ColumnMassStaggeringError::RangeOutOfBounds {
                axis: ColumnMassStaggeringAxis::WestEast,
                range_end: 5,
                field_extent: 4,
            })
        );
    }

    #[test]
    fn requires_preceding_neighbors_only_on_the_staggered_axes() {
        let shape = GridShape::try_new(4, 4, 1).unwrap();
        assert_eq!(
            ColumnMassStaggeringRegion::try_new(shape, 0..4, 0..4, 0..4, 1..4),
            Err(ColumnMassStaggeringError::MissingPrecedingNeighbor {
                axis: ColumnMassStaggeringAxis::WestEast,
            })
        );
        assert_eq!(
            ColumnMassStaggeringRegion::try_new(shape, 1..4, 0..4, 0..4, 0..4),
            Err(ColumnMassStaggeringError::MissingPrecedingNeighbor {
                axis: ColumnMassStaggeringAxis::SouthNorth,
            })
        );
    }
}
