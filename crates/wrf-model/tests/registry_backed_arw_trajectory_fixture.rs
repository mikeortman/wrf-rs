//! Exact initialization checks for the Registry-backed ARW trajectory fixture.

mod support;

use wrf_compute::CpuBackend;
use wrf_model::{
    ArwColumnField, ArwGeopotentialField, ArwMapField, ArwMassField, ArwModelError,
    ArwRestartVolumeField, ArwWorkspaceColumnField, ArwWorkspaceVolumeField,
};

use support::RegistryBackedArwTrajectoryFixture;

#[test]
fn initializes_complete_fortran_storage_including_upper_w_level() {
    let backend = CpuBackend::try_with_worker_count(1).unwrap();
    let fixture = RegistryBackedArwTrajectoryFixture::new(&backend);

    assert_eq!(fixture.state.mass_shape().west_east_points(), 6);
    assert_eq!(fixture.state.mass_shape().south_north_points(), 6);
    assert_eq!(fixture.state.mass_shape().bottom_top_points(), 6);
    assert_eq!(fixture.state.geopotential_shape().bottom_top_points(), 6);
    assert_eq!(
        fixture
            .state
            .mass_field_values(ArwMassField::CurrentWestEastVelocity)[1],
        0.201
    );
    assert_eq!(
        fixture
            .state
            .column_field_values(ArwColumnField::StepPrecipitation)[0]
            .to_bits(),
        (-7_777.0_f32).to_bits()
    );
    let upper_w_index = (6 + 5) * 6 + 1;
    assert_eq!(
        fixture
            .state
            .geopotential_field_values(ArwGeopotentialField::CurrentPerturbation)[upper_w_index]
            .to_bits(),
        15.0_f32.to_bits()
    );
    assert_eq!(
        fixture
            .state
            .restart_volume_field_values(ArwRestartVolumeField::VerticalMassFlux)[0]
            .to_bits(),
        (-7_777.0_f32).to_bits()
    );
    assert_eq!(
        fixture
            .state
            .restart_volume_field_values(ArwRestartVolumeField::AverageVerticalMassFlux)[0],
        0.2
    );
    assert_eq!(
        fixture
            .state
            .restart_volume_field_values(ArwRestartVolumeField::DiabaticHeating)[0],
        0.0001
    );
    assert_eq!(
        fixture
            .state
            .restart_volume_field_values(ArwRestartVolumeField::WaterVaporDiabaticTendency)[0],
        0.0
    );
    assert_eq!(
        fixture
            .state
            .restart_volume_field_values(ArwRestartVolumeField::PressurePointGeopotential)[0]
            .to_bits(),
        (-7_777.0_f32).to_bits()
    );
    assert_eq!(
        fixture
            .state
            .restart_volume_field_values(ArwRestartVolumeField::CloudWaterDiabaticTendency)[0],
        0.0
    );
    for field in [
        ArwRestartVolumeField::DryAirDensity,
        ArwRestartVolumeField::PerturbationDryPotentialTemperature,
    ] {
        assert_eq!(
            fixture.state.restart_volume_field_values(field)[0].to_bits(),
            (-7_777.0_f32).to_bits()
        );
    }
    assert_eq!(
        fixture.state.map_field_values(ArwMapField::MassPointX)[0],
        1.0
    );
    assert_eq!(
        fixture.state.map_field_values(ArwMapField::TerrainHeight)[0],
        0.0
    );
    assert_eq!(
        fixture
            .workspace
            .volume_field_values(ArwWorkspaceVolumeField::ForwardPotentialTemperatureTendency)[0],
        0.007
    );
    assert_eq!(
        fixture
            .workspace
            .column_field_values(ArwWorkspaceColumnField::SavedPerturbationColumnMass,)[0],
        0.2
    );
}

