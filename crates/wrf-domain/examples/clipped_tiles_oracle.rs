//! Emits clipped tile bounds for comparison with WRF `set_tiles2` semantics.

use wrf_domain::{
    BoundaryWidths, DomainBounds, DomainTopology, HorizontalBounds, IndexRange, ProcessGrid,
    TileGrid,
};

fn main() {
    let topology = DomainTopology::try_new(
        DomainBounds::new(
            IndexRange::try_new(0, 13).unwrap(),
            IndexRange::try_new(0, 1).unwrap(),
            IndexRange::try_new(0, 8).unwrap(),
        ),
        ProcessGrid::try_new(2, 1).unwrap(),
        2,
        BoundaryWidths::default(),
    )
    .unwrap();
    let patch = topology.patches()[0];
    let requested = HorizontalBounds::new(
        IndexRange::try_new(-1, 8).unwrap(),
        IndexRange::try_new(-1, 9).unwrap(),
    );
    let tiles = topology
        .create_tiles(
            patch.patch_id(),
            TileGrid::try_new(3, 2).unwrap(),
            requested,
        )
        .unwrap();

    for tile in tiles {
        let execution = tile.execution();
        println!(
            "tile={} ips={} ipe={} jps={} jpe={}",
            tile.tile_index(),
            execution.west_east().start() + 1,
            execution.west_east().end(),
            execution.south_north().start() + 1,
            execution.south_north().end(),
        );
    }
}
