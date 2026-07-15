use std::error::Error;
use std::fmt;
use std::ops::Range;

use wrf_compute::GridShape;

use crate::{
    AcousticStepFinalizationAxis, AcousticStepFinalizationCoefficient,
    AcousticStepFinalizationField,
};

/// Failure reported while configuring or executing acoustic finalization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AcousticStepFinalizationError {
    /// The accepted acoustic trajectory contains no substeps.
    ZeroSubstepCount,
    /// A physical mass-domain range is empty.
    EmptyMassDomainRange {
        /// Axis whose range is empty.
        axis: AcousticStepFinalizationAxis,
    },
    /// A physical mass-domain range exceeds field storage.
    MassDomainRangeOutOfBounds {
        /// Axis whose range is invalid.
        axis: AcousticStepFinalizationAxis,
        /// Invalid half-open range.
        range: Range<usize>,
        /// Stored extent on that axis.
        field_extent: usize,
    },
    /// No full-level point exists above the active mass levels.
    MissingUpperFullLevel {
        /// Required full-level index.
        required_index: usize,
        /// Stored bottom-top extent.
        field_extent: usize,
    },
    /// An active horizontal tile range is empty.
    EmptyTileRange {
        /// Axis whose tile is empty.
        axis: AcousticStepFinalizationAxis,
    },
    /// A horizontal tile exceeds storage or its mass domain plus upper stagger.
    TileRangeOutOfBounds {
        /// Axis whose tile is invalid.
        axis: AcousticStepFinalizationAxis,
        /// Invalid half-open tile range.
        range: Range<usize>,
        /// Largest permitted exclusive endpoint.
        permitted_end: usize,
    },
    /// A field shape differs from the validated role shape.
    FieldShapeMismatch {
        /// Scientific role of the field.
        field: AcousticStepFinalizationField,
        /// Required shape.
        expected: GridShape,
        /// Supplied shape.
        actual: GridShape,
    },
    /// A vertical coefficient has the wrong length.
    CoefficientLengthMismatch {
        /// Coefficient role.
        coefficient: AcousticStepFinalizationCoefficient,
        /// Required number of values.
        expected: usize,
        /// Supplied number of values.
        actual: usize,
    },
    /// A persistent CPU worker panicked.
    WorkerPanicked,
}

impl fmt::Display for AcousticStepFinalizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroSubstepCount => {
                formatter.write_str("acoustic finalization requires at least one substep")
            }
            Self::EmptyMassDomainRange { axis } => write!(formatter, "{axis} mass domain is empty"),
            Self::MassDomainRangeOutOfBounds {
                axis,
                range,
                field_extent,
            } => write!(
                formatter,
                "{axis} mass domain {range:?} exceeds field extent {field_extent}"
            ),
            Self::MissingUpperFullLevel {
                required_index,
                field_extent,
            } => write!(
                formatter,
                "full-level index {required_index} is required, but bottom-top extent is {field_extent}"
            ),
            Self::EmptyTileRange { axis } => write!(formatter, "{axis} tile is empty"),
            Self::TileRangeOutOfBounds {
                axis,
                range,
                permitted_end,
            } => write!(
                formatter,
                "{axis} tile {range:?} exceeds permitted endpoint {permitted_end}"
            ),
            Self::FieldShapeMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "{field} shape {actual:?} differs from {expected:?}"
            ),
            Self::CoefficientLengthMismatch {
                coefficient,
                expected,
                actual,
            } => write!(
                formatter,
                "{coefficient} has {actual} values, expected {expected}"
            ),
            Self::WorkerPanicked => formatter.write_str("an acoustic-finalization worker panicked"),
        }
    }
}

impl Error for AcousticStepFinalizationError {}

/// Result type for acoustic-step finalization.
pub type AcousticStepFinalizationResult<Value> = Result<Value, AcousticStepFinalizationError>;
