use std::error::Error;
use std::fmt;
use std::ops::Range;

use wrf_compute::GridShape;

use crate::{AcousticMassThetaAxis, AcousticMassThetaCoefficient, AcousticMassThetaField};

/// Failure reported before or during acoustic mass-and-theta advancement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AcousticMassThetaError {
    /// A physical-domain range is empty.
    EmptyDomainRange {
        /// Affected axis.
        axis: AcousticMassThetaAxis,
    },
    /// Storage lacks an upper stagger or full-level neighbor.
    MissingUpperNeighbor {
        /// Affected axis.
        axis: AcousticMassThetaAxis,
        /// Required upper point.
        boundary_index: usize,
        /// Allocated extent.
        field_extent: usize,
    },
    /// A tile range is empty.
    EmptyTileRange {
        /// Affected axis.
        axis: AcousticMassThetaAxis,
    },
    /// A tile range exceeds storage.
    TileRangeOutOfBounds {
        /// Affected axis.
        axis: AcousticMassThetaAxis,
        /// Exclusive range end.
        range_end: usize,
        /// Allocated extent.
        field_extent: usize,
    },
    /// A tile lies outside its mass domain plus upper stagger point.
    TileOutsideDomain {
        /// Affected axis.
        axis: AcousticMassThetaAxis,
    },
    /// The complete-column recurrence received a partial vertical tile.
    IncompleteVerticalColumn {
        /// Required complete tile.
        expected: Range<usize>,
        /// Supplied tile.
        actual: Range<usize>,
    },
    /// A horizontal scalar stencil lacks its west or south neighbor.
    MissingLowerNeighbor {
        /// Affected axis.
        axis: AcousticMassThetaAxis,
    },
    /// A field shape differs from the common region shape.
    FieldShapeMismatch {
        /// Scientific field role.
        field: AcousticMassThetaField,
        /// Required shape.
        expected: GridShape,
        /// Supplied shape.
        actual: GridShape,
    },
    /// A coefficient does not span the vertical storage extent.
    CoefficientLengthMismatch {
        /// Scientific coefficient role.
        coefficient: AcousticMassThetaCoefficient,
        /// Required length.
        expected: usize,
        /// Supplied length.
        actual: usize,
    },
    /// A CPU worker panicked while updating an owned output block.
    WorkerPanicked,
    /// The exact-block scheduler rejected validated storage.
    SchedulerContractViolated,
}

impl fmt::Display for AcousticMassThetaError {
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
            Self::MissingLowerNeighbor { axis } => {
                write!(formatter, "{axis} scalar stencil requires a lower neighbor")
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
            Self::WorkerPanicked => {
                formatter.write_str("an acoustic mass-and-theta worker panicked")
            }
            Self::SchedulerContractViolated => {
                formatter.write_str("validated acoustic mass-and-theta block shape was rejected")
            }
        }
    }
}

impl Error for AcousticMassThetaError {}

/// Result type for acoustic mass-and-theta advancement.
pub type AcousticMassThetaResult<T> = Result<T, AcousticMassThetaError>;
