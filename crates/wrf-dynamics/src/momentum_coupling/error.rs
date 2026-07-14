use std::error::Error;
use std::fmt;

use wrf_compute::GridShape;

use crate::{MomentumCouplingAxis, MomentumCouplingCoefficient, MomentumCouplingField};

/// Result returned by momentum-coupling operations.
pub type MomentumCouplingResult<Value> = Result<Value, MomentumCouplingError>;

/// Validation or execution failure from momentum coupling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MomentumCouplingError {
    /// A physical mass-domain range contains no points.
    EmptyMassDomainRange {
        /// Axis whose domain range is empty.
        axis: MomentumCouplingAxis,
    },
    /// A physical mass-domain range exceeds field storage.
    MassDomainRangeOutOfBounds {
        /// Axis whose domain range is invalid.
        axis: MomentumCouplingAxis,
        /// Exclusive range endpoint.
        range_end: usize,
        /// Available field extent.
        field_extent: usize,
    },
    /// An active tile range contains no points.
    EmptyTileRange {
        /// Axis whose tile range is empty.
        axis: MomentumCouplingAxis,
    },
    /// An active tile range exceeds field storage.
    TileRangeOutOfBounds {
        /// Axis whose tile range is invalid.
        axis: MomentumCouplingAxis,
        /// Exclusive range endpoint.
        range_end: usize,
        /// Available field extent.
        field_extent: usize,
    },
    /// An active tile lies outside the physical domain and its upper stagger.
    TileOutsideMassDomain {
        /// Axis on which the tile is invalid.
        axis: MomentumCouplingAxis,
    },
    /// A field shape differs from the shape required by the region.
    FieldShapeMismatch {
        /// Role of the mismatched field.
        field: MomentumCouplingField,
        /// Required shape.
        expected: GridShape,
        /// Supplied shape.
        actual: GridShape,
    },
    /// A vertical coefficient array does not span field storage.
    CoefficientLengthMismatch {
        /// Role of the mismatched coefficient.
        coefficient: MomentumCouplingCoefficient,
        /// Required element count.
        expected: usize,
        /// Supplied element count.
        actual: usize,
    },
    /// A persistent CPU worker panicked.
    WorkerPanicked,
}

impl fmt::Display for MomentumCouplingError {
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
            Self::WorkerPanicked => formatter.write_str("a momentum-coupling worker panicked"),
        }
    }
}

impl Error for MomentumCouplingError {}
