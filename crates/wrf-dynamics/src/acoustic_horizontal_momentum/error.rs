use std::error::Error;
use std::fmt;

use wrf_compute::GridShape;

use crate::{
    AcousticHorizontalMomentumAxis, AcousticHorizontalMomentumCoefficient,
    AcousticHorizontalMomentumField,
};

/// Failure reported before or during acoustic horizontal-momentum advancement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AcousticHorizontalMomentumError {
    /// A physical-domain range is empty.
    EmptyDomainRange {
        /// Affected axis.
        axis: AcousticHorizontalMomentumAxis,
    },
    /// Storage lacks the upper U, V, or full-level point.
    MissingUpperStaggerPoint {
        /// Affected axis.
        axis: AcousticHorizontalMomentumAxis,
        /// Required upper point.
        boundary_index: usize,
        /// Allocated extent.
        field_extent: usize,
    },
    /// A tile range is empty.
    EmptyTileRange {
        /// Affected axis.
        axis: AcousticHorizontalMomentumAxis,
    },
    /// A tile range exceeds storage.
    TileRangeOutOfBounds {
        /// Affected axis.
        axis: AcousticHorizontalMomentumAxis,
        /// Exclusive range end.
        range_end: usize,
        /// Allocated extent.
        field_extent: usize,
    },
    /// A tile lies outside the mass domain plus its upper stagger point.
    TileOutsideDomain {
        /// Affected axis.
        axis: AcousticHorizontalMomentumAxis,
    },
    /// A pressure-gradient stencil lacks its west or south neighbor.
    MissingLowerNeighbor {
        /// Affected axis.
        axis: AcousticHorizontalMomentumAxis,
    },
    /// A relaxation width exceeds the corresponding mass domain.
    RelaxationZoneTooWide {
        /// Affected axis.
        axis: AcousticHorizontalMomentumAxis,
        /// Requested zone width.
        width: usize,
        /// Physical mass-point count.
        domain_points: usize,
    },
    /// Nonhydrostatic boundary interpolation requires three half levels.
    InsufficientNonhydrostaticLevels {
        /// Available half-level count.
        available: usize,
    },
    /// A field shape differs from the common region shape.
    FieldShapeMismatch {
        /// Scientific field role.
        field: AcousticHorizontalMomentumField,
        /// Required shape.
        expected: GridShape,
        /// Supplied shape.
        actual: GridShape,
    },
    /// A vertical coefficient does not span the storage vertical extent.
    CoefficientLengthMismatch {
        /// Scientific coefficient role.
        coefficient: AcousticHorizontalMomentumCoefficient,
        /// Required length.
        expected: usize,
        /// Supplied length.
        actual: usize,
    },
    /// A CPU worker panicked while updating an owned output plane.
    WorkerPanicked,
    /// The exact-plane scheduler rejected a validated shape.
    SchedulerContractViolated,
}

impl fmt::Display for AcousticHorizontalMomentumError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDomainRange { axis } => write!(formatter, "{axis} domain range is empty"),
            Self::MissingUpperStaggerPoint {
                axis,
                boundary_index,
                field_extent,
            } => write!(
                formatter,
                "{axis} upper stagger point {boundary_index} is outside field extent {field_extent}"
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
            Self::MissingLowerNeighbor { axis } => {
                write!(
                    formatter,
                    "{axis} pressure gradient requires a lower neighbor"
                )
            }
            Self::RelaxationZoneTooWide {
                axis,
                width,
                domain_points,
            } => write!(
                formatter,
                "{axis} relaxation width {width} exceeds {domain_points} mass points"
            ),
            Self::InsufficientNonhydrostaticLevels { available } => write!(
                formatter,
                "nonhydrostatic pressure interpolation requires at least three half levels, found {available}"
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
                formatter.write_str("an acoustic horizontal-momentum worker panicked")
            }
            Self::SchedulerContractViolated => formatter
                .write_str("validated acoustic horizontal-momentum plane shape was rejected"),
        }
    }
}

impl Error for AcousticHorizontalMomentumError {}

/// Result type for acoustic horizontal-momentum advancement.
pub type AcousticHorizontalMomentumResult<T> = Result<T, AcousticHorizontalMomentumError>;
