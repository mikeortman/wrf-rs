use std::error::Error;
use std::fmt;

use wrf_compute::GridShape;

use crate::{AcousticPressureAxis, AcousticPressureCoefficient, AcousticPressureField};

/// Failure reported before or during acoustic pressure diagnosis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AcousticPressureError {
    /// A physical-domain range is empty.
    EmptyDomainRange {
        /// Affected axis.
        axis: AcousticPressureAxis,
    },
    /// A physical-domain range exceeds allocated storage.
    DomainRangeOutOfBounds {
        /// Affected axis.
        axis: AcousticPressureAxis,
        /// Exclusive range end.
        range_end: usize,
        /// Allocated extent.
        field_extent: usize,
    },
    /// A tile range is empty.
    EmptyTileRange {
        /// Affected axis.
        axis: AcousticPressureAxis,
    },
    /// A tile range exceeds allocated storage.
    TileRangeOutOfBounds {
        /// Affected axis.
        axis: AcousticPressureAxis,
        /// Exclusive range end.
        range_end: usize,
        /// Allocated extent.
        field_extent: usize,
    },
    /// A tile begins below or extends beyond the domain plus its clipped endpoint.
    TileOutsideDomain {
        /// Affected axis.
        axis: AcousticPressureAxis,
    },
    /// The active half levels lack the geopotential level at `k + 1`.
    MissingUpperFullLevel {
        /// Required exclusive full-level end.
        required_end: usize,
        /// Allocated vertical extent.
        field_extent: usize,
    },
    /// A field shape differs from the validated region shape.
    FieldShapeMismatch {
        /// Scientific field role.
        field: AcousticPressureField,
        /// Required shape.
        expected: GridShape,
        /// Supplied shape.
        actual: GridShape,
    },
    /// A vertical coefficient does not span the storage vertical extent.
    CoefficientLengthMismatch {
        /// Scientific coefficient role.
        coefficient: AcousticPressureCoefficient,
        /// Required length.
        expected: usize,
        /// Supplied length.
        actual: usize,
    },
    /// A CPU worker panicked while updating disjoint output blocks.
    WorkerPanicked,
    /// The internal block scheduler rejected a shape established by validation.
    SchedulerContractViolated,
}

impl fmt::Display for AcousticPressureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDomainRange { axis } => write!(formatter, "{axis} domain range is empty"),
            Self::DomainRangeOutOfBounds {
                axis,
                range_end,
                field_extent,
            } => write!(
                formatter,
                "{axis} domain end {range_end} exceeds field extent {field_extent}"
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
                write!(formatter, "{axis} tile lies outside its physical domain")
            }
            Self::MissingUpperFullLevel {
                required_end,
                field_extent,
            } => write!(
                formatter,
                "active half levels require full-level end {required_end}, but vertical extent is {field_extent}"
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
                "{coefficient} length {actual} does not match required length {expected}"
            ),
            Self::WorkerPanicked => formatter.write_str("an acoustic-pressure worker panicked"),
            Self::SchedulerContractViolated => {
                formatter.write_str("validated acoustic-pressure block shape was rejected")
            }
        }
    }
}

impl Error for AcousticPressureError {}

/// Result type for acoustic pressure diagnosis.
pub type AcousticPressureResult<T> = Result<T, AcousticPressureError>;
