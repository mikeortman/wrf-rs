use crate::{
    BoundaryWidths, DomainBounds, HorizontalBounds, IndexRange, MemoryBounds, PatchBounds,
    PatchCoordinate, PatchId, ProcessGrid, TileBounds, TileGrid, TopologyError, TopologyResult,
};

/// Validated WRF process-grid decomposition and derived patch memory bounds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainTopology {
    domain: DomainBounds,
    process_grid: ProcessGrid,
    maximum_halo_width: usize,
    boundary_widths: BoundaryWidths,
    patches: Vec<PatchBounds>,
}

impl DomainTopology {
    /// Constructs the centered-remainder decomposition used by RSL_LITE.
    pub fn try_new(
        domain: DomainBounds,
        process_grid: ProcessGrid,
        maximum_halo_width: usize,
        boundary_widths: BoundaryWidths,
    ) -> TopologyResult<Self> {
        validate_process_grid(domain, process_grid)?;
        let halo_width = to_i32_halo_width(maximum_halo_width)?;
        let west_east_boundary = to_i32_boundary_width(boundary_widths.west_east())?;
        let south_north_boundary = to_i32_boundary_width(boundary_widths.south_north())?;
        let patches = create_patches(
            domain,
            process_grid,
            halo_width,
            west_east_boundary,
            south_north_boundary,
        )?;

        Ok(Self {
            domain,
            process_grid,
            maximum_halo_width,
            boundary_widths,
            patches,
        })
    }

    /// Returns the physical domain bounds.
    pub const fn domain(&self) -> DomainBounds {
        self.domain
    }

    /// Returns the process-grid shape.
    pub const fn process_grid(&self) -> ProcessGrid {
        self.process_grid
    }

    /// Returns the maximum allocated halo width.
    pub const fn maximum_halo_width(&self) -> usize {
        self.maximum_halo_width
    }

    /// Returns physical-boundary storage widths.
    pub const fn boundary_widths(&self) -> BoundaryWidths {
        self.boundary_widths
    }

    /// Returns patches in WRF/MPI row-major rank order.
    pub fn patches(&self) -> &[PatchBounds] {
        &self.patches
    }

    /// Resolves one patch by its stable identifier.
    pub fn patch(&self, patch_id: PatchId) -> TopologyResult<PatchBounds> {
        self.patches
            .get(patch_id.value())
            .copied()
            .ok_or(TopologyError::UnknownPatch { patch_id })
    }

    /// Resolves a row-major patch identifier from a process-grid coordinate.
    pub fn patch_id_at(&self, column: usize, row: usize) -> Option<PatchId> {
        if column >= self.process_grid.columns() || row >= self.process_grid.rows() {
            return None;
        }
        Some(PatchId::new(row * self.process_grid.columns() + column))
    }

    /// Splits a patch into WRF-style tiles and clips execution to the domain.
    ///
    /// `requested_execution` may extend from the owned patch into allocated
    /// halo memory. Only edge tiles inherit that extension, matching
    /// `set_tiles2`; all resulting tiles are clipped to the physical domain.
    pub fn create_tiles(
        &self,
        patch_id: PatchId,
        tile_grid: TileGrid,
        requested_execution: HorizontalBounds,
    ) -> TopologyResult<Vec<TileBounds>> {
        let patch = self.patch(patch_id)?;
        if !patch.memory().horizontal().contains(requested_execution) {
            return Err(TopologyError::TileExecutionOutsideMemory { patch_id });
        }

        let owned = patch.owned();
        let mut tiles = Vec::with_capacity(tile_grid.columns() * tile_grid.rows());
        for row in 0..tile_grid.rows() {
            let base_south_north = centered_partition(owned.south_north(), tile_grid.rows(), row);
            let south_north = extend_and_clip_edge(
                base_south_north,
                owned.south_north(),
                requested_execution.south_north(),
                self.domain.south_north(),
                row,
                tile_grid.rows(),
            );
            for column in 0..tile_grid.columns() {
                let base_west_east =
                    centered_partition(owned.west_east(), tile_grid.columns(), column);
                let west_east = extend_and_clip_edge(
                    base_west_east,
                    owned.west_east(),
                    requested_execution.west_east(),
                    self.domain.west_east(),
                    column,
                    tile_grid.columns(),
                );
                if let (Some(west_east), Some(south_north)) = (west_east, south_north) {
                    let tile_index = row * tile_grid.columns() + column;
                    tiles.push(TileBounds::new(
                        patch_id,
                        tile_index,
                        HorizontalBounds::new(west_east, south_north),
                    ));
                }
            }
        }
        Ok(tiles)
    }
}

