use std::error::Error;
use std::fmt;
use std::ops::Range;

use wrf_compute::GridShape;

use crate::{AcousticVerticalAxis, AcousticVerticalCoefficient, AcousticVerticalField};

/// Failure reported before or during acoustic vertical advancement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AcousticVerticalError {
    /// A physical-domain range is empty.
    EmptyDomainRange {
        /// Affected axis.
        axis: AcousticVerticalAxis,
    },
    /// Storage lacks an upper stencil or stagger point.
    MissingUpperNeighbor {
        /// Affected axis.
        axis: AcousticVerticalAxis,
        /// Required upper point.
        boundary_index: usize,
        /// Allocated extent.
        field_extent: usize,
    },
    /// A tile range is empty.
    EmptyTileRange {
        /// Affected axis.
        axis: AcousticVerticalAxis,
    },
    /// A tile range exceeds storage.
    TileRangeOutOfBounds {
        /// Affected axis.
        axis: AcousticVerticalAxis,
        /// Exclusive range end.
        range_end: usize,
        /// Allocated extent.
        field_extent: usize,
    },
    /// A tile lies outside its physical domain.
    TileOutsideDomain {
        /// Affected axis.
        axis: AcousticVerticalAxis,
    },
    /// The implicit recurrence received a partial vertical column.
    IncompleteVerticalColumn {
        /// Required complete tile.
        expected: Range<usize>,
        /// Supplied tile.
        actual: Range<usize>,
    },
    /// Surface reconstruction lacks its three required velocity levels.
    InsufficientVerticalLevels {
        /// Minimum mass-level count.
        required: usize,
        /// Supplied mass-level count.
        actual: usize,
    },
    /// A terrain stencil lacks its west or south neighbor.
    MissingLowerNeighbor {
        /// Affected axis.
        axis: AcousticVerticalAxis,
    },
    /// A field shape differs from the region contract.
    FieldShapeMismatch {
        /// Scientific field role.
        field: AcousticVerticalField,
        /// Required shape.
        expected: GridShape,
        /// Supplied shape.
        actual: GridShape,
    },
    /// A vertical coefficient does not span storage.
    CoefficientLengthMismatch {
        /// Scientific coefficient role.
        coefficient: AcousticVerticalCoefficient,
        /// Required length.
        expected: usize,
        /// Supplied length.
        actual: usize,
    },
    /// A CPU worker panicked.
    WorkerPanicked,
    /// The scheduler rejected prevalidated block storage.
    SchedulerContractViolated,
}

impl fmt::Display for AcousticVerticalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDomainRange { axis } => write!(formatter, "{axis} domain range is empty"),
            Self::MissingUpperNeighbor {
                axis,
                boundary_index,
                field_extent,
            } => write!(
                formatter,
                "{axis} upper neighbor {boundary_index} is outside field extent {field_extent}"
            ),
            Self::EmptyTileRange { axis } => write!(formatter, "{axis} tile range is empty"),
            Self::TileRangeOutOfBounds {
                axis,
                range_end,
                field_extent,
            } => write!(
                formatter,
                "{axis} tile end {range_end} exceeds field extent {field_extent}"
            ),
            Self::TileOutsideDomain { axis } => {
                write!(formatter, "{axis} tile lies outside its mass domain")
            }
            Self::IncompleteVerticalColumn { expected, actual } => write!(
                formatter,
                "vertical tile {actual:?} does not match required complete column {expected:?}"
            ),
            Self::InsufficientVerticalLevels { required, actual } => write!(
                formatter,
                "vertical column has {actual} mass levels but surface reconstruction requires {required}"
            ),
            Self::MissingLowerNeighbor { axis } => {
                write!(
                    formatter,
                    "{axis} terrain stencil requires a lower neighbor"
                )
            }
            Self::FieldShapeMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "{field} shape {actual:?} does not match required shape {expected:?}"
            ),
            Self::CoefficientLengthMismatch {
                coefficient,
                expected,
                actual,
            } => write!(
                formatter,
                "{coefficient} length {actual} does not match required length {expected}"
            ),
            Self::WorkerPanicked => formatter.write_str("an acoustic vertical worker panicked"),
            Self::SchedulerContractViolated => {
                formatter.write_str("validated acoustic vertical block shape was rejected")
            }
        }
    }
}

impl Error for AcousticVerticalError {}

/// Result type for acoustic vertical advancement.
pub type AcousticVerticalResult<T> = Result<T, AcousticVerticalError>;
