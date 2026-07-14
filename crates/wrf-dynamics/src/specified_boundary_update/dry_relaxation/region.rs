use std::ops::Range;

use wrf_compute::GridShape;

use super::{DryBoundaryRelaxationError, DryBoundaryRelaxationResult, DryBoundaryRelaxationTarget};
use crate::{SpecifiedBoundaryFieldLocation, SpecifiedBoundaryUpdateRegion};

/// Validated common domain, tile, and tile-halo workspace for `relax_bdy_dry`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DryBoundaryRelaxationRegion {
    volume_shape: GridShape,
    horizontal_shape: GridShape,
    workspace_shape: GridShape,
    workspace_west_east: Range<usize>,
    workspace_south_north: Range<usize>,
    workspace_bottom_top: Range<usize>,
    west_east_momentum: SpecifiedBoundaryUpdateRegion,
    south_north_momentum: SpecifiedBoundaryUpdateRegion,
    perturbation_geopotential: SpecifiedBoundaryUpdateRegion,
    potential_temperature: SpecifiedBoundaryUpdateRegion,
    perturbation_column_mass: SpecifiedBoundaryUpdateRegion,
    vertical_momentum: SpecifiedBoundaryUpdateRegion,
}

impl DryBoundaryRelaxationRegion {
    /// Validates zero-based half-open storage, physical-domain, and tile ranges.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        volume_shape: GridShape,
        mass_domain_west_east: Range<usize>,
        mass_domain_south_north: Range<usize>,
        half_level_domain: Range<usize>,
        tile_west_east: Range<usize>,
        tile_south_north: Range<usize>,
        tile_bottom_top: Range<usize>,
    ) -> DryBoundaryRelaxationResult<Self> {
        let west_east_momentum = Self::create_region(
            DryBoundaryRelaxationTarget::WestEastMomentum,
            volume_shape,
            SpecifiedBoundaryFieldLocation::WestEastFace,
            &mass_domain_west_east,
            &mass_domain_south_north,
            &half_level_domain,
            &tile_west_east,
            &tile_south_north,
            &tile_bottom_top,
        )?;
        let south_north_momentum = Self::create_region(
            DryBoundaryRelaxationTarget::SouthNorthMomentum,
            volume_shape,
            SpecifiedBoundaryFieldLocation::SouthNorthFace,
            &mass_domain_west_east,
            &mass_domain_south_north,
            &half_level_domain,
            &tile_west_east,
            &tile_south_north,
            &tile_bottom_top,
        )?;
        let perturbation_geopotential = Self::create_region(
            DryBoundaryRelaxationTarget::PerturbationGeopotential,
            volume_shape,
            SpecifiedBoundaryFieldLocation::FullLevel,
            &mass_domain_west_east,
            &mass_domain_south_north,
            &half_level_domain,
            &tile_west_east,
            &tile_south_north,
            &tile_bottom_top,
        )?;
        let potential_temperature = Self::create_region(
            DryBoundaryRelaxationTarget::PotentialTemperature,
            volume_shape,
            SpecifiedBoundaryFieldLocation::MassHalfLevel,
            &mass_domain_west_east,
            &mass_domain_south_north,
            &half_level_domain,
            &tile_west_east,
            &tile_south_north,
            &tile_bottom_top,
        )?;
        let vertical_momentum = Self::create_region(
            DryBoundaryRelaxationTarget::VerticalMomentum,
            volume_shape,
            SpecifiedBoundaryFieldLocation::FullLevel,
            &mass_domain_west_east,
            &mass_domain_south_north,
            &half_level_domain,
            &tile_west_east,
            &tile_south_north,
            &tile_bottom_top,
        )?;
        let horizontal_shape = volume_shape.horizontal_shape();
        let perturbation_column_mass = Self::create_region(
            DryBoundaryRelaxationTarget::PerturbationColumnMass,
            horizontal_shape,
            SpecifiedBoundaryFieldLocation::HorizontalMass,
            &mass_domain_west_east,
            &mass_domain_south_north,
            &(0..1),
            &tile_west_east,
            &tile_south_north,
            &(0..1),
        )?;

        let workspace_west_east = halo_range(&tile_west_east, &mass_domain_west_east);
        let workspace_south_north = halo_range(&tile_south_north, &mass_domain_south_north);
        let workspace_bottom_top = tile_bottom_top;
        let workspace_shape = GridShape::try_new(
            workspace_west_east.len(),
            workspace_south_north.len(),
            workspace_bottom_top.len(),
        )
        .map_err(|_| DryBoundaryRelaxationError::WorkspaceShapeOverflow)?;

        Ok(Self {
            volume_shape,
            horizontal_shape,
            workspace_shape,
            workspace_west_east,
            workspace_south_north,
            workspace_bottom_top,
            west_east_momentum,
            south_north_momentum,
            perturbation_geopotential,
            potential_temperature,
            perturbation_column_mass,
            vertical_momentum,
        })
    }

    /// Returns the required caller-owned mass-weighting workspace shape.
    pub const fn workspace_shape(&self) -> GridShape {
        self.workspace_shape
    }

    pub(crate) const fn volume_shape(&self) -> GridShape {
        self.volume_shape
    }

    pub(crate) const fn horizontal_shape(&self) -> GridShape {
        self.horizontal_shape
    }

    pub(crate) fn workspace_ranges(&self) -> (&Range<usize>, &Range<usize>, &Range<usize>) {
        (
            &self.workspace_west_east,
            &self.workspace_south_north,
            &self.workspace_bottom_top,
        )
    }

    pub(crate) const fn west_east_momentum(&self) -> &SpecifiedBoundaryUpdateRegion {
        &self.west_east_momentum
    }

    pub(crate) const fn south_north_momentum(&self) -> &SpecifiedBoundaryUpdateRegion {
        &self.south_north_momentum
    }

    pub(crate) const fn perturbation_geopotential(&self) -> &SpecifiedBoundaryUpdateRegion {
        &self.perturbation_geopotential
    }

    pub(crate) const fn potential_temperature(&self) -> &SpecifiedBoundaryUpdateRegion {
        &self.potential_temperature
    }

    pub(crate) const fn perturbation_column_mass(&self) -> &SpecifiedBoundaryUpdateRegion {
        &self.perturbation_column_mass
    }

    pub(crate) const fn vertical_momentum(&self) -> &SpecifiedBoundaryUpdateRegion {
        &self.vertical_momentum
    }

    #[allow(clippy::too_many_arguments)]
    fn create_region(
        target: DryBoundaryRelaxationTarget,
        shape: GridShape,
        location: SpecifiedBoundaryFieldLocation,
        mass_domain_west_east: &Range<usize>,
        mass_domain_south_north: &Range<usize>,
        half_level_domain: &Range<usize>,
        tile_west_east: &Range<usize>,
        tile_south_north: &Range<usize>,
        tile_bottom_top: &Range<usize>,
    ) -> DryBoundaryRelaxationResult<SpecifiedBoundaryUpdateRegion> {
        SpecifiedBoundaryUpdateRegion::try_new(
            shape,
            location,
            mass_domain_west_east.clone(),
            mass_domain_south_north.clone(),
            half_level_domain.clone(),
            tile_west_east.clone(),
            tile_south_north.clone(),
            tile_bottom_top.clone(),
        )
        .map_err(|source| DryBoundaryRelaxationError::InvalidRegion { target, source })
    }
}

fn halo_range(tile: &Range<usize>, domain: &Range<usize>) -> Range<usize> {
    tile.start.saturating_sub(1).max(domain.start)..tile.end.saturating_add(1).min(domain.end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_clipped_tile_halo_workspace_and_all_staggers() {
        let region = DryBoundaryRelaxationRegion::try_new(
            GridShape::try_new(10, 10, 6).unwrap(),
            1..9,
            1..9,
            1..5,
            1..5,
            4..9,
            1..6,
        )
        .unwrap();

        assert_eq!(region.workspace_ranges(), (&(1..6), &(3..9), &(1..6)));
        assert_eq!(
            region.workspace_shape(),
            GridShape::try_new(5, 6, 5).unwrap()
        );
        assert_eq!(
            region.west_east_momentum().location(),
            SpecifiedBoundaryFieldLocation::WestEastFace
        );
        assert_eq!(
            region.perturbation_column_mass().location(),
            SpecifiedBoundaryFieldLocation::HorizontalMass
        );
    }
}