fn validate_process_grid(domain: DomainBounds, process_grid: ProcessGrid) -> TopologyResult<()> {
    if process_grid.columns() > domain.west_east().len() {
        return Err(TopologyError::TooManyProcessColumns {
            process_columns: process_grid.columns(),
            west_east_points: domain.west_east().len(),
        });
    }
    if process_grid.rows() > domain.south_north().len() {
        return Err(TopologyError::TooManyProcessRows {
            process_rows: process_grid.rows(),
            south_north_points: domain.south_north().len(),
        });
    }
    Ok(())
}

fn to_i32_halo_width(halo_width: usize) -> TopologyResult<i32> {
    i32::try_from(halo_width).map_err(|_| TopologyError::HaloWidthTooLarge { halo_width })
}

fn to_i32_boundary_width(boundary_width: usize) -> TopologyResult<i32> {
    i32::try_from(boundary_width)
        .map_err(|_| TopologyError::BoundaryWidthTooLarge { boundary_width })
}

fn create_patches(
    domain: DomainBounds,
    process_grid: ProcessGrid,
    halo_width: i32,
    west_east_boundary: i32,
    south_north_boundary: i32,
) -> TopologyResult<Vec<PatchBounds>> {
    let mut patches = Vec::with_capacity(process_grid.process_count());
    for row in 0..process_grid.rows() {
        let south_north = centered_partition(domain.south_north(), process_grid.rows(), row);
        for column in 0..process_grid.columns() {
            let west_east = centered_partition(domain.west_east(), process_grid.columns(), column);
            let patch_id = PatchId::new(row * process_grid.columns() + column);
            let owned = HorizontalBounds::new(west_east, south_north);
            let memory = create_memory_bounds(
                domain,
                owned,
                halo_width,
                west_east_boundary,
                south_north_boundary,
            )?;
            patches.push(PatchBounds::new(
                patch_id,
                PatchCoordinate::new(column, row),
                owned,
                memory,
            ));
        }
    }
    Ok(patches)
}

pub(crate) fn centered_partition(range: IndexRange, part_count: usize, part: usize) -> IndexRange {
    debug_assert!(part < part_count);
    debug_assert!(part_count <= range.len());
    let base_length = range.len() / part_count;
    let extra_count = range.len() % part_count;
    let lower_extra_count = extra_count / 2;
    let has_extra =
        part < lower_extra_count || part >= part_count - (extra_count - lower_extra_count);
    let preceding_extra_count = if part <= lower_extra_count {
        part
    } else {
        lower_extra_count + part.saturating_sub(part_count - (extra_count - lower_extra_count))
    };
    let start_offset = part * base_length + preceding_extra_count;
    let length = base_length + usize::from(has_extra);
    let start = (i64::from(range.start()) + start_offset as i64) as i32;
    let end = (i64::from(start) + length as i64) as i32;
    IndexRange::from_validated(start, end)
}

fn create_memory_bounds(
    domain: DomainBounds,
    owned: HorizontalBounds,
    halo_width: i32,
    west_east_boundary: i32,
    south_north_boundary: i32,
) -> TopologyResult<MemoryBounds> {
    let west_east = wrf_memory_range(
        owned.west_east(),
        domain.west_east(),
        halo_width,
        west_east_boundary,
    )?;
    let south_north = wrf_memory_range(
        owned.south_north(),
        domain.south_north(),
        halo_width,
        south_north_boundary,
    )?;
    Ok(MemoryBounds::new(
        HorizontalBounds::new(west_east, south_north),
        domain.bottom_top(),
    ))
}

fn wrf_memory_range(
    owned: IndexRange,
    domain: IndexRange,
    halo_width: i32,
    boundary_width: i32,
) -> TopologyResult<IndexRange> {
    let owned_start = owned
        .start()
        .checked_sub(halo_width)
        .ok_or(TopologyError::IndexArithmeticOverflow)?;
    let boundary_start = domain
        .start()
        .checked_sub(boundary_width)
        .ok_or(TopologyError::IndexArithmeticOverflow)?;
    let start = owned_start
        .max(boundary_start)
        .checked_sub(1)
        .ok_or(TopologyError::IndexArithmeticOverflow)?;

    let owned_end = owned
        .end()
        .checked_add(halo_width)
        .ok_or(TopologyError::IndexArithmeticOverflow)?;
    let boundary_end = domain
        .end()
        .checked_add(boundary_width)
        .ok_or(TopologyError::IndexArithmeticOverflow)?;
    let end = owned_end
        .min(boundary_end)
        .checked_add(1)
        .ok_or(TopologyError::IndexArithmeticOverflow)?;
    Ok(IndexRange::from_validated(start, end))
}

