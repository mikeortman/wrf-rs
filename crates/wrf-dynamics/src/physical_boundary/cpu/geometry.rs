use crate::{
    PHYSICAL_BOUNDARY_ZONE, PhysicalBoundaryConditions, PhysicalBoundaryRegion,
    PhysicalBoundaryVariable,
};

/// Fortran-style signed indices resolved once per kernel call.
///
/// All values are zero-based storage indices: `ids..=ide` and `jds..=jde` are
/// the staggered physical domain bounds, `its..=ite`/`jts..=jte` the inclusive
/// tile bounds, and `kts..=k_end` the inclusive vertical loop for the current
/// variable class. Signed arithmetic mirrors the pinned Fortran index
/// expressions; region validation proves every touched index is stored.
#[derive(Clone, Copy)]
pub(super) struct PhysicalBoundaryGeometry {
    pub(super) west_east_points: isize,
    pub(super) bottom_top_points: isize,
    pub(super) ids: isize,
    pub(super) ide: isize,
    pub(super) jds: isize,
    pub(super) jde: isize,
    pub(super) its: isize,
    pub(super) ite: isize,
    pub(super) jts: isize,
    pub(super) jte: isize,
    pub(super) kts: isize,
    pub(super) k_end: isize,
    pub(super) west_east_stagger: isize,
    pub(super) south_north_stagger: isize,
    pub(super) zone: isize,
    pub(super) conditions: PhysicalBoundaryConditions,
    pub(super) variable: PhysicalBoundaryVariable,
}

impl PhysicalBoundaryGeometry {
    /// Resolves the volume-field geometry of `set_physical_bc3d`.
    pub(super) fn for_volume(
        variable: PhysicalBoundaryVariable,
        conditions: PhysicalBoundaryConditions,
        region: &PhysicalBoundaryRegion,
    ) -> Self {
        let half_level = region.half_level();
        let tile_bottom_top = region.tile_bottom_top();
        let bottom_tile_start = tile_bottom_top.start as isize;
        let bottom_tile_end = tile_bottom_top.end as isize - 1;
        let half_level_end = half_level.end as isize;
        // WRF: k_end = MAX(1, MIN(kde-1, kte)), overridden to MIN(kde, kte)
        // for full-level fields; storage index 1 mirrors Fortran's literal 1
        // under the crate's kms -> 0 memory mapping.
        let k_end = if variable.is_full_level() {
            half_level_end.min(bottom_tile_end)
        } else {
            (half_level_end - 1).min(bottom_tile_end).max(1)
        };
        Self::with_levels(variable, conditions, region, bottom_tile_start, k_end)
    }

    /// Resolves the single-level geometry of `set_physical_bc2d`.
    pub(super) fn for_horizontal(
        variable: PhysicalBoundaryVariable,
        conditions: PhysicalBoundaryConditions,
        region: &PhysicalBoundaryRegion,
    ) -> Self {
        let mut geometry = Self::with_levels(variable, conditions, region, 0, 0);
        geometry.bottom_top_points = 1;
        geometry
    }

    fn with_levels(
        variable: PhysicalBoundaryVariable,
        conditions: PhysicalBoundaryConditions,
        region: &PhysicalBoundaryRegion,
        bottom_tile_start: isize,
        k_end: isize,
    ) -> Self {
        let shape = region.shape();
        Self {
            west_east_points: shape.west_east_points() as isize,
            bottom_top_points: shape.bottom_top_points() as isize,
            ids: region.mass_west_east().start as isize,
            ide: region.mass_west_east().end as isize,
            jds: region.mass_south_north().start as isize,
            jde: region.mass_south_north().end as isize,
            its: region.tile_west_east().start as isize,
            ite: region.tile_west_east().end as isize - 1,
            jts: region.tile_south_north().start as isize,
            jte: region.tile_south_north().end as isize - 1,
            kts: bottom_tile_start,
            k_end,
            west_east_stagger: variable.west_east_stagger(),
            south_north_stagger: variable.south_north_stagger(),
            zone: PHYSICAL_BOUNDARY_ZONE as isize,
            conditions,
            variable,
        }
    }

    /// True when this tile owns the west physical edge (`its == ids`).
    pub(super) const fn touches_west(&self) -> bool {
        self.its == self.ids
    }

    /// True when this tile owns the east physical edge (`ite == ide`).
    pub(super) const fn touches_east(&self) -> bool {
        self.ite == self.ide
    }

    /// True when this tile owns the south physical edge (`jts == jds`).
    pub(super) const fn touches_south(&self) -> bool {
        self.jts == self.jds
    }

    /// True when this tile owns the north physical edge (`jte == jde`).
    pub(super) const fn touches_north(&self) -> bool {
        self.jte == self.jde
    }

    /// WRF's shared row window `MAX(jds, jts-1) ..= MIN(jte+1, jde+jstag)`.
    pub(super) fn lateral_row_window(&self) -> (isize, isize) {
        (
            self.jds.max(self.jts - 1),
            (self.jte + 1).min(self.jde + self.south_north_stagger),
        )
    }

    /// The `i` window used by every south-north branch, with edge overrides.
    pub(super) fn south_north_column_window(&self) -> (isize, isize) {
        let mut start = self.ids.max(self.its - 1);
        let mut end = (self.ite + 1).min(self.ide + self.west_east_stagger);
        if self.touches_west() {
            start = 0;
        }
        if self.touches_east() {
            end = self.west_east_points - 1;
        }
        (start, end)
    }
}
