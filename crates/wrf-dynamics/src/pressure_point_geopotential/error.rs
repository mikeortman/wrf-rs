use std::fmt;

use wrf_compute::GridShape;

use crate::{PressurePointGeopotentialAxis, PressurePointGeopotentialField};

/// Failure returned while validating or calculating pressure-point geopotential.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PressurePointGeopotentialError {
    /// A physical mass-domain range contains no points.
    EmptyMassDomainRange {
        /// Axis whose domain range is empty.
        axis: PressurePointGeopotentialAxis,
    },
    /// A physical mass-domain range exceeds field storage.
    MassDomainRangeOutOfBounds {
        /// Axis whose domain range is invalid.
        axis: PressurePointGeopotentialAxis,
        /// Exclusive range endpoint.
        range_end: usize,
        /// Available field extent.
        field_extent: usize,
    },
    /// The mass-level domain has no stored full level above it.
    MissingUpperFullLevel {
        /// Exclusive mass-domain end, also the required neighbor index.
        required_index: usize,
        /// Available bottom-top field extent.
        field_extent: usize,
    },
    /// An active tile range contains no points.
    EmptyTileRange {
        /// Axis whose tile range is empty.
        axis: PressurePointGeopotentialAxis,
    },
    /// An active tile range exceeds field storage.
    TileRangeOutOfBounds {
        /// Axis whose tile range is invalid.
        axis: PressurePointGeopotentialAxis,
        /// Exclusive range endpoint.
        range_end: usize,
        /// Available field extent.
        field_extent: usize,
    },
    /// An active tile lies outside the mass domain and its upper stagger point.
    TileOutsideMassDomain {
        /// Axis on which the tile is invalid.
        axis: PressurePointGeopotentialAxis,
    },
    /// A field shape differs from the shape required by the region.
    FieldShapeMismatch {
        /// Role of the mismatched field.
        field: PressurePointGeopotentialField,
        /// Required shape.
        expected: GridShape,
        /// Supplied shape.
        actual: GridShape,
    },
    /// A persistent CPU worker panicked.
    WorkerPanicked,
}

/// Result returned by pressure-point geopotential operations.
pub type PressurePointGeopotentialResult<Value> = Result<Value, PressurePointGeopotentialError>;

impl fmt::Display for PressurePointGeopotentialError {
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
                "{axis} mass-domain end {range_end} exceeds field extent {field_extent}"
            ),
            Self::MissingUpperFullLevel {
                required_index,
                field_extent,
            } => write!(
                formatter,
                "bottom-top mass-domain end requires full-level index {required_index}, but field extent is {field_extent}"
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
            Self::TileOutsideMassDomain { axis } => write!(
                formatter,
                "{axis} tile lies outside the mass domain and its upper stagger point"
            ),
            Self::FieldShapeMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "{field} shape {actual:?} differs from expected shape {expected:?}"
            ),
            Self::WorkerPanicked => {
                formatter.write_str("a pressure-point geopotential worker panicked")
            }
        }
    }
}

impl std::error::Error for PressurePointGeopotentialError {}
