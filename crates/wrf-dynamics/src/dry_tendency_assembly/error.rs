use std::error::Error;
use std::fmt;

use wrf_compute::GridShape;

use crate::{DryTendencyAssemblyAxis, DryTendencyAssemblyCoefficient, DryTendencyAssemblyField};

/// Result returned by dry-tendency assembly.
pub type DryTendencyAssemblyResult<Value> = Result<Value, DryTendencyAssemblyError>;

/// Validation or execution failure from dry-tendency assembly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DryTendencyAssemblyError {
    /// A physical mass-domain range contains no points.
    EmptyMassDomainRange {
        /// Axis whose domain range is empty.
        axis: DryTendencyAssemblyAxis,
    },
    /// A mass-domain range exceeds field storage.
    MassDomainRangeOutOfBounds {
        /// Axis whose range is invalid.
        axis: DryTendencyAssemblyAxis,
        /// Exclusive endpoint of the supplied range.
        range_end: usize,
        /// Available field extent on the axis.
        field_extent: usize,
    },
    /// An active tile range contains no points.
    EmptyTileRange {
        /// Axis whose tile range is empty.
        axis: DryTendencyAssemblyAxis,
    },
    /// An active tile range exceeds field storage.
    TileRangeOutOfBounds {
        /// Axis whose range is invalid.
        axis: DryTendencyAssemblyAxis,
        /// Exclusive endpoint of the supplied range.
        range_end: usize,
        /// Available field extent on the axis.
        field_extent: usize,
    },
    /// An active tile lies outside the mass domain plus one upper stagger.
    TileOutsideMassDomain {
        /// Axis on which the tile is invalid.
        axis: DryTendencyAssemblyAxis,
    },
    /// A field shape differs from the region's required shape.
    FieldShapeMismatch {
        /// Semantic role of the mismatched field.
        field: DryTendencyAssemblyField,
        /// Shape required by the validated region.
        expected: GridShape,
        /// Shape supplied by the caller.
        actual: GridShape,
    },
    /// A vertical coefficient does not span vertical field storage.
    CoefficientLengthMismatch {
        /// Semantic role of the mismatched coefficient.
        coefficient: DryTendencyAssemblyCoefficient,
        /// Required number of vertical values.
        expected: usize,
        /// Number of values supplied by the caller.
        actual: usize,
    },
    /// The validated scheduler contract was unexpectedly rejected.
    SchedulerContractViolated,
    /// A persistent CPU worker panicked.
    WorkerPanicked,
}

impl fmt::Display for DryTendencyAssemblyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMassDomainRange { axis } => {
                write!(formatter, "{axis} mass-domain range is empty")
            }
            Self::MassDomainRangeOutOfBounds {
                axis,
                range_end,
                field_extent,
            } => write!(
                formatter,
                "{axis} mass-domain range ends at {range_end}, beyond field extent {field_extent}"
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
            Self::TileOutsideMassDomain { axis } => write!(
                formatter,
                "{axis} tile lies outside the mass domain and its upper stagger"
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
                formatter.write_str("validated paired-output scheduler contract was rejected")
            }
            Self::WorkerPanicked => formatter.write_str("a dry-tendency assembly worker panicked"),
        }
    }
}

impl Error for DryTendencyAssemblyError {}
