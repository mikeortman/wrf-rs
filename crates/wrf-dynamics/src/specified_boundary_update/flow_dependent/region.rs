use std::ops::Range;

use wrf_compute::GridShape;

use crate::{
    SpecifiedBoundaryFieldLocation, SpecifiedBoundaryFlowResult, SpecifiedBoundaryUpdateRegion,
};

/// Validated unstaggered scalar domain and tile for `flow_dep_bdy`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpecifiedBoundaryFlowRegion {
    inner: SpecifiedBoundaryUpdateRegion,
}

impl SpecifiedBoundaryFlowRegion {
    /// Validates storage, physical domains, and the active scalar tile.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        mass_domain_west_east: Range<usize>,
        mass_domain_south_north: Range<usize>,
        half_level_domain: Range<usize>,
        tile_west_east: Range<usize>,
        tile_south_north: Range<usize>,
        tile_bottom_top: Range<usize>,
    ) -> SpecifiedBoundaryFlowResult<Self> {
        Ok(Self {
            inner: SpecifiedBoundaryUpdateRegion::try_new(
                shape,
                SpecifiedBoundaryFieldLocation::MassHalfLevel,
                mass_domain_west_east,
                mass_domain_south_north,
                half_level_domain,
                tile_west_east,
                tile_south_north,
                tile_bottom_top,
            )?,
        })
    }

    /// Returns the common field shape checked before mutation.
    pub const fn shape(&self) -> GridShape {
        self.inner.shape()
    }

    pub(crate) const fn inner(&self) -> &SpecifiedBoundaryUpdateRegion {
        &self.inner
    }
}
