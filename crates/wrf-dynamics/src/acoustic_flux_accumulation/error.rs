use std::error::Error;
use std::fmt;

use wrf_compute::GridShape;

use crate::{AcousticFluxCoefficient, AcousticFluxField};

/// Failure reported before or during acoustic mass-flux accumulation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AcousticFluxAccumulationError {
    /// A physical-domain range is empty.
    EmptyDomainRange,
    /// Storage lacks one upper stagger point.
    MissingUpperStaggerPoint {
        /// Required upper point.
        boundary_index: usize,
        /// Allocated extent.
        field_extent: usize,
    },
    /// A tile range is empty.
    EmptyTileRange,
    /// A tile exceeds storage.
    TileRangeOutOfBounds {
        /// Exclusive range end.
        range_end: usize,
        /// Allocated extent.
        field_extent: usize,
    },
    /// A tile lies outside its physical domain plus upper stagger point.
    TileOutsideDomain,
    /// The acoustic sequence contains no substeps.
    ZeroSubstepCount,
    /// The one-based iteration is outside the sequence.
    SubstepOutOfRange {
        /// Supplied one-based iteration.
        iteration: usize,
        /// Total substep count.
        count: usize,
    },
    /// A field shape differs from its required shape.
    FieldShapeMismatch {
        /// Scientific field role.
        field: AcousticFluxField,
        /// Required shape.
        expected: GridShape,
        /// Supplied shape.
        actual: GridShape,
    },
    /// A coefficient does not span the vertical storage extent.
    CoefficientLengthMismatch {
        /// Scientific coefficient role.
        coefficient: AcousticFluxCoefficient,
        /// Required length.
        expected: usize,
        /// Supplied length.
        actual: usize,
    },
    /// A CPU worker panicked.
    WorkerPanicked,
    /// The scheduler rejected a validated block shape.
    SchedulerContractViolated,
}

impl fmt::Display for AcousticFluxAccumulationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDomainRange => formatter.write_str("physical domain range is empty"),
            Self::MissingUpperStaggerPoint {
                boundary_index,
                field_extent,
            } => write!(
                formatter,
                "upper stagger point {boundary_index} is outside field extent {field_extent}"
            ),
            Self::EmptyTileRange => formatter.write_str("tile range is empty"),
            Self::TileRangeOutOfBounds {
                range_end,
                field_extent,
            } => write!(
                formatter,
                "tile end {range_end} exceeds field extent {field_extent}"
            ),
            Self::TileOutsideDomain => formatter.write_str("tile lies outside its physical domain"),
            Self::ZeroSubstepCount => formatter.write_str("acoustic substep count must be nonzero"),
            Self::SubstepOutOfRange { iteration, count } => write!(
                formatter,
                "acoustic iteration {iteration} is outside 1..={count}"
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
            Self::WorkerPanicked => formatter.write_str("an acoustic flux worker panicked"),
            Self::SchedulerContractViolated => {
                formatter.write_str("validated acoustic flux block shape was rejected")
            }
        }
    }
}

impl Error for AcousticFluxAccumulationError {}

/// Result type for acoustic mass-flux accumulation.
pub type AcousticFluxAccumulationResult<T> = Result<T, AcousticFluxAccumulationError>;
