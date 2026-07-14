use std::ops::Range;

use wrf_compute::GridShape;

use crate::column_mass_staggering::axis_boundary::ColumnMassStaggeringAxisBoundary;
use crate::{ColumnMassStaggeringAxis, ColumnMassStaggeringError, ColumnMassStaggeringResult};

/// Validated physical-domain and active-tile ranges for column-mass staggering.
///
/// Memory storage comes from [`Self::shape`]. The mass-domain ranges represent
/// WRF's inclusive lower and exclusive upper mass-point bounds (`ids..ide` and
/// `jds..jde`), while the tile ranges include momentum points through WRF's
/// inclusive `ite` and `jte`. Keeping these concepts separate lets the region
/// derive physical-boundary contact without confusing it with memory halos.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColumnMassStaggeringRegion {
    shape: GridShape,
    mass_domain_west_east_range: Range<usize>,
    mass_domain_south_north_range: Range<usize>,
    tile_west_east_momentum_range: Range<usize>,
    tile_south_north_momentum_range: Range<usize>,
    west_east_boundary: ColumnMassStaggeringAxisBoundary,
    south_north_boundary: ColumnMassStaggeringAxisBoundary,
}

impl ColumnMassStaggeringRegion {
    /// Validates mass-domain and active momentum-tile ranges.
    ///
    /// All ranges use zero-based, half-open memory offsets. A mass-domain range
    /// excludes its upper physical-boundary momentum point, so that endpoint
    /// must itself fit in the field shape. A tile range may include that point.
    /// The constructor derives lower and upper physical-boundary contact by
    /// comparing the tile endpoints with the domain endpoints.
    pub fn try_new(
        shape: GridShape,
        mass_domain_west_east_range: Range<usize>,
        mass_domain_south_north_range: Range<usize>,
        tile_west_east_momentum_range: Range<usize>,
        tile_south_north_momentum_range: Range<usize>,
    ) -> ColumnMassStaggeringResult<Self> {
        if shape.bottom_top_points() != 1 {
            return Err(ColumnMassStaggeringError::RequiresSingleVerticalLevel {
                bottom_top_points: shape.bottom_top_points(),
            });
        }

        validate_mass_domain_range(
            ColumnMassStaggeringAxis::WestEast,
            &mass_domain_west_east_range,
            shape.west_east_points(),
        )?;
        validate_mass_domain_range(
            ColumnMassStaggeringAxis::SouthNorth,
            &mass_domain_south_north_range,
            shape.south_north_points(),
        )?;
        validate_tile_range(
            ColumnMassStaggeringAxis::WestEast,
            &tile_west_east_momentum_range,
            &mass_domain_west_east_range,
            shape.west_east_points(),
        )?;
        validate_tile_range(
            ColumnMassStaggeringAxis::SouthNorth,
            &tile_south_north_momentum_range,
            &mass_domain_south_north_range,
            shape.south_north_points(),
        )?;

        let west_east_boundary = ColumnMassStaggeringAxisBoundary::from_contacts(
            tile_west_east_momentum_range.start == mass_domain_west_east_range.start,
            tile_west_east_momentum_range.end == mass_domain_west_east_range.end + 1,
        );
        let south_north_boundary = ColumnMassStaggeringAxisBoundary::from_contacts(
            tile_south_north_momentum_range.start == mass_domain_south_north_range.start,
            tile_south_north_momentum_range.end == mass_domain_south_north_range.end + 1,
        );

        Ok(Self {
            shape,
            mass_domain_west_east_range,
            mass_domain_south_north_range,
            tile_west_east_momentum_range,
            tile_south_north_momentum_range,
            west_east_boundary,
            south_north_boundary,
        })
    }

    /// Returns the field shape used during validation.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    pub(crate) fn west_east_momentum_west_east_range(&self) -> Range<usize> {
        self.tile_west_east_momentum_range.clone()
    }

    pub(crate) fn west_east_momentum_south_north_range(&self) -> Range<usize> {
        self.tile_south_north_momentum_range.start
            ..self
                .tile_south_north_momentum_range
                .end
                .min(self.mass_domain_south_north_range.end)
    }

    pub(crate) fn south_north_momentum_west_east_range(&self) -> Range<usize> {
        self.tile_west_east_momentum_range.start
            ..self
                .tile_west_east_momentum_range
                .end
                .min(self.mass_domain_west_east_range.end)
    }

    pub(crate) fn south_north_momentum_south_north_range(&self) -> Range<usize> {
        self.tile_south_north_momentum_range.clone()
    }

    pub(crate) fn full_mass_ranges(
        &self,
    ) -> ColumnMassStaggeringResult<(Range<usize>, Range<usize>)> {
        let west_east_start = self
            .tile_west_east_momentum_range
            .start
            .checked_sub(1)
            .ok_or(ColumnMassStaggeringError::FullMassLowerHaloMissing {
                axis: ColumnMassStaggeringAxis::WestEast,
            })?;
        let south_north_start = self
            .tile_south_north_momentum_range
            .start
            .checked_sub(1)
            .ok_or(ColumnMassStaggeringError::FullMassLowerHaloMissing {
                axis: ColumnMassStaggeringAxis::SouthNorth,
            })?;

        Ok((
            west_east_start
                ..self
                    .tile_west_east_momentum_range
                    .end
                    .min(self.mass_domain_west_east_range.end),
            south_north_start
                ..self
                    .tile_south_north_momentum_range
                    .end
                    .min(self.mass_domain_south_north_range.end),
        ))
    }

    pub(crate) const fn west_east_boundary(&self) -> ColumnMassStaggeringAxisBoundary {
        self.west_east_boundary
    }

    pub(crate) const fn south_north_boundary(&self) -> ColumnMassStaggeringAxisBoundary {
        self.south_north_boundary
    }
}

