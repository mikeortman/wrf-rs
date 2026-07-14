use crate::{ExchangeAxis, ExchangeDirection, HorizontalBounds, IndexRange, PatchId};

/// One transport-neutral, contiguous-box halo message.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HaloTransfer {
    source_patch_id: PatchId,
    destination_patch_id: PatchId,
    axis: ExchangeAxis,
    direction: ExchangeDirection,
    source: HorizontalBounds,
    destination: HorizontalBounds,
    bottom_top: IndexRange,
}

impl HaloTransfer {
    pub(crate) const fn new(
        source_patch_id: PatchId,
        destination_patch_id: PatchId,
        axis: ExchangeAxis,
        direction: ExchangeDirection,
        source: HorizontalBounds,
        destination: HorizontalBounds,
        bottom_top: IndexRange,
    ) -> Self {
        Self {
            source_patch_id,
            destination_patch_id,
            axis,
            direction,
            source,
            destination,
            bottom_top,
        }
    }

    /// Returns the patch whose values are packed.
    pub const fn source_patch_id(self) -> PatchId {
        self.source_patch_id
    }

    /// Returns the patch whose memory is updated.
    pub const fn destination_patch_id(self) -> PatchId {
        self.destination_patch_id
    }

    /// Returns the exchange phase.
    pub const fn axis(self) -> ExchangeAxis {
        self.axis
    }

    /// Returns the receiving side of the destination patch.
    pub const fn direction(self) -> ExchangeDirection {
        self.direction
    }

    /// Returns source horizontal bounds.
    pub const fn source(self) -> HorizontalBounds {
        self.source
    }

    /// Returns destination horizontal bounds.
    pub const fn destination(self) -> HorizontalBounds {
        self.destination
    }

    /// Returns the unchanged vertical range.
    pub const fn bottom_top(self) -> IndexRange {
        self.bottom_top
    }

    /// Returns the number of scalar values in the message.
    pub fn value_count(self) -> Option<usize> {
        self.source
            .west_east()
            .len()
            .checked_mul(self.bottom_top.len())?
            .checked_mul(self.source.south_north().len())
    }
}