fn extend_and_clip_edge(
    base: IndexRange,
    owned: IndexRange,
    requested: IndexRange,
    domain: IndexRange,
    part: usize,
    part_count: usize,
) -> Option<IndexRange> {
    let requested_start = if part == 0 && requested.start() < owned.start() {
        requested.start()
    } else {
        base.start()
    };
    let requested_end = if part + 1 == part_count && requested.end() > owned.end() {
        requested.end()
    } else {
        base.end()
    };
    let start = requested_start.max(domain.start());
    let end = requested_end.min(domain.end());
    (start < end).then(|| IndexRange::from_validated(start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn domain() -> DomainBounds {
        DomainBounds::new(
            IndexRange::try_new(0, 13).unwrap(),
            IndexRange::try_new(0, 3).unwrap(),
            IndexRange::try_new(0, 8).unwrap(),
        )
    }

    #[test]
    fn decomposition_places_remainder_points_at_both_ends() {
        let topology = DomainTopology::try_new(
            domain(),
            ProcessGrid::try_new(5, 1).unwrap(),
            2,
            BoundaryWidths::default(),
        )
        .unwrap();
        let ranges = topology
            .patches()
            .iter()
            .map(|patch| patch.owned().west_east())
            .collect::<Vec<_>>();

        assert_eq!(
            ranges,
            [
                IndexRange::try_new(0, 3).unwrap(),
                IndexRange::try_new(3, 5).unwrap(),
                IndexRange::try_new(5, 7).unwrap(),
                IndexRange::try_new(7, 10).unwrap(),
                IndexRange::try_new(10, 13).unwrap(),
            ]
        );
    }

    #[test]
    fn memory_bounds_match_wrf_guard_point_formula() {
        let topology = DomainTopology::try_new(
            domain(),
            ProcessGrid::try_new(2, 1).unwrap(),
            2,
            BoundaryWidths::new(3, 0),
        )
        .unwrap();

        assert_eq!(
            topology.patches()[0].memory().west_east(),
            IndexRange::try_new(-3, 9).unwrap()
        );
        assert_eq!(
            topology.patches()[1].memory().west_east(),
            IndexRange::try_new(3, 16).unwrap()
        );
    }

    #[test]
    fn decomposition_preserves_non_one_domain_origins() {
        let offset_domain = DomainBounds::new(
            IndexRange::try_new(-7, 6).unwrap(),
            IndexRange::try_new(4, 7).unwrap(),
            IndexRange::try_new(11, 19).unwrap(),
        );
        let topology = DomainTopology::try_new(
            offset_domain,
            ProcessGrid::try_new(5, 3).unwrap(),
            1,
            BoundaryWidths::default(),
        )
        .unwrap();

        assert_eq!(
            topology.patches()[0].owned().west_east(),
            IndexRange::try_new(-7, -4).unwrap()
        );
        assert_eq!(
            topology.patches().last().unwrap().owned().south_north(),
            IndexRange::try_new(16, 19).unwrap()
        );
    }

    #[test]
    fn tiles_extend_edge_execution_then_clip_to_domain() {
        let topology = DomainTopology::try_new(
            domain(),
            ProcessGrid::try_new(2, 1).unwrap(),
            2,
            BoundaryWidths::default(),
        )
        .unwrap();
        let patch = topology.patches()[0];
        let requested = HorizontalBounds::new(
            IndexRange::try_new(-1, 8).unwrap(),
            patch.owned().south_north(),
        );
        let tiles = topology
            .create_tiles(
                patch.patch_id(),
                TileGrid::try_new(3, 1).unwrap(),
                requested,
            )
            .unwrap();

        assert_eq!(tiles[0].execution().west_east().start(), 0);
        assert_eq!(tiles.last().unwrap().execution().west_east().end(), 8);
    }

    #[test]
    fn invalid_process_grid_fails_before_topology_exists() {
        let result = DomainTopology::try_new(
            domain(),
            ProcessGrid::try_new(14, 1).unwrap(),
            1,
            BoundaryWidths::default(),
        );

        assert_eq!(
            result,
            Err(TopologyError::TooManyProcessColumns {
                process_columns: 14,
                west_east_points: 13,
            })
        );
    }
}
