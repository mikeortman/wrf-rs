use std::fmt;

use wrf_compute::GridShape;

use crate::{HeldSuarezDampingAxis, HeldSuarezDampingField};

/// A validation or execution failure in Held-Suarez damping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HeldSuarezDampingError {
    /// An active half-open range is empty.
    EmptyRange {
        /// Axis containing the invalid range.
        axis: HeldSuarezDampingAxis,
    },
    /// An active range extends beyond its field dimension.
    RangeOutOfBounds {
        /// Axis containing the invalid range.
        axis: HeldSuarezDampingAxis,
        /// Exclusive requested end.
        range_end: usize,
        /// Available field extent.
        field_extent: usize,
    },
    /// A staggered pressure average requires a preceding grid point.
    MissingPrecedingNeighbor {
        /// Axis whose active range begins at zero.
        axis: HeldSuarezDampingAxis,
    },
    /// The pressure reference level lies outside the vertical memory extent.
    SurfaceLevelOutOfBounds {
        /// Requested zero-based surface-level offset.
        surface_level: usize,
        /// Available bottom-top extent.
        bottom_top_points: usize,
    },
    /// A participating field does not match the region's validated shape.
    FieldShapeMismatch {
        /// Field with the unexpected shape.
        field: HeldSuarezDampingField,
        /// Shape against which the region was validated.
        expected: GridShape,
        /// Actual field shape.
        actual: GridShape,
    },
    /// A CPU worker panicked during the update.
    WorkerPanicked,
}

/// The typed result returned by Held-Suarez damping operations.
pub type HeldSuarezDampingResult<T> = Result<T, HeldSuarezDampingError>;

impl fmt::Display for HeldSuarezDampingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRange { axis } => write!(formatter, "Held-Suarez {axis} range is empty"),
            Self::RangeOutOfBounds {
                axis,
                range_end,
                field_extent,
            } => write!(
                formatter,
                "Held-Suarez {axis} range ends at {range_end}, beyond field extent {field_extent}"
            ),
            Self::MissingPrecedingNeighbor { axis } => write!(
                formatter,
                "Held-Suarez {axis} range requires a preceding pressure neighbor"
            ),
            Self::SurfaceLevelOutOfBounds {
                surface_level,
                bottom_top_points,
            } => write!(
                formatter,
                "Held-Suarez surface level {surface_level} is outside {bottom_top_points} vertical points"
            ),
            Self::FieldShapeMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "Held-Suarez {field} shape {actual:?} does not match {expected:?}"
            ),
            Self::WorkerPanicked => {
                formatter.write_str("a CPU worker panicked during Held-Suarez damping")
            }
        }
    }
}

impl std::error::Error for HeldSuarezDampingError {}
