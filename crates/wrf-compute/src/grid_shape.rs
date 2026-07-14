use crate::{ComputeError, ComputeResult};

/// A three-dimensional WRF field shape in logical domain order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GridShape {
    west_east_points: usize,
    south_north_points: usize,
    bottom_top_points: usize,
    point_count: usize,
}

impl GridShape {
    /// Validates dimensions and calculates their checked product.
    pub fn try_new(
        west_east_points: usize,
        south_north_points: usize,
        bottom_top_points: usize,
    ) -> ComputeResult<Self> {
        if west_east_points == 0 || south_north_points == 0 || bottom_top_points == 0 {
            return Err(ComputeError::EmptyGridDimension);
        }

        let horizontal_point_count = west_east_points
            .checked_mul(south_north_points)
            .ok_or(ComputeError::GridPointCountOverflow)?;
        let point_count = horizontal_point_count
            .checked_mul(bottom_top_points)
            .ok_or(ComputeError::GridPointCountOverflow)?;

        Ok(Self {
            west_east_points,
            south_north_points,
            bottom_top_points,
            point_count,
        })
    }

    /// Returns the number of points along the west-east dimension.
    pub const fn west_east_points(self) -> usize {
        self.west_east_points
    }

    /// Returns the number of points along the south-north dimension.
    pub const fn south_north_points(self) -> usize {
        self.south_north_points
    }

    /// Returns the number of points along the bottom-top dimension.
    pub const fn bottom_top_points(self) -> usize {
        self.bottom_top_points
    }

    /// Returns the checked product of all dimensions.
    pub const fn point_count(self) -> usize {
        self.point_count
    }

    /// Returns the matching horizontal field shape with one vertical level.
    pub const fn horizontal_shape(self) -> Self {
        Self {
            west_east_points: self.west_east_points,
            south_north_points: self.south_north_points,
            bottom_top_points: 1,
            point_count: self.point_count / self.bottom_top_points,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_calculates_checked_point_count() {
        let shape = GridShape::try_new(5, 7, 3).unwrap();

        assert_eq!(shape.point_count(), 105);
    }

    #[test]
    fn try_new_rejects_empty_and_overflowing_dimensions() {
        assert_eq!(
            GridShape::try_new(0, 2, 3),
            Err(ComputeError::EmptyGridDimension)
        );
        assert_eq!(
            GridShape::try_new(usize::MAX, 2, 1),
            Err(ComputeError::GridPointCountOverflow)
        );
    }

    #[test]
    fn horizontal_shape_preserves_horizontal_extents() {
        let shape = GridShape::try_new(5, 7, 3).unwrap();

        assert_eq!(
            shape.horizontal_shape(),
            GridShape::try_new(5, 7, 1).unwrap()
        );
    }
}
