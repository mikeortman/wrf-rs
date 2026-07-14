use std::fmt;

use crate::PatchId;

/// A failure while planning or applying a halo exchange.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HaloExchangeError {
    /// A zero-width exchange was requested.
    ZeroHaloWidth,
    /// The exchange exceeded the topology's allocated halo width.
    HaloWidthExceedsAllocation {
        /// Requested exchange width.
        halo_width: usize,
        /// Maximum width present in patch memory.
        maximum_halo_width: usize,
    },
    /// Periodic storage was narrower than the requested exchange.
    PeriodicBoundaryStorageTooNarrow {
        /// Required periodic storage width, including staggering.
        halo_width: usize,
        /// Available physical-boundary storage width.
        boundary_width: usize,
    },
    /// Grid-index arithmetic exceeded `i32`.
    IndexArithmeticOverflow,
    /// A generated transfer escaped source or destination memory.
    TransferOutsideMemory {
        /// Patch whose memory does not contain the transfer.
        patch_id: PatchId,
    },
    /// A generated source and destination had different point counts.
    TransferShapeMismatch,
    /// A plan was applied to a field with a different topology.
    TopologyMismatch,
    /// A field allocation exceeded addressable memory.
    FieldPointCountOverflow,
    /// A patch identifier was not present in the field.
    UnknownPatch {
        /// Missing patch identifier.
        patch_id: PatchId,
    },
    /// A requested field coordinate was outside allocated patch memory.
    CoordinateOutsideMemory {
        /// Patch addressed by the caller.
        patch_id: PatchId,
        /// Requested west-east index.
        west_east: i32,
        /// Requested bottom-top index.
        bottom_top: i32,
        /// Requested south-north index.
        south_north: i32,
    },
    /// Packed values did not match the destination transfer shape.
    PackedValueCountMismatch {
        /// Destination value count.
        expected: usize,
        /// Supplied packed value count.
        actual: usize,
    },
    /// A generated one-dimensional transfer range was empty.
    EmptyTransferRange {
        /// Included lower index.
        start: i32,
        /// Excluded upper index.
        end: i32,
    },
    /// Validated topology lookup failed while building a plan.
    TopologyInconsistent,
}

/// Result returned by halo planning and execution.
pub type HaloExchangeResult<T> = Result<T, HaloExchangeError>;

impl fmt::Display for HaloExchangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroHaloWidth => formatter.write_str("halo width must be non-zero"),
            Self::HaloWidthExceedsAllocation {
                halo_width,
                maximum_halo_width,
            } => write!(
                formatter,
                "halo width {halo_width} exceeds allocated width {maximum_halo_width}"
            ),
            Self::PeriodicBoundaryStorageTooNarrow {
                halo_width,
                boundary_width,
            } => write!(
                formatter,
                "periodic halo width {halo_width} exceeds boundary storage {boundary_width}"
            ),
            Self::IndexArithmeticOverflow => {
                formatter.write_str("halo index arithmetic overflowed")
            }
            Self::TransferOutsideMemory { patch_id } => write!(
                formatter,
                "halo transfer escapes memory for patch {}",
                patch_id.value()
            ),
            Self::TransferShapeMismatch => {
                formatter.write_str("halo source and destination shapes differ")
            }
            Self::TopologyMismatch => formatter.write_str("halo plan and field topologies differ"),
            Self::FieldPointCountOverflow => {
                formatter.write_str("patch field size exceeds addressable memory")
            }
            Self::UnknownPatch { patch_id } => {
                write!(formatter, "patch {} is not in this field", patch_id.value())
            }
            Self::CoordinateOutsideMemory {
                patch_id,
                west_east,
                bottom_top,
                south_north,
            } => write!(
                formatter,
                "coordinate ({west_east}, {bottom_top}, {south_north}) is outside patch {} memory",
                patch_id.value()
            ),
            Self::PackedValueCountMismatch { expected, actual } => write!(
                formatter,
                "packed halo has {actual} values but destination needs {expected}"
            ),
            Self::EmptyTransferRange { start, end } => {
                write!(formatter, "halo transfer range {start}..{end} is empty")
            }
            Self::TopologyInconsistent => {
                formatter.write_str("validated topology contains an inconsistent patch lookup")
            }
        }
    }
}

impl std::error::Error for HaloExchangeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_reports_requested_and_allocated_halo_widths() {
        let error = HaloExchangeError::HaloWidthExceedsAllocation {
            halo_width: 4,
            maximum_halo_width: 2,
        };

        assert_eq!(error.to_string(), "halo width 4 exceeds allocated width 2");
    }
}
