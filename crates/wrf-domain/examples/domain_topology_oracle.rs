//! Emits the Rust side of the pinned `task_for_point.c` differential fixture.

use wrf_domain::{
    BoundaryWidths, DomainBounds, DomainTopology, IndexRange, ProcessGrid, TopologyResult,
};

#[derive(Clone, Copy)]
struct TopologyCase {
    ids: i32,
    ide: i32,
    jds: i32,
    jde: i32,
    process_columns: usize,
    process_rows: usize,
}

fn main() -> TopologyResult<()> {
    let cases = [
        TopologyCase {
            ids: 1,
            ide: 13,
            jds: 1,
            jde: 8,
            process_columns: 5,
            process_rows: 3,
        },
        TopologyCase {
            ids: 1,
            ide: 16,
            jds: 1,
            jde: 11,
            process_columns: 4,
            process_rows: 2,
        },
        TopologyCase {
            ids: 1,
            ide: 17,
            jds: 1,
            jde: 9,
            process_columns: 6,
            process_rows: 4,
        },
    ];

    for (case_index, current) in cases.into_iter().enumerate() {
        let domain = DomainBounds::new(
            IndexRange::try_from_fortran_inclusive(current.ids, current.ide)?,
            IndexRange::try_from_fortran_inclusive(1, 1)?,
            IndexRange::try_from_fortran_inclusive(current.jds, current.jde)?,
        );
        let topology = DomainTopology::try_new(
            domain,
            ProcessGrid::try_new(current.process_columns, current.process_rows)?,
            0,
            BoundaryWidths::default(),
        )?;
        for patch in topology.patches() {
            let coordinate = patch.coordinate();
            let owned = patch.owned();
            println!(
                "case={} patch={} column={} row={} ips={} ipe={} jps={} jpe={}",
                case_index,
                patch.patch_id().value(),
                coordinate.column(),
                coordinate.row(),
                owned.west_east().start() + 1,
                owned.west_east().end(),
                owned.south_north().start() + 1,
                owned.south_north().end(),
            );
        }
    }
    Ok(())
}
