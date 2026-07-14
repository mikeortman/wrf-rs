//! Emits periodic destinations for comparison with WRF `period.c`.

use wrf_domain::{
    BoundaryWidths, DomainBounds, DomainTopology, HaloExchangePlan, HorizontalPeriodicity,
    HorizontalStaggering, IndexRange, LocalHaloExchange, LocalPatchField, ProcessGrid,
};

fn main() {
    let topology = DomainTopology::try_new(
        DomainBounds::new(
            IndexRange::try_new(0, 10).unwrap(),
            IndexRange::try_new(0, 2).unwrap(),
            IndexRange::try_new(0, 8).unwrap(),
        ),
        ProcessGrid::try_new(2, 2).unwrap(),
        2,
        BoundaryWidths::new(3, 2),
    )
    .unwrap();
    let plan = HaloExchangePlan::try_new(
        topology.clone(),
        2,
        HorizontalPeriodicity::new(true, true),
        HorizontalStaggering::new(true, false),
    )
    .unwrap();
    let mut field = LocalPatchField::try_from_value(topology.clone(), -1_i32).unwrap();
    fill_owned(&mut field);
    LocalHaloExchange::execute(&plan, &mut field).unwrap();

    for patch in topology.patches() {
        let memory = patch.memory();
        for south_north in memory.south_north().start()..memory.south_north().end() {
            for level in memory.bottom_top().start()..memory.bottom_top().end() {
                for west_east in memory.west_east().start()..memory.west_east().end() {
                    if is_periodic_destination(*patch, west_east, south_north) {
                        println!(
                            "rank={} i={} k={} j={} value={}",
                            patch.patch_id().value(),
                            west_east + 1,
                            level + 1,
                            south_north + 1,
                            field
                                .value(patch.patch_id(), west_east, level, south_north)
                                .unwrap()
                        );
                    }
                }
            }
        }
    }
}

fn fill_owned(field: &mut LocalPatchField<i32>) {
    let patches = field.topology().patches().to_vec();
    let vertical = field.topology().domain().bottom_top();
    for patch in patches {
        for south_north in patch.owned().south_north().start()..patch.owned().south_north().end() {
            for level in vertical.start()..vertical.end() {
                for west_east in patch.owned().west_east().start()..patch.owned().west_east().end()
                {
                    field
                        .set_value(
                            patch.patch_id(),
                            west_east,
                            level,
                            south_north,
                            west_east + 100 * south_north + 10_000 * level,
                        )
                        .unwrap();
                }
            }
        }
    }
}

fn is_periodic_destination(
    patch: wrf_domain::PatchBounds,
    west_east: i32,
    south_north: i32,
) -> bool {
    let owned = patch.owned();
    let coordinate = patch.coordinate();
    let periodic_y_transverse = owned.south_north().contains(south_north)
        || (coordinate.row() == 0
            && (owned.south_north().start() - 2..owned.south_north().start())
                .contains(&south_north))
        || (coordinate.row() == 1
            && (owned.south_north().end() - 1..owned.south_north().end() + 1)
                .contains(&south_north));
    let periodic_x_transverse = owned.west_east().contains(west_east)
        || (coordinate.column() == 0 && (-2..0).contains(&west_east))
        || (coordinate.column() == 1
            && (owned.west_east().end() - 1..owned.west_east().end() + 2).contains(&west_east));
    let west_destination =
        coordinate.column() == 0 && (-2..0).contains(&west_east) && periodic_y_transverse;
    let east_destination = coordinate.column() == 1
        && (owned.west_east().end() - 1..owned.west_east().end() + 2).contains(&west_east)
        && periodic_y_transverse;
    let south_destination = coordinate.row() == 0
        && (owned.south_north().start() - 2..owned.south_north().start()).contains(&south_north)
        && periodic_x_transverse;
    let north_destination = coordinate.row() == 1
        && (owned.south_north().end() - 1..owned.south_north().end() + 1).contains(&south_north)
        && periodic_x_transverse;
    west_destination || east_destination || south_destination || north_destination
}
