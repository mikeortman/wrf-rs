use std::ops::Range;

use wrf_compute::GridShape;

use super::{DryBoundaryTendencyError, DryBoundaryTendencyResult, DryBoundaryTendencyTarget};
use crate::{SpecifiedBoundaryFieldLocation, SpecifiedBoundaryUpdateRegion};

/// Validated common domain and location-specific tiles for `spec_bdy_dry`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DryBoundaryTendencyRegion {
    west_east_momentum: SpecifiedBoundaryUpdateRegion,
    south_north_momentum: SpecifiedBoundaryUpdateRegion,
    perturbation_geopotential: SpecifiedBoundaryUpdateRegion,
    potential_temperature: SpecifiedBoundaryUpdateRegion,
    perturbation_column_mass: SpecifiedBoundaryUpdateRegion,
    vertical_momentum: SpecifiedBoundaryUpdateRegion,
}

impl DryBoundaryTendencyRegion {
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
    ) -> DryBoundaryTendencyResult<Self> {
        let west_east_momentum = Self::create_region(
            DryBoundaryTendencyTarget::WestEastMomentum,
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
            DryBoundaryTendencyTarget::SouthNorthMomentum,
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
            DryBoundaryTendencyTarget::PerturbationGeopotential,
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
            DryBoundaryTendencyTarget::PotentialTemperature,
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
            DryBoundaryTendencyTarget::VerticalMomentum,
            volume_shape,
            SpecifiedBoundaryFieldLocation::FullLevel,
            &mass_domain_west_east,
            &mass_domain_south_north,
            &half_level_domain,
            &tile_west_east,
            &tile_south_north,
            &tile_bottom_top,
        )?;
        let perturbation_column_mass = Self::create_region(
            DryBoundaryTendencyTarget::PerturbationColumnMass,
            volume_shape.horizontal_shape(),
            SpecifiedBoundaryFieldLocation::HorizontalMass,
            &mass_domain_west_east,
            &mass_domain_south_north,
            &(0..1),
            &tile_west_east,
            &tile_south_north,
            &(0..1),
        )?;

        Ok(Self {
            west_east_momentum,
            south_north_momentum,
            perturbation_geopotential,
            potential_temperature,
            perturbation_column_mass,
            vertical_momentum,
        })
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
        target: DryBoundaryTendencyTarget,
        shape: GridShape,
        location: SpecifiedBoundaryFieldLocation,
        mass_domain_west_east: &Range<usize>,
        mass_domain_south_north: &Range<usize>,
        half_level_domain: &Range<usize>,
        tile_west_east: &Range<usize>,
        tile_south_north: &Range<usize>,
        tile_bottom_top: &Range<usize>,
    ) -> DryBoundaryTendencyResult<SpecifiedBoundaryUpdateRegion> {
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
        .map_err(|source| DryBoundaryTendencyError::InvalidRegion { target, source })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_every_wrf_stagger_from_one_dry_tile() {
        let region = DryBoundaryTendencyRegion::try_new(
            GridShape::try_new(10, 10, 6).unwrap(),
            1..9,
            1..9,
            1..5,
            1..5,
            4..9,
            1..4,
        )
        .unwrap();

        assert_eq!(
            region.west_east_momentum().location(),
            SpecifiedBoundaryFieldLocation::WestEastFace
        );
        assert_eq!(
            region.south_north_momentum().location(),
            SpecifiedBoundaryFieldLocation::SouthNorthFace
        );
        assert_eq!(
            region.perturbation_geopotential().location(),
            SpecifiedBoundaryFieldLocation::FullLevel
        );
        assert_eq!(
            region.potential_temperature().location(),
            SpecifiedBoundaryFieldLocation::MassHalfLevel
        );
        assert_eq!(
            region.perturbation_column_mass().location(),
            SpecifiedBoundaryFieldLocation::HorizontalMass
        );
    }
}
