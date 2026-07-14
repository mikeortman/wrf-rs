use std::ops::Range;

use wrf_compute::GridShape;

use crate::{DryBoundaryRelaxationRegion, DryBoundaryTendencyRegion, DryTendencyAssemblyRegion};

use super::{DryLargeStepBoundaryStageError, DryLargeStepBoundaryStageResult};

/// Regions for the three consecutive dry large-step boundary kernels.
///
/// All three typed regions derive from one physical domain and one active
/// tile, so the composed stage cannot run its kernels on disagreeing ranges.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DryLargeStepBoundaryStageRegions {
    pub(crate) relaxation: DryBoundaryRelaxationRegion,
    pub(crate) assembly: DryTendencyAssemblyRegion,
    pub(crate) boundary_assignment: DryBoundaryTendencyRegion,
}

impl DryLargeStepBoundaryStageRegions {
    /// Validates zero-based half-open storage, physical-domain, and tile ranges.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        mass_domain_west_east: Range<usize>,
        mass_domain_south_north: Range<usize>,
        half_level_domain: Range<usize>,
        tile_west_east: Range<usize>,
        tile_south_north: Range<usize>,
        tile_bottom_top: Range<usize>,
    ) -> DryLargeStepBoundaryStageResult<Self> {
        let relaxation = DryBoundaryRelaxationRegion::try_new(
            shape,
            mass_domain_west_east.clone(),
            mass_domain_south_north.clone(),
            half_level_domain.clone(),
            tile_west_east.clone(),
            tile_south_north.clone(),
            tile_bottom_top.clone(),
        )
        .map_err(DryLargeStepBoundaryStageError::Relaxation)?;
        let assembly = DryTendencyAssemblyRegion::try_new(
            shape,
            mass_domain_west_east.clone(),
            mass_domain_south_north.clone(),
            half_level_domain.clone(),
            tile_west_east.clone(),
            tile_south_north.clone(),
            tile_bottom_top.clone(),
        )
        .map_err(DryLargeStepBoundaryStageError::Assembly)?;
        let boundary_assignment = DryBoundaryTendencyRegion::try_new(
            shape,
            mass_domain_west_east,
            mass_domain_south_north,
            half_level_domain,
            tile_west_east,
            tile_south_north,
            tile_bottom_top,
        )
        .map_err(DryLargeStepBoundaryStageError::BoundaryAssignment)?;
        Ok(Self {
            relaxation,
            assembly,
            boundary_assignment,
        })
    }

    /// Returns the caller-owned first-substep mass-weighting workspace shape.
    pub const fn relaxation_workspace_shape(&self) -> GridShape {
        self.relaxation.workspace_shape()
    }
}
