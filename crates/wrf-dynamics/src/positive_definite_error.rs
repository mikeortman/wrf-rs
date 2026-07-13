use std::fmt;

use crate::PositiveDefiniteSlabAxis;

/// A failure while applying WRF's positive-definite correction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PositiveDefiniteError {
    /// The sheet kernel received a field with more than one vertical level.
    SheetRequiresSingleVerticalLevel {
        /// Number of vertical levels in the supplied field.
        bottom_top_points: usize,
    },
    /// The sheet has a different number of lines than supplied target totals.
    LineTotalCountMismatch {
        /// Number of west-east lines in the sheet.
        line_count: usize,
        /// Number of target totals supplied by the caller.
        line_total_count: usize,
    },
    /// A slab region contains an empty half-open range.
    EmptySlabRange {
        /// Logical axis whose start is not less than its end.
        axis: PositiveDefiniteSlabAxis,
    },
    /// A slab region extends beyond the field shape used to construct it.
    SlabRangeOutOfBounds {
        /// Logical axis whose end exceeds its field extent.
        axis: PositiveDefiniteSlabAxis,
        /// Exclusive end of the requested range.
        range_end: usize,
        /// Number of points available on the axis.
        field_extent: usize,
    },
    /// A validated slab region was applied to a field with another shape.
    SlabFieldShapeMismatch,
    /// A CPU worker panicked while processing an independent line.
    WorkerPanicked,
}

/// The typed result returned by positive-definite kernels.
pub type PositiveDefiniteResult<T> = Result<T, PositiveDefiniteError>;

impl fmt::Display for PositiveDefiniteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SheetRequiresSingleVerticalLevel { bottom_top_points } => write!(
                formatter,
                "positive-definite sheet requires one vertical level, received {bottom_top_points}"
            ),
            Self::LineTotalCountMismatch {
                line_count,
                line_total_count,
            } => write!(
                formatter,
                "positive-definite sheet has {line_count} lines but {line_total_count} target totals"
            ),
            Self::EmptySlabRange { axis } => {
                write!(formatter, "positive-definite slab {axis} range is empty")
            }
            Self::SlabRangeOutOfBounds {
                axis,
                range_end,
                field_extent,
            } => write!(
                formatter,
                "positive-definite slab {axis} range ends at {range_end}, beyond field extent {field_extent}"
            ),
            Self::SlabFieldShapeMismatch => formatter.write_str(
                "positive-definite slab region was constructed for a different field shape",
            ),
            Self::WorkerPanicked => {
                formatter.write_str("a CPU worker panicked during positive-definite correction")
            }
        }
    }
}

impl std::error::Error for PositiveDefiniteError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mismatch_display_includes_both_counts() {
        let error = PositiveDefiniteError::LineTotalCountMismatch {
            line_count: 3,
            line_total_count: 2,
        };

        assert_eq!(
            error.to_string(),
            "positive-definite sheet has 3 lines but 2 target totals"
        );
    }
}
