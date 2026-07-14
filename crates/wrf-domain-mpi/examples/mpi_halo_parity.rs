//! Runs serial and four-rank MPI halo exchanges and compares complete memory.

use mpi::collective::CommunicatorCollectives;
use mpi::topology::Communicator;
use wrf_domain::{
    BoundaryWidths, DomainBounds, DomainTopology, HaloExchangePlan, HorizontalPeriodicity,
    HorizontalStaggering, IndexRange, LocalHaloExchange, LocalPatchField, PatchField, ProcessGrid,
};
use wrf_domain_mpi::MpiHaloExchange;

fn main() {
    let universe = mpi::initialize().expect("MPI must not already be initialized");
    let world = universe.world();
    assert_eq!(world.size(), 4, "parity fixture requires four MPI ranks");

    run_case(
        &world,
        HorizontalPeriodicity::default(),
        HorizontalStaggering::default(),
    );
    world.barrier();
    run_case(
        &world,
        HorizontalPeriodicity::new(true, true),
        HorizontalStaggering::new(true, false),
    );

    if world.rank() == 0 {
        println!("MPI halo exchange matches the deterministic local executor.");
    }
}

fn run_case<CommunicatorType>(
    communicator: &CommunicatorType,
    periodicity: HorizontalPeriodicity,
    staggering: HorizontalStaggering,
) where
    CommunicatorType: Communicator + CommunicatorCollectives,
{
    let topology = create_topology();
    let plan = HaloExchangePlan::try_new(topology.clone(), 2, periodicity, staggering).unwrap();
    let rank = usize::try_from(communicator.rank()).unwrap();
    let patch = topology.patches()[rank];

    let mut mpi_field = PatchField::try_from_value(patch, -1_i32).unwrap();
    fill_patch(&mut mpi_field);
    MpiHaloExchange::execute(communicator, &plan, &mut mpi_field).unwrap();

    let mut local_field = LocalPatchField::try_from_value(topology.clone(), -1_i32).unwrap();
    fill_local(&mut local_field);
    LocalHaloExchange::execute(&plan, &mut local_field).unwrap();

    let memory = patch.memory();
    for south_north in memory.south_north().start()..memory.south_north().end() {
        for level in memory.bottom_top().start()..memory.bottom_top().end() {
            for west_east in memory.west_east().start()..memory.west_east().end() {
                assert_eq!(
                    mpi_field.value(west_east, level, south_north).unwrap(),
                    local_field
                        .value(patch.patch_id(), west_east, level, south_north)
                        .unwrap(),
                    "rank {rank} diverged at ({west_east}, {level}, {south_north})"
                );
            }
        }
    }
}

fn create_topology() -> DomainTopology {
    DomainTopology::try_new(
        DomainBounds::new(
            IndexRange::try_new(0, 10).unwrap(),
            IndexRange::try_new(0, 2).unwrap(),
            IndexRange::try_new(0, 8).unwrap(),
        ),
        ProcessGrid::try_new(2, 2).unwrap(),
        2,
        BoundaryWidths::new(3, 2),
    )
    .unwrap()
}

fn fill_patch(field: &mut PatchField<i32>) {
    let owned = field.bounds().owned();
    let bottom_top = field.bounds().memory().bottom_top();
    for south_north in owned.south_north().start()..owned.south_north().end() {
        for level in bottom_top.start()..bottom_top.end() {
            for west_east in owned.west_east().start()..owned.west_east().end() {
                field
                    .set_value(
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

fn fill_local(field: &mut LocalPatchField<i32>) {
    let patches = field.topology().patches().to_vec();
    for patch in patches {
        let mut patch_field = PatchField::try_from_value(patch, -1_i32).unwrap();
        fill_patch(&mut patch_field);
        let owned = patch.owned();
        let bottom_top = patch.memory().bottom_top();
        for south_north in owned.south_north().start()..owned.south_north().end() {
            for level in bottom_top.start()..bottom_top.end() {
                for west_east in owned.west_east().start()..owned.west_east().end() {
                    field
                        .set_value(
                            patch.patch_id(),
                            west_east,
                            level,
                            south_north,
                            *patch_field.value(west_east, level, south_north).unwrap(),
                        )
                        .unwrap();
                }
            }
        }
    }
}
