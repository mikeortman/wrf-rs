use std::error::Error;
use std::fmt;

use wrf_compute::GridShape;

use crate::{
    AcousticStepPreparationAxis, AcousticStepPreparationCoefficient, AcousticStepPreparationField,
};

/// Result returned by acoustic-step preparation.
pub type AcousticStepPreparationResult<Value> = Result<Value, AcousticStepPreparationError>;

/// Validation or execution failure from acoustic-step preparation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AcousticStepPreparationError {
    /// A physical-domain range contains no points.
    EmptyDomainRange {
        /// Axis whose domain is empty.
        axis: AcousticStepPreparationAxis,
    },
    /// A physical-domain range exceeds field storage.
    DomainRangeOutOfBounds {
        /// Axis whose range is invalid.
        axis: AcousticStepPreparationAxis,
        /// Exclusive range endpoint.
        range_end: usize,
        /// Available field extent.
        field_extent: usize,
    },
    /// An active tile contains no points.
    EmptyTileRange {
        /// Axis whose tile is empty.
        axis: AcousticStepPreparationAxis,
    },
    /// An active tile exceeds field storage.
    TileRangeOutOfBounds {
        /// Axis whose tile is invalid.
        axis: AcousticStepPreparationAxis,
        /// Exclusive tile endpoint.
        range_end: usize,
        /// Available field extent.
        field_extent: usize,
    },
    /// A tile lies outside the mass domain and its upper stagger.
    TileOutsideDomain {
        /// Axis on which the tile is invalid.
        axis: AcousticStepPreparationAxis,
    },
    /// The vertical tile does not cover every half and full level.
    IncompleteVerticalColumn {
        /// Required first level.
        expected_start: usize,
        /// Required exclusive endpoint.
        expected_end: usize,
        /// Supplied first level.
        actual_start: usize,
        /// Supplied exclusive endpoint.
        actual_end: usize,
    },
    /// A later substep lacks a lower C-grid neighbor.
    MissingLowerNeighbor {
        /// Axis whose lower neighbor is missing.
        axis: AcousticStepPreparationAxis,
    },
    /// A field shape differs from the region shape.
    FieldShapeMismatch {
        /// Semantic role of the field.
        field: AcousticStepPreparationField,
        /// Required shape.
        expected: GridShape,
        /// Supplied shape.
        actual: GridShape,
    },
    /// A coefficient does not span vertical storage.
    CoefficientLengthMismatch {
        /// Semantic role of the coefficient.
        coefficient: AcousticStepPreparationCoefficient,
        /// Required value count.
        expected: usize,
        /// Supplied value count.
        actual: usize,
    },
    /// Validated output storage unexpectedly violated a scheduler contract.
    SchedulerContractViolated,
    /// A persistent CPU worker panicked.
    WorkerPanicked,
}

impl fmt::Display for AcousticStepPreparationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDomainRange { axis } => write!(formatter, "{axis} domain range is empty"),
            Self::DomainRangeOutOfBounds {
                axis,
                range_end,
                field_extent,
            } => write!(
                formatter,
                "{axis} domain range ends at {range_end}, beyond field extent {field_extent}"
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
            Self::TileOutsideDomain { axis } => write!(
                formatter,
                "{axis} tile lies outside the domain and its upper stagger"
            ),
            Self::IncompleteVerticalColumn {
                expected_start,
                expected_end,
                actual_start,
                actual_end,
            } => write!(
                formatter,
                "vertical tile {actual_start}..{actual_end} does not cover required full column {expected_start}..{expected_end}"
            ),
            Self::MissingLowerNeighbor { axis } => write!(
                formatter,
                "{axis} tile lacks the lower neighbor required on later substeps"
            ),
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
                "{coefficient} has {actual} values, expected {expected}"
            ),
            Self::SchedulerContractViolated => {
                formatter.write_str("validated scheduler contract was rejected")
            }
            Self::WorkerPanicked => {
                formatter.write_str("an acoustic-step preparation worker panicked")
            }
        }
    }
}

impl Error for AcousticStepPreparationError {}
