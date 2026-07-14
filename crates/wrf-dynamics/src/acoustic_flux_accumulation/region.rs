use std::ops::Range;

use wrf_compute::GridShape;

use crate::{AcousticFluxAccumulationError, AcousticFluxAccumulationResult};

/// Validated physical domains and C-grid tile for WRF `sumflux` execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcousticFluxAccumulationRegion {
    shape: GridShape,
    mass_domain_west_east: Range<usize>,
    mass_domain_south_north: Range<usize>,
    half_level_domain: Range<usize>,
    tile_west_east: Range<usize>,
    tile_south_north: Range<usize>,
    tile_bottom_top: Range<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AcousticFluxActiveRanges {
    pub(crate) mass_west_east: Range<usize>,
    pub(crate) staggered_west_east: Range<usize>,
    pub(crate) mass_south_north: Range<usize>,
    pub(crate) staggered_south_north: Range<usize>,
    pub(crate) half_levels: Range<usize>,
    pub(crate) full_levels: Range<usize>,
}

impl AcousticFluxAccumulationRegion {
    /// Validates physical mass/half-level domains and a tile that may include
    /// each domain's single upper stagger point.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        shape: GridShape,
        mass_domain_west_east: Range<usize>,
        mass_domain_south_north: Range<usize>,
        half_level_domain: Range<usize>,
        tile_west_east: Range<usize>,
        tile_south_north: Range<usize>,
        tile_bottom_top: Range<usize>,
    ) -> AcousticFluxAccumulationResult<Self> {
        validate_domain(&mass_domain_west_east, shape.west_east_points())?;
        validate_domain(&mass_domain_south_north, shape.south_north_points())?;
        validate_domain(&half_level_domain, shape.bottom_top_points())?;
        validate_tile(
            &tile_west_east,
            &mass_domain_west_east,
            shape.west_east_points(),
        )?;
        validate_tile(
            &tile_south_north,
            &mass_domain_south_north,
            shape.south_north_points(),
        )?;
        validate_tile(
            &tile_bottom_top,
            &half_level_domain,
            shape.bottom_top_points(),
        )?;
        Ok(Self {
            shape,
            mass_domain_west_east,
            mass_domain_south_north,
            half_level_domain,
            tile_west_east,
            tile_south_north,
            tile_bottom_top,
        })
    }

    /// Returns the common three-dimensional storage shape.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    pub(crate) fn active_ranges(&self) -> AcousticFluxActiveRanges {
        AcousticFluxActiveRanges {
            mass_west_east: self.tile_west_east.start
                ..self.tile_west_east.end.min(self.mass_domain_west_east.end),
            staggered_west_east: self.tile_west_east.clone(),
            mass_south_north: self.tile_south_north.start
                ..self
                    .tile_south_north
                    .end
                    .min(self.mass_domain_south_north.end),
            staggered_south_north: self.tile_south_north.clone(),
            half_levels: self.tile_bottom_top.start
                ..self.tile_bottom_top.end.min(self.half_level_domain.end),
            full_levels: self.tile_bottom_top.clone(),
        }
    }
}

fn validate_domain(domain: &Range<usize>, extent: usize) -> AcousticFluxAccumulationResult<()> {
    if domain.is_empty() {
        return Err(AcousticFluxAccumulationError::EmptyDomainRange);
    }
    if domain.end >= extent {
        return Err(AcousticFluxAccumulationError::MissingUpperStaggerPoint {
            boundary_index: domain.end,
            field_extent: extent,
        });
    }
    Ok(())
}

fn validate_tile(
    tile: &Range<usize>,
    domain: &Range<usize>,
    extent: usize,
) -> AcousticFluxAccumulationResult<()> {
    if tile.is_empty() {
        return Err(AcousticFluxAccumulationError::EmptyTileRange);
    }
    if tile.end > extent {
        return Err(AcousticFluxAccumulationError::TileRangeOutOfBounds {
            range_end: tile.end,
            field_extent: extent,
        });
    }
    if tile.start < domain.start || tile.end > domain.end + 1 {
        return Err(AcousticFluxAccumulationError::TileOutsideDomain);
    }
    Ok(())
}