#[test]
fn pinned_registry_fixture_executes_the_accepted_trajectory() {
    let backend = CpuBackend::try_with_worker_count(1).unwrap();
    let mut fixture = RegistryBackedArwTrajectoryFixture::new(&backend);

    fixture
        .model
        .advance_short_trajectory(&backend, &mut fixture.state, &mut fixture.workspace)
        .unwrap();

    assert!(
        fixture
            .state
            .mass_field_values(ArwMassField::CurrentPotentialTemperature)
            .iter()
            .all(|value| value.is_finite())
    );
}

#[test]
fn one_and_four_workers_produce_identical_complete_scientific_bits() {
    let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
    let four_workers = CpuBackend::try_with_worker_count(4).unwrap();
    let mut one = RegistryBackedArwTrajectoryFixture::new(&one_worker);
    let mut four = RegistryBackedArwTrajectoryFixture::new(&four_workers);

    one.model
        .advance_short_trajectory(&one_worker, &mut one.state, &mut one.workspace)
        .unwrap();
    four.model
        .advance_short_trajectory(&four_workers, &mut four.state, &mut four.workspace)
        .unwrap();

    assert_eq!(scientific_bits(&one), scientific_bits(&four));
}

#[test]
fn worker_mismatch_fails_before_any_scientific_storage_changes() {
    let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
    let four_workers = CpuBackend::try_with_worker_count(4).unwrap();
    let mut fixture = RegistryBackedArwTrajectoryFixture::new(&one_worker);
    let before = scientific_bits(&fixture);

    let result = fixture.model.advance_short_trajectory(
        &four_workers,
        &mut fixture.state,
        &mut fixture.workspace,
    );

    assert!(matches!(
        result,
        Err(ArwModelError::WorkspaceWorkerCountMismatch {
            expected: 1,
            actual: 4
        })
    ));
    assert_eq!(scientific_bits(&fixture), before);
}

#[test]
fn recreated_workspace_with_identical_stage_inputs_matches_fresh_execution() {
    let backend = CpuBackend::try_with_worker_count(2).unwrap();
    let mut fresh = RegistryBackedArwTrajectoryFixture::new(&backend);
    let mut recreated = RegistryBackedArwTrajectoryFixture::new(&backend);
    recreated.recreate_workspace(&backend);

    fresh
        .model
        .advance_short_trajectory(&backend, &mut fresh.state, &mut fresh.workspace)
        .unwrap();
    recreated
        .model
        .advance_short_trajectory(&backend, &mut recreated.state, &mut recreated.workspace)
        .unwrap();

    assert_eq!(scientific_bits(&fresh), scientific_bits(&recreated));
}

fn scientific_bits(fixture: &RegistryBackedArwTrajectoryFixture) -> Vec<u32> {
    ArwMassField::ALL
        .into_iter()
        .flat_map(|field| fixture.state.mass_field_values(field).iter())
        .chain(
            ArwGeopotentialField::ALL
                .into_iter()
                .flat_map(|field| fixture.state.geopotential_field_values(field).iter()),
        )
        .chain(
            ArwColumnField::ALL
                .into_iter()
                .flat_map(|field| fixture.state.column_field_values(field).iter()),
        )
        .chain(
            ArwRestartVolumeField::ALL
                .into_iter()
                .flat_map(|field| fixture.state.restart_volume_field_values(field).iter()),
        )
        .chain(
            ArwMapField::ALL
                .into_iter()
                .flat_map(|field| fixture.state.map_field_values(field).iter()),
        )
        .chain(
            fixture
                .state
                .moisture_fields()
                .iter()
                .flat_map(|field| field.values().iter()),
        )
        .chain(
            ArwWorkspaceVolumeField::ALL
                .into_iter()
                .flat_map(|field| fixture.workspace.volume_field_values(field).iter()),
        )
        .chain(
            ArwWorkspaceColumnField::ALL
                .into_iter()
                .flat_map(|field| fixture.workspace.column_field_values(field).iter()),
        )
        .map(|value| value.to_bits())
        .collect()
}
