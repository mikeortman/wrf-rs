use std::ops::Range;

use wrf_compute::GridShape;

use super::{DryTendencyBoundaryStageError, DryTendencyBoundaryStageResult};
use crate::{DryBoundaryTendencyRegion, DryTendencyAssemblyRegion};

/// Regions for the consecutive assembly and boundary-assignment kernels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DryTendencyBoundaryStageRegions {
    pub(crate) assembly: DryTendencyAssemblyRegion,
    pub(crate) boundary_assignment: DryBoundaryTendencyRegion,
}

impl DryTendencyBoundaryStageRegions {
    /// Derives both typed regions from one physical domain and active tile.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        mass_domain_west_east: Range<usize>,
        mass_domain_south_north: Range<usize>,
        half_level_domain: Range<usize>,
        tile_west_east: Range<usize>,
        tile_south_north: Range<usize>,
        tile_bottom_top: Range<usize>,
    ) -> DryTendencyBoundaryStageResult<Self> {
        let assembly = DryTendencyAssemblyRegion::try_new(
            shape,
            mass_domain_west_east.clone(),
            mass_domain_south_north.clone(),
            half_level_domain.clone(),
            tile_west_east.clone(),
            tile_south_north.clone(),
            tile_bottom_top.clone(),
        )
        .map_err(DryTendencyBoundaryStageError::Assembly)?;
        let boundary_assignment = DryBoundaryTendencyRegion::try_new(
            shape,
            mass_domain_west_east,
            mass_domain_south_north,
            half_level_domain,
            tile_west_east,
            tile_south_north,
            tile_bottom_top,
        )
        .map_err(DryTendencyBoundaryStageError::BoundaryAssignment)?;
        Ok(Self {
            assembly,
            boundary_assignment,
        })
    }
}
