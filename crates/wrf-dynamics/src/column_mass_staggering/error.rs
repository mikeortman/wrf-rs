use std::error::Error;
use std::fmt;

use crate::{ColumnMassStaggeringAxis, ColumnMassStaggeringField};

/// Result returned by column-mass staggering operations.
pub type ColumnMassStaggeringResult<Value> = Result<Value, ColumnMassStaggeringError>;

/// Validation or execution failure from column-mass staggering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ColumnMassStaggeringError {
    /// A mass-domain range contains no mass points.
    EmptyMassDomainRange {
        /// Axis whose domain range is empty.
        axis: ColumnMassStaggeringAxis,
    },
    /// A mass-domain upper boundary does not fit in field storage.
    MassDomainBoundaryOutOfBounds {
        /// Axis whose physical boundary is invalid.
        axis: ColumnMassStaggeringAxis,
        /// Momentum-point index at the upper physical boundary.
        boundary_index: usize,
        /// Available field extent.
        field_extent: usize,
    },
    /// An active tile range contains no momentum points.
    EmptyTileRange {
        /// Axis whose tile range is empty.
        axis: ColumnMassStaggeringAxis,
    },
    /// An active tile range extends beyond field storage.
    TileRangeOutOfBounds {
        /// Axis whose tile range is invalid.
        axis: ColumnMassStaggeringAxis,
        /// Exclusive range end.
        range_end: usize,
        /// Available field extent.
        field_extent: usize,
    },
    /// An active tile extends outside its physical mass domain.
    TileOutsideMassDomain {
        /// Axis on which the tile leaves the domain.
        axis: ColumnMassStaggeringAxis,
    },
    /// A periodic lower boundary has no preceding halo mass point.
    PeriodicLowerHaloMissing {
        /// Axis whose lower periodic halo is absent.
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
            Self::EmptyMassDomainRange { axis } => {
                write!(formatter, "{axis} mass-domain range is empty")
            }
            Self::MassDomainBoundaryOutOfBounds {
                axis,
                boundary_index,
                field_extent,
            } => write!(
                formatter,
                "{axis} mass-domain boundary index {boundary_index} is outside field extent {field_extent}"
            ),
            Self::EmptyTileRange { axis } => write!(formatter, "{axis} tile range is empty"),
            Self::TileRangeOutOfBounds {
                axis,
                range_end,
                field_extent,
            } => write!(
                formatter,
                "{axis} tile range ends at {range_end}, beyond field extent {field_extent}"
            ),
            Self::TileOutsideMassDomain { axis } => {
                write!(formatter, "{axis} tile extends outside its mass domain")
            }
            Self::PeriodicLowerHaloMissing { axis } => write!(
                formatter,
                "{axis} periodic lower boundary requires one preceding halo mass point"
            ),
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
