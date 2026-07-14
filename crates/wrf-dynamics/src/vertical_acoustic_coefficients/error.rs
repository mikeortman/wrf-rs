use std::error::Error;
use std::fmt;

use wrf_compute::GridShape;

use crate::{
    VerticalAcousticCoefficient, VerticalAcousticCoefficientAxis, VerticalAcousticCoefficientField,
};

/// Failure reported before or during vertical acoustic coefficient construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerticalAcousticCoefficientError {
    /// A physical-domain range is empty.
    EmptyDomainRange {
        /// Affected axis.
        axis: VerticalAcousticCoefficientAxis,
    },
    /// A physical-domain range exceeds allocated storage.
    DomainRangeOutOfBounds {
        /// Affected axis.
        axis: VerticalAcousticCoefficientAxis,
        /// Exclusive range end.
        range_end: usize,
        /// Allocated extent.
        field_extent: usize,
    },
    /// A horizontal tile range is empty.
    EmptyTileRange {
        /// Affected axis.
        axis: VerticalAcousticCoefficientAxis,
    },
    /// A horizontal tile range exceeds allocated storage.
    TileRangeOutOfBounds {
        /// Affected axis.
        axis: VerticalAcousticCoefficientAxis,
        /// Exclusive range end.
        range_end: usize,
        /// Allocated extent.
        field_extent: usize,
    },
    /// A horizontal tile lies outside its physical mass domain.
    TileOutsideDomain {
        /// Affected axis.
        axis: VerticalAcousticCoefficientAxis,
    },
    /// The half-level domain lacks its upper full-level coefficient point.
    MissingUpperFullLevel {
        /// Required exclusive storage end.
        required_end: usize,
        /// Allocated vertical extent.
        field_extent: usize,
    },
    /// A field shape differs from the validated region shape.
    FieldShapeMismatch {
        /// Scientific field role.
        field: VerticalAcousticCoefficientField,
        /// Required shape.
        expected: GridShape,
        /// Supplied shape.
        actual: GridShape,
    },
    /// A vertical coefficient does not span the storage vertical extent.
    CoefficientLengthMismatch {
        /// Scientific coefficient role.
        coefficient: VerticalAcousticCoefficient,
        /// Required length.
        expected: usize,
        /// Supplied length.
        actual: usize,
    },
    /// A CPU worker panicked while updating disjoint output planes.
    WorkerPanicked,
    /// The internal exact-plane scheduler rejected a validated shape.
    SchedulerContractViolated,
}

impl fmt::Display for VerticalAcousticCoefficientError {
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
                write!(
                    formatter,
                    "{axis} tile lies outside its physical mass domain"
                )
            }
            Self::MissingUpperFullLevel {
                required_end,
                field_extent,
            } => write!(
                formatter,
                "half levels require full-level storage end {required_end}, but the vertical extent is {field_extent}"
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
            Self::WorkerPanicked => {
                formatter.write_str("a vertical-acoustic coefficient worker panicked")
            }
            Self::SchedulerContractViolated => formatter
                .write_str("validated vertical-acoustic coefficient plane shape was rejected"),
        }
    }
}

impl Error for VerticalAcousticCoefficientError {}

/// Result type for vertical acoustic coefficient construction.
pub type VerticalAcousticCoefficientResult<T> = Result<T, VerticalAcousticCoefficientError>;
