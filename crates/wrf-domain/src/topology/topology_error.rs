use std::fmt;

use crate::PatchId;

/// A failure while validating or constructing domain topology.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TopologyError {
    /// A zero-based half-open range contained no indices.
    EmptyIndexRange {
        /// Included lower index.
        start: i32,
        /// Excluded upper index.
        end: i32,
    },
    /// An inclusive Fortran range contained no indices.
    EmptyFortranRange {
        /// Inclusive lower index.
        start: i32,
        /// Inclusive upper index.
        end: i32,
    },
    /// Signed index arithmetic exceeded `i32`.
    IndexArithmeticOverflow,
    /// A process-grid dimension was zero.
    ZeroProcessGridDimension,
    /// Multiplying process-grid dimensions exceeded `usize`.
    ProcessCountOverflow,
    /// More process columns were requested than west-east points.
    TooManyProcessColumns {
        /// Requested west-east process count.
        process_columns: usize,
        /// Available west-east point count.
        west_east_points: usize,
    },
    /// More process rows were requested than south-north points.
    TooManyProcessRows {
        /// Requested south-north process count.
        process_rows: usize,
        /// Available south-north point count.
        south_north_points: usize,
    },
    /// The maximum halo width did not fit signed index arithmetic.
    HaloWidthTooLarge {
        /// Requested maximum halo width.
        halo_width: usize,
    },
    /// A physical-boundary width did not fit signed index arithmetic.
    BoundaryWidthTooLarge {
        /// Requested physical-boundary width.
        boundary_width: usize,
    },
    /// A patch identifier was not part of the topology.
    UnknownPatch {
        /// Missing patch identifier.
        patch_id: PatchId,
    },
    /// A tile-grid dimension was zero.
    ZeroTileGridDimension,
    /// Requested tile execution escaped the patch's allocated memory.
    TileExecutionOutsideMemory {
        /// Patch whose requested tile bounds were invalid.
        patch_id: PatchId,
    },
}

/// Result returned by typed domain-topology operations.
pub type TopologyResult<T> = Result<T, TopologyError>;

impl fmt::Display for TopologyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyIndexRange { start, end } => {
                write!(formatter, "index range {start}..{end} must be non-empty")
            }
            Self::EmptyFortranRange { start, end } => write!(
                formatter,
                "inclusive Fortran range {start}:{end} must be non-empty"
            ),
            Self::IndexArithmeticOverflow => {
                formatter.write_str("grid index arithmetic overflowed")
            }
            Self::ZeroProcessGridDimension => {
                formatter.write_str("process-grid dimensions must be non-zero")
            }
            Self::ProcessCountOverflow => formatter.write_str("process-grid size overflowed"),
            Self::TooManyProcessColumns {
                process_columns,
                west_east_points,
            } => write!(
                formatter,
                "{process_columns} process columns exceed {west_east_points} west-east points"
            ),
            Self::TooManyProcessRows {
                process_rows,
                south_north_points,
            } => write!(
                formatter,
                "{process_rows} process rows exceed {south_north_points} south-north points"
            ),
            Self::HaloWidthTooLarge { halo_width } => {
                write!(
                    formatter,
                    "halo width {halo_width} exceeds signed grid indices"
                )
            }
            Self::BoundaryWidthTooLarge { boundary_width } => write!(
                formatter,
                "boundary width {boundary_width} exceeds signed grid indices"
            ),
            Self::UnknownPatch { patch_id } => {
                write!(
                    formatter,
                    "patch {} is not in this topology",
                    patch_id.value()
                )
            }
            Self::ZeroTileGridDimension => {
                formatter.write_str("tile-grid dimensions must be non-zero")
            }
            Self::TileExecutionOutsideMemory { patch_id } => write!(
                formatter,
                "tile execution for patch {} exceeds allocated memory",
                patch_id.value()
            ),
        }
    }
}

impl std::error::Error for TopologyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_names_invalid_process_columns() {
        let error = TopologyError::TooManyProcessColumns {
            process_columns: 5,
            west_east_points: 4,
        };

        assert_eq!(
            error.to_string(),
            "5 process columns exceed 4 west-east points"
        );
    }
}
