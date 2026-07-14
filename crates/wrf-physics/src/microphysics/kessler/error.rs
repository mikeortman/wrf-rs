use std::error::Error;
use std::fmt;

use wrf_compute::GridShape;

use crate::{KesslerMicrophysicsAxis, KesslerMicrophysicsField, KesslerMicrophysicsParameter};

/// Failure produced while configuring or executing Kessler microphysics.
#[derive(Clone, Debug, PartialEq)]
pub enum KesslerMicrophysicsError {
    /// A required scalar parameter is non-finite or outside its valid range.
    InvalidParameter {
        /// Parameter that failed validation.
        parameter: KesslerMicrophysicsParameter,
        /// Rejected value.
        value: f32,
    },
    /// An active range contains no points.
    EmptyRange {
        /// Axis containing the empty range.
        axis: KesslerMicrophysicsAxis,
    },
    /// An active range extends beyond its allocated field dimension.
    RangeOutOfBounds {
        /// Axis containing the invalid range.
        axis: KesslerMicrophysicsAxis,
        /// Exclusive requested range end.
        range_end: usize,
        /// Allocated number of points on the axis.
        field_extent: usize,
    },
    /// The upstream routine requires sedimentation to begin at the surface level.
    BottomTopRangeMustStartAtSurface {
        /// Rejected zero-based range start.
        range_start: usize,
    },
    /// At least two active vertical levels are required for the top spacing.
    RequiresTwoVerticalLevels {
        /// Requested active vertical level count.
        level_count: usize,
    },
    /// A participating field has the wrong allocation shape.
    FieldShapeMismatch {
        /// Field whose shape differs.
        field: KesslerMicrophysicsField,
        /// Shape required by the region.
        expected: GridShape,
        /// Actual field shape.
        actual: GridShape,
    },
    /// Reusable scratch storage was created for a different field shape.
    WorkspaceShapeMismatch {
        /// Shape required by the region.
        expected: GridShape,
        /// Shape owned by the workspace.
        actual: GridShape,
    },
    /// The CPU backend could not allocate reusable scratch storage.
    WorkspaceAllocationFailed {
        /// Backend error message.
        message: String,
    },
    /// The operation escaped the backend's persistent worker pool.
    WorkerIndexUnavailable,
    /// A backend worker index exceeded the workspace's scratch allocation.
    WorkerIndexOutOfBounds {
        /// Worker index reported by Rayon.
        worker_index: usize,
        /// Number of allocated worker scratch entries.
        worker_count: usize,
    },
    /// Reusable column scratch was poisoned by an earlier worker panic.
    WorkspacePoisoned,
    /// A CPU worker panicked during execution.
    WorkerPanicked,
}

/// Result alias for Kessler configuration and execution.
pub type KesslerMicrophysicsResult<T> = Result<T, KesslerMicrophysicsError>;

impl fmt::Display for KesslerMicrophysicsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParameter { parameter, value } => {
                write!(formatter, "invalid Kessler {parameter}: {value}")
            }
            Self::EmptyRange { axis } => write!(formatter, "Kessler {axis} range is empty"),
            Self::RangeOutOfBounds {
                axis,
                range_end,
                field_extent,
            } => write!(
                formatter,
                "Kessler {axis} range ends at {range_end}, beyond field extent {field_extent}"
            ),
            Self::BottomTopRangeMustStartAtSurface { range_start } => write!(
                formatter,
                "Kessler bottom-top range starts at {range_start}; the pinned routine requires surface level 0"
            ),
            Self::RequiresTwoVerticalLevels { level_count } => write!(
                formatter,
                "Kessler requires at least two active vertical levels, got {level_count}"
            ),
            Self::FieldShapeMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "Kessler {field} shape {actual:?} does not match expected shape {expected:?}"
            ),
            Self::WorkspaceShapeMismatch { expected, actual } => write!(
                formatter,
                "Kessler workspace shape {actual:?} does not match expected shape {expected:?}"
            ),
            Self::WorkspaceAllocationFailed { message } => {
                write!(formatter, "failed to allocate Kessler workspace: {message}")
            }
            Self::WorkerIndexUnavailable => {
                formatter.write_str("Kessler operation ran outside the CPU worker pool")
            }
            Self::WorkerIndexOutOfBounds {
                worker_index,
                worker_count,
            } => write!(
                formatter,
                "Kessler worker index {worker_index} exceeds {worker_count} scratch entries"
            ),
            Self::WorkspacePoisoned => formatter.write_str("Kessler column workspace is poisoned"),
            Self::WorkerPanicked => formatter.write_str("a Kessler CPU worker panicked"),
        }
    }
}

impl Error for KesslerMicrophysicsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_names_invalid_parameter_and_value() {
        let error = KesslerMicrophysicsError::InvalidParameter {
            parameter: KesslerMicrophysicsParameter::TimeStep,
            value: 0.0,
        };

        assert_eq!(error.to_string(), "invalid Kessler time step: 0");
    }
}
