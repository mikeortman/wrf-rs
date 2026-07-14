use crate::{TopologyError, TopologyResult};

/// A signed, zero-based, half-open grid-index range.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IndexRange {
    start: i32,
    end: i32,
}

impl IndexRange {
    /// Creates a non-empty half-open range.
    pub fn try_new(start: i32, end: i32) -> TopologyResult<Self> {
        if start >= end {
            return Err(TopologyError::EmptyIndexRange { start, end });
        }
        Ok(Self { start, end })
    }

    /// Converts an inclusive, one-based Fortran range into this representation.
    pub fn try_from_fortran_inclusive(start: i32, end: i32) -> TopologyResult<Self> {
        if start > end {
            return Err(TopologyError::EmptyFortranRange { start, end });
        }
        let zero_based_start = start
            .checked_sub(1)
            .ok_or(TopologyError::IndexArithmeticOverflow)?;
        Ok(Self {
            start: zero_based_start,
            end,
        })
    }

    /// Returns the included lower index.
    pub const fn start(self) -> i32 {
        self.start
    }

    /// Returns the excluded upper index.
    pub const fn end(self) -> i32 {
        self.end
    }

    /// Returns the number of indices in the range.
    pub const fn len(self) -> usize {
        (self.end as i64 - self.start as i64) as usize
    }

    /// Returns whether the range is empty.
    ///
    /// Validated `IndexRange` values are never empty; this method is provided
    /// so the range follows Rust collection conventions.
    pub const fn is_empty(self) -> bool {
        false
    }

    /// Returns whether the range contains an index.
    pub const fn contains(self, index: i32) -> bool {
        index >= self.start && index < self.end
    }

    /// Returns whether this range fully contains another range.
    pub const fn contains_range(self, other: Self) -> bool {
        other.start >= self.start && other.end <= self.end
    }

    pub(crate) fn from_validated(start: i32, end: i32) -> Self {
        debug_assert!(start < end);
        Self { start, end }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fortran_conversion_preserves_inclusive_point_count() {
        let range = IndexRange::try_from_fortran_inclusive(-2, 4).unwrap();

        assert_eq!(range, IndexRange::try_new(-3, 4).unwrap());
        assert_eq!(range.len(), 7);
    }

    #[test]
    fn construction_rejects_empty_ranges() {
        assert_eq!(
            IndexRange::try_new(4, 4),
            Err(TopologyError::EmptyIndexRange { start: 4, end: 4 })
        );
    }

    #[test]
    fn length_handles_the_complete_signed_index_span() {
        let range = IndexRange::try_new(i32::MIN, i32::MAX).unwrap();

        assert_eq!(range.len(), u32::MAX as usize);
    }
}
