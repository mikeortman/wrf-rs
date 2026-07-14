use std::ops::Range;

use wrf_compute::FieldStorage;

/// A field plus the model-storage ranges represented by its native allocation.
///
/// Full patch fields normally use ranges beginning at zero. Halo-extended tile
/// fields use the exact tile-plus-neighbor ranges, matching WRF's
/// `relax_bdytend_tile` contract without copying into patch-sized storage.
pub struct SpecifiedBoundaryRelaxationField<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) field: &'a Field,
    pub(crate) west_east: Range<usize>,
    pub(crate) south_north: Range<usize>,
    pub(crate) bottom_top: Range<usize>,
}

impl<Field> Clone for SpecifiedBoundaryRelaxationField<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        Self {
            field: self.field,
            west_east: self.west_east.clone(),
            south_north: self.south_north.clone(),
            bottom_top: self.bottom_top.clone(),
        }
    }
}

impl<'a, Field> SpecifiedBoundaryRelaxationField<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Associates native field storage with its model-coordinate ranges.
    pub const fn new(
        field: &'a Field,
        west_east: Range<usize>,
        south_north: Range<usize>,
        bottom_top: Range<usize>,
    ) -> Self {
        Self {
            field,
            west_east,
            south_north,
            bottom_top,
        }
    }

    /// Returns the west–east model-storage coverage.
    pub fn west_east_range(&self) -> &Range<usize> {
        &self.west_east
    }

    /// Returns the south–north model-storage coverage.
    pub fn south_north_range(&self) -> &Range<usize> {
        &self.south_north
    }

    /// Returns the bottom–top model-storage coverage.
    pub fn bottom_top_range(&self) -> &Range<usize> {
        &self.bottom_top
    }
}