fn validate_mass_domain_range(
    axis: ColumnMassStaggeringAxis,
    range: &Range<usize>,
    field_extent: usize,
) -> ColumnMassStaggeringResult<()> {
    if range.start >= range.end {
        return Err(ColumnMassStaggeringError::EmptyMassDomainRange { axis });
    }
    if range.end >= field_extent {
        return Err(ColumnMassStaggeringError::MassDomainBoundaryOutOfBounds {
            axis,
            boundary_index: range.end,
            field_extent,
        });
    }
    Ok(())
}

fn validate_tile_range(
    axis: ColumnMassStaggeringAxis,
    tile_range: &Range<usize>,
    mass_domain_range: &Range<usize>,
    field_extent: usize,
) -> ColumnMassStaggeringResult<()> {
    if tile_range.start >= tile_range.end {
        return Err(ColumnMassStaggeringError::EmptyTileRange { axis });
    }
    if tile_range.end > field_extent {
        return Err(ColumnMassStaggeringError::TileRangeOutOfBounds {
            axis,
            range_end: tile_range.end,
            field_extent,
        });
    }
    if tile_range.start < mass_domain_range.start || tile_range.end > mass_domain_range.end + 1 {
        return Err(ColumnMassStaggeringError::TileOutsideMassDomain { axis });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_planar_shapes_and_invalid_domain_ranges() {
        let three_dimensional_shape = GridShape::try_new(4, 4, 2).unwrap();
        assert_eq!(
            ColumnMassStaggeringRegion::try_new(three_dimensional_shape, 0..3, 0..3, 1..3, 1..3,),
            Err(ColumnMassStaggeringError::RequiresSingleVerticalLevel {
                bottom_top_points: 2,
            })
        );

        let shape = GridShape::try_new(4, 4, 1).unwrap();
        assert_eq!(
            ColumnMassStaggeringRegion::try_new(shape, 1..1, 0..3, 1..3, 1..3),
            Err(ColumnMassStaggeringError::EmptyMassDomainRange {
                axis: ColumnMassStaggeringAxis::WestEast,
            })
        );
        assert_eq!(
            ColumnMassStaggeringRegion::try_new(shape, 0..4, 0..3, 1..3, 1..3),
            Err(ColumnMassStaggeringError::MassDomainBoundaryOutOfBounds {
                axis: ColumnMassStaggeringAxis::WestEast,
                boundary_index: 4,
                field_extent: 4,
            })
        );
    }

    #[test]
    fn rejects_invalid_tile_ranges() {
        let shape = GridShape::try_new(5, 5, 1).unwrap();
        assert_eq!(
            ColumnMassStaggeringRegion::try_new(shape, 1..4, 1..4, 2..2, 2..4),
            Err(ColumnMassStaggeringError::EmptyTileRange {
                axis: ColumnMassStaggeringAxis::WestEast,
            })
        );
        assert_eq!(
            ColumnMassStaggeringRegion::try_new(shape, 0..4, 0..4, 1..6, 1..4),
            Err(ColumnMassStaggeringError::TileRangeOutOfBounds {
                axis: ColumnMassStaggeringAxis::WestEast,
                range_end: 6,
                field_extent: 5,
            })
        );
        assert_eq!(
            ColumnMassStaggeringRegion::try_new(shape, 1..4, 1..4, 0..4, 1..4),
            Err(ColumnMassStaggeringError::TileOutsideMassDomain {
                axis: ColumnMassStaggeringAxis::WestEast,
            })
        );
    }

    #[test]
    fn derives_staggered_output_ranges_and_boundary_contacts() {
        let shape = GridShape::try_new(6, 5, 1).unwrap();
        let region = ColumnMassStaggeringRegion::try_new(shape, 1..4, 1..3, 1..5, 1..4).unwrap();

        assert_eq!(region.west_east_momentum_west_east_range(), 1..5);
        assert_eq!(region.west_east_momentum_south_north_range(), 1..3);
        assert_eq!(region.south_north_momentum_west_east_range(), 1..4);
        assert_eq!(region.south_north_momentum_south_north_range(), 1..4);
        assert_eq!(
            region.west_east_boundary(),
            ColumnMassStaggeringAxisBoundary::Both
        );
        assert_eq!(
            region.south_north_boundary(),
            ColumnMassStaggeringAxisBoundary::Both
        );
        assert_eq!(region.full_mass_ranges(), Ok((0..4, 0..3)));
    }

    #[test]
    fn full_mass_requires_a_lower_memory_halo_on_both_axes() {
        let shape = GridShape::try_new(5, 5, 1).unwrap();
        let west_east_missing =
            ColumnMassStaggeringRegion::try_new(shape, 0..4, 1..4, 0..4, 1..4).unwrap();
        assert_eq!(
            west_east_missing.full_mass_ranges(),
            Err(ColumnMassStaggeringError::FullMassLowerHaloMissing {
                axis: ColumnMassStaggeringAxis::WestEast,
            })
        );

        let south_north_missing =
            ColumnMassStaggeringRegion::try_new(shape, 1..4, 0..4, 1..4, 0..4).unwrap();
        assert_eq!(
            south_north_missing.full_mass_ranges(),
            Err(ColumnMassStaggeringError::FullMassLowerHaloMissing {
                axis: ColumnMassStaggeringAxis::SouthNorth,
            })
        );
    }
}
