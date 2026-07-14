use crate::{HorizontalBounds, MemoryBounds, PatchCoordinate, PatchId};

/// Owned and allocated bounds for one process-grid patch.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PatchBounds {
    patch_id: PatchId,
    coordinate: PatchCoordinate,
    owned: HorizontalBounds,
    memory: MemoryBounds,
}

impl PatchBounds {
    pub(crate) const fn new(
        patch_id: PatchId,
        coordinate: PatchCoordinate,
        owned: HorizontalBounds,
        memory: MemoryBounds,
    ) -> Self {
        Self {
            patch_id,
            coordinate,
            owned,
            memory,
        }
    }

    /// Returns this patch's stable row-major identifier.
    pub const fn patch_id(self) -> PatchId {
        self.patch_id
    }

    /// Returns this patch's process-grid coordinate.
    pub const fn coordinate(self) -> PatchCoordinate {
        self.coordinate
    }

    /// Returns the physical points owned by the patch.
    pub const fn owned(self) -> HorizontalBounds {
        self.owned
    }

    /// Returns the patch's allocated bounds.
    pub const fn memory(self) -> MemoryBounds {
        self.memory
    }
}
