use std::fmt;

use wrf_compute::GridShape;

use crate::{OmegaDiagnosisAxis, OmegaDiagnosisCoefficient, OmegaDiagnosisField};

/// Failure produced while validating or executing WRF omega diagnosis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OmegaDiagnosisError {
    /// A physical-domain range contains no points.
    EmptyDomainRange {
        /// Axis whose physical-domain range is empty.
        axis: OmegaDiagnosisAxis,
    },
    /// A physical-domain range extends beyond field storage.
    DomainRangeOutOfBounds {
        /// Axis whose range is invalid.
        axis: OmegaDiagnosisAxis,
        /// Exclusive invalid range end.
        range_end: usize,
        /// Available field extent.
        field_extent: usize,
    },
    /// A horizontal tile range contains no points.
    EmptyTileRange {
        /// Axis whose tile range is empty.
        axis: OmegaDiagnosisAxis,
    },
    /// A horizontal tile range extends beyond field storage.
    TileRangeOutOfBounds {
        /// Axis whose tile is invalid.
        axis: OmegaDiagnosisAxis,
        /// Exclusive invalid tile end.
        range_end: usize,
        /// Available field extent.
        field_extent: usize,
    },
    /// A horizontal tile lies outside the domain and its upper stagger point.
    TileOutsideDomain {
        /// Axis whose tile lies outside its allowed domain.
        axis: OmegaDiagnosisAxis,
    },
    /// An active tile lacks the lower neighbor used by C-grid averaging.
    MissingLowerNeighbor {
        /// Axis missing its lower neighbor.
        axis: OmegaDiagnosisAxis,
        /// Tile start that cannot be decremented safely.
        tile_start: usize,
    },
    /// An active tile lacks the upper neighbor used by flux divergence.
    MissingUpperNeighbor {
        /// Axis missing its upper neighbor.
        axis: OmegaDiagnosisAxis,
        /// Exclusive active end whose point is also read as a neighbor.
        active_end: usize,
        /// Available field extent.
        field_extent: usize,
    },
    /// The vertical tile is not the complete physical column including its top face.
    IncompleteVerticalColumn {
        /// Required inclusive-bottom offset.
        expected_start: usize,
        /// Required exclusive end after the top full level.
        expected_end: usize,
        /// Supplied inclusive-bottom offset.
        actual_start: usize,
        /// Supplied exclusive vertical end.
        actual_end: usize,
    },
    /// A field shape differs from the region contract.
    FieldShapeMismatch {
        /// Scientific role of the mismatched field.
        field: OmegaDiagnosisField,
        /// Shape required by the region.
        expected: GridShape,
        /// Shape supplied by the caller.
        actual: GridShape,
    },
    /// A vertical coefficient does not span allocated vertical storage.
    CoefficientLengthMismatch {
        /// Scientific role of the mismatched coefficient.
        coefficient: OmegaDiagnosisCoefficient,
        /// Required allocated vertical length.
        expected: usize,
        /// Supplied coefficient length.
        actual: usize,
    },
    /// A worker panicked while processing an independent horizontal plane.
    WorkerPanicked,
}

/// Result returned by omega-diagnosis operations.
pub type OmegaDiagnosisResult<Value> = Result<Value, OmegaDiagnosisError>;

impl fmt::Display for OmegaDiagnosisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDomainRange { axis } => {
                write!(formatter, "{axis} physical-domain range is empty")
            }
            Self::DomainRangeOutOfBounds {
                axis,
                range_end,
                field_extent,
            } => write!(
                formatter,
                "{axis} physical-domain end {range_end} exceeds field extent {field_extent}"
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
            Self::TileOutsideDomain { axis } => write!(
                formatter,
                "{axis} tile lies outside the physical domain and its upper stagger point"
            ),
            Self::MissingLowerNeighbor { axis, tile_start } => write!(
                formatter,
                "{axis} tile start {tile_start} lacks the lower neighbor required by C-grid averaging"
            ),
            Self::MissingUpperNeighbor {
                axis,
                active_end,
                field_extent,
            } => write!(
                formatter,
                "{axis} active end {active_end} lacks an upper neighbor in field extent {field_extent}"
            ),
            Self::IncompleteVerticalColumn {
                expected_start,
                expected_end,
                actual_start,
                actual_end,
            } => write!(
                formatter,
                "vertical tile {actual_start}..{actual_end} must cover the complete column {expected_start}..{expected_end}"
            ),
            Self::FieldShapeMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "{field} shape {actual:?} differs from expected shape {expected:?}"
            ),
            Self::CoefficientLengthMismatch {
                coefficient,
                expected,
                actual,
            } => write!(
                formatter,
                "{coefficient} length {actual} differs from expected length {expected}"
            ),
            Self::WorkerPanicked => formatter.write_str("an omega-diagnosis worker panicked"),
        }
    }
}

impl std::error::Error for OmegaDiagnosisError {}
