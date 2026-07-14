use std::error::Error;
use std::fmt;

use crate::{ColumnMassStaggeringAxis, ColumnMassStaggeringField};

/// Result returned by column-mass staggering operations.
pub type ColumnMassStaggeringResult<Value> = Result<Value, ColumnMassStaggeringError>;

/// Validation or execution failure from column-mass staggering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ColumnMassStaggeringError {
    /// A range contains no output points.
    EmptyRange {
        /// Axis whose range is empty.
        axis: ColumnMassStaggeringAxis,
    },
    /// A range extends beyond field storage.
    RangeOutOfBounds {
        /// Axis whose range is invalid.
        axis: ColumnMassStaggeringAxis,
        /// Exclusive range end.
        range_end: usize,
        /// Available field extent.
        field_extent: usize,
    },
    /// An interpolation range lacks its required preceding mass point.
    MissingPrecedingNeighbor {
        /// Axis on which the neighbor is missing.
        axis: ColumnMassStaggeringAxis,
    },
    /// A field does not match the shape validated by the region.
    FieldShapeMismatch {
        /// Role of the mismatched field.
        field: ColumnMassStaggeringField,
    },
    /// Column mass must be represented by a two-dimensional field.
    RequiresSingleVerticalLevel {
        /// Actual vertical extent.
        bottom_top_points: usize,
    },
    /// A persistent CPU worker panicked while processing an output row.
    WorkerPanicked,
}

impl fmt::Display for ColumnMassStaggeringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRange { axis } => write!(formatter, "{axis} output range is empty"),
            Self::RangeOutOfBounds {
                axis,
                range_end,
                field_extent,
            } => write!(
                formatter,
                "{axis} output range ends at {range_end}, beyond field extent {field_extent}"
            ),
            Self::MissingPrecedingNeighbor { axis } => {
                write!(
                    formatter,
                    "{axis} output range requires a preceding mass point"
                )
            }
            Self::FieldShapeMismatch { field } => {
                write!(
                    formatter,
                    "{field} shape does not match the staggering region"
                )
            }
            Self::RequiresSingleVerticalLevel { bottom_top_points } => write!(
                formatter,
                "column-mass staggering requires one vertical level, found {bottom_top_points}"
            ),
            Self::WorkerPanicked => formatter.write_str("a column-mass staggering worker panicked"),
        }
    }
}

impl Error for ColumnMassStaggeringError {}
