use std::fmt;

use wrf_compute::GridShape;

use crate::{MoistureCoefficientAxis, MoistureCoefficientField};

/// Failure returned while validating or calculating moisture coefficients.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MoistureCoefficientError {
    /// A physical-domain range contains no points.
    EmptyDomainRange {
        /// Axis whose range is empty.
        axis: MoistureCoefficientAxis,
    },
    /// A physical-domain range exceeds field storage.
    DomainRangeOutOfBounds {
        /// Axis whose range is invalid.
        axis: MoistureCoefficientAxis,
        /// Exclusive range endpoint.
        range_end: usize,
        /// Available field extent.
        field_extent: usize,
    },
    /// An active tile range contains no points.
    EmptyTileRange {
        /// Axis whose tile is empty.
        axis: MoistureCoefficientAxis,
    },
    /// An active tile range exceeds field storage.
    TileRangeOutOfBounds {
        /// Axis whose tile is invalid.
        axis: MoistureCoefficientAxis,
        /// Exclusive range endpoint.
        range_end: usize,
        /// Available field extent.
        field_extent: usize,
    },
    /// A tile lies outside the physical domain and its upper stagger point.
    TileOutsideDomain {
        /// Axis whose tile is invalid.
        axis: MoistureCoefficientAxis,
    },
    /// An active horizontal tile lacks the lower neighbor read by WRF.
    MissingLowerNeighbor {
        /// Axis on which the neighbor is missing.
        axis: MoistureCoefficientAxis,
        /// Tile start that would be decremented.
        tile_start: usize,
    },
    /// A mutable output field has the wrong shape.
    OutputShapeMismatch {
        /// Scientific role of the output.
        field: MoistureCoefficientField,
        /// Shape required by the region.
        expected: GridShape,
        /// Shape supplied by the caller.
        actual: GridShape,
    },
    /// One active moisture-species field has the wrong shape.
    SpeciesShapeMismatch {
        /// Zero-based index in the active-species slice.
        active_species_index: usize,
        /// Shape required by the region.
        expected: GridShape,
        /// Shape supplied by the caller.
        actual: GridShape,
    },
    /// A worker panicked while processing an independent output row.
    WorkerPanicked,
}

/// Result returned by moisture-coefficient operations.
pub type MoistureCoefficientResult<Value> = Result<Value, MoistureCoefficientError>;

impl fmt::Display for MoistureCoefficientError {
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
                "{axis} tile start {tile_start} lacks the lower neighbor required by moisture averaging"
            ),
            Self::OutputShapeMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "{field} shape {actual:?} differs from expected shape {expected:?}"
            ),
            Self::SpeciesShapeMismatch {
                active_species_index,
                expected,
                actual,
            } => write!(
                formatter,
                "active moisture species {active_species_index} shape {actual:?} differs from expected shape {expected:?}"
            ),
            Self::WorkerPanicked => formatter.write_str("a moisture-coefficient worker panicked"),
        }
    }
}

impl std::error::Error for MoistureCoefficientError {}
