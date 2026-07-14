use crate::{HorizontalBounds, PatchId};

/// One thread-level tile clipped to the physical domain.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TileBounds {
    patch_id: PatchId,
    tile_index: usize,
    execution: HorizontalBounds,
}

impl TileBounds {
    pub(crate) const fn new(
        patch_id: PatchId,
        tile_index: usize,
        execution: HorizontalBounds,
    ) -> Self {
        Self {
            patch_id,
            tile_index,
            execution,
        }
    }

    /// Returns the patch that owns the tile.
    pub const fn patch_id(self) -> PatchId {
        self.patch_id
    }

    /// Returns the tile's row-major index within its patch.
    pub const fn tile_index(self) -> usize {
        self.tile_index
    }

    /// Returns the tile bounds after physical-domain clipping.
    pub const fn execution(self) -> HorizontalBounds {
        self.execution
    }
}
