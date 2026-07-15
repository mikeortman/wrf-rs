//! Emits raw-bit checkpoints for the Registry-backed accepted-stage trajectory.

use std::error::Error;

use wrf_compute::{CpuBackend, GridShape};
use wrf_model::{
    ArwColumnField, ArwMassField, ArwModelStage, ArwModelStageView, ArwRestartVolumeField,
    ArwWorkspaceColumnField, ArwWorkspaceVolumeField,
};

#[path = "../tests/support/mod.rs"]
mod support;

use support::RegistryBackedArwTrajectoryFixture;

const SENTINEL: f32 = -7_777.0;

struct MicrophysicsBaseline {
    potential_temperature: Vec<f32>,
    moisture: Vec<Vec<f32>>,
    diabatic_heating: Vec<f32>,
    water_vapor_tendency: Vec<f32>,
    cloud_water_tendency: Vec<f32>,
    dry_air_density: Vec<f32>,
    dry_potential_temperature: Vec<f32>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let backend = CpuBackend::try_with_worker_count(1)?;
    let mut fixture = RegistryBackedArwTrajectoryFixture::new(&backend);
    let mass_shape = fixture.state.mass_shape();
    let mut baseline = None;

    fixture.model.advance_short_trajectory_with_observer(
        &backend,
        &mut fixture.state,
        &mut fixture.workspace,
        |stage, view| match view {
            ArwModelStageView::Dynamics { state, workspace } => {
                emit_dynamics(stage, state, workspace);
                if stage == ArwModelStage::AcousticFinalized {
                    baseline = Some(MicrophysicsBaseline {
                        potential_temperature: state
                            .mass_field_values(ArwMassField::CurrentPotentialTemperature)
                            .to_vec(),
                        moisture: state
                            .moisture_fields()
                            .iter()
                            .map(|field| field.values().to_vec())
                            .collect(),
                        diabatic_heating: state
                            .restart_volume_field_values(ArwRestartVolumeField::DiabaticHeating)
                            .to_vec(),
                        water_vapor_tendency: state
                            .restart_volume_field_values(
                                ArwRestartVolumeField::WaterVaporDiabaticTendency,
                            )
                            .to_vec(),
                        cloud_water_tendency: state
                            .restart_volume_field_values(
                                ArwRestartVolumeField::CloudWaterDiabaticTendency,
                            )
                            .to_vec(),
                        dry_air_density: state
                            .restart_volume_field_values(ArwRestartVolumeField::DryAirDensity)
                            .to_vec(),
                        dry_potential_temperature: state
                            .restart_volume_field_values(
                                ArwRestartVolumeField::PerturbationDryPotentialTemperature,
                            )
                            .to_vec(),
                    });
                }
            }
            ArwModelStageView::Microphysics(view) => {
                emit_microphysics(
                    stage,
                    mass_shape,
                    baseline
                        .as_ref()
                        .expect("acoustic baseline precedes physics"),
                    &view,
                );
            }
        },
    )?;
    Ok(())
}

fn emit_dynamics(
    stage: ArwModelStage,
    state: &wrf_model::ArwModelState,
    workspace: &wrf_model::ArwModelWorkspace,
) {
    match stage {
        ArwModelStage::RungeKuttaPrepared => {
            let stage = "01.rk-preparation";
            emit_horizontal(
                stage,
                "mut",
                workspace.column_field_values(ArwWorkspaceColumnField::FullColumnMass),
            );
            emit_horizontal(
                stage,
                "muu",
                workspace.column_field_values(ArwWorkspaceColumnField::WestEastColumnMass),
            );
            emit_horizontal(
                stage,
                "muv",
                workspace.column_field_values(ArwWorkspaceColumnField::SouthNorthColumnMass),
            );
            emit_volume(
                stage,
                "ru",
                workspace.volume_field_values(ArwWorkspaceVolumeField::CoupledWestEastMomentum),
            );
            emit_volume(
                stage,
                "rv",
                workspace.volume_field_values(ArwWorkspaceVolumeField::CoupledSouthNorthMomentum),
            );
            emit_volume(
                stage,
                "rw",
                workspace.volume_field_values(ArwWorkspaceVolumeField::CoupledVerticalMomentum),
            );
            emit_volume(
                stage,
                "ww",
                state.restart_volume_field_values(ArwRestartVolumeField::VerticalMassFlux),
            );
            emit_volume(
                stage,
                "cqu",
                workspace.volume_field_values(ArwWorkspaceVolumeField::WestEastMoistureCoefficient),
            );
            emit_volume(
                stage,
                "cqv",
                workspace
                    .volume_field_values(ArwWorkspaceVolumeField::SouthNorthMoistureCoefficient),
            );
            emit_volume(
                stage,
                "cqw",
                workspace.volume_field_values(ArwWorkspaceVolumeField::VerticalMoistureCoefficient),
            );
            emit_volume(
                stage,
                "alt",
                workspace.volume_field_values(ArwWorkspaceVolumeField::FullInverseDensity),
            );
            emit_volume(
                stage,
                "php",
                state.restart_volume_field_values(ArwRestartVolumeField::PressurePointGeopotential),
            );
        }
        ArwModelStage::DryTendenciesAssembled => {
            let stage = "02.dry-tendency";
            for (name, field) in [
                ("ru_tend", ArwWorkspaceVolumeField::WestEastMomentumTendency),
                (
                    "rv_tend",
                    ArwWorkspaceVolumeField::SouthNorthMomentumTendency,
                ),
                ("rw_tend", ArwWorkspaceVolumeField::VerticalMomentumTendency),
                ("ph_tend", ArwWorkspaceVolumeField::GeopotentialTendency),
                (
                    "t_tend",
                    ArwWorkspaceVolumeField::PotentialTemperatureTendency,
                ),
                (
                    "ruf",
                    ArwWorkspaceVolumeField::ForwardWestEastMomentumTendency,
                ),
                (
                    "rvf",
                    ArwWorkspaceVolumeField::ForwardSouthNorthMomentumTendency,
                ),
                (
                    "rwf",
                    ArwWorkspaceVolumeField::ForwardVerticalMomentumTendency,
                ),
                ("phf", ArwWorkspaceVolumeField::ForwardGeopotentialTendency),
                (
                    "tf",
                    ArwWorkspaceVolumeField::ForwardPotentialTemperatureTendency,
                ),
            ] {
                emit_volume(stage, name, workspace.volume_field_values(field));
            }
            emit_horizontal(
                stage,
                "mu_tend",
                workspace.column_field_values(ArwWorkspaceColumnField::ColumnMassTendency),
            );
            emit_horizontal(
                stage,
                "muf",
                workspace.column_field_values(ArwWorkspaceColumnField::ForwardColumnMassTendency),
            );
        }
        ArwModelStage::AcousticAdvanced | ArwModelStage::AcousticFinalized => {
            let stage = if stage == ArwModelStage::AcousticAdvanced {
                "20.acoustic-3-pressure"
            } else {
                "21.acoustic-finish"
            };
            for (name, field) in [
                ("u2", ArwMassField::CurrentWestEastVelocity),
                ("v2", ArwMassField::CurrentSouthNorthVelocity),
                ("w2", ArwMassField::CurrentVerticalVelocity),
                ("t2", ArwMassField::CurrentPotentialTemperature),
            ] {
                emit_volume(stage, name, state.mass_field_values(field));
            }
            emit_volume(
                stage,
                "ph2",
                workspace
                    .volume_field_values(ArwWorkspaceVolumeField::CurrentPerturbationGeopotential),
            );
            emit_volume(
                stage,
                "ww",
                state.restart_volume_field_values(ArwRestartVolumeField::VerticalMassFlux),
            );
            emit_volume(
                stage,
                "al",
                state.mass_field_values(ArwMassField::PerturbationInverseDensity),
            );
            emit_volume(
                stage,
                "p",
                state.mass_field_values(ArwMassField::PerturbationPressure),
            );
            emit_volume(
                stage,
                "pm1",
                workspace
                    .volume_field_values(ArwWorkspaceVolumeField::PreviousPressurePerturbation),
            );
            emit_volume(
                stage,
                "ru_m",
                workspace.volume_field_values(ArwWorkspaceVolumeField::AverageWestEastMassFlux),
            );
            emit_volume(
                stage,
                "rv_m",
                workspace.volume_field_values(ArwWorkspaceVolumeField::AverageSouthNorthMassFlux),
            );
            emit_volume(
                stage,
                "ww_m",
                state.restart_volume_field_values(ArwRestartVolumeField::AverageVerticalMassFlux),
            );
            emit_horizontal(
                stage,
                "mu2",
                state.column_field_values(ArwColumnField::CurrentPerturbationMass),
            );
            emit_horizontal(
                stage,
                "muts",
                workspace.column_field_values(ArwWorkspaceColumnField::FinalFullColumnMass),
            );
            emit_horizontal(
                stage,
                "mudf",
                workspace.column_field_values(ArwWorkspaceColumnField::DivergenceDampingColumnMass),
            );
        }
        _ => {}
    }
}

fn emit_microphysics(
    stage: ArwModelStage,
    mass_shape: GridShape,
    baseline: &MicrophysicsBaseline,
    view: &wrf_physics::ArwMicrophysicsStageView<'_>,
) {
    let stage = match stage {
        ArwModelStage::MicrophysicsPrepared => "22.microphysics-prepared",
        ArwModelStage::MicrophysicsApplied => "23.kessler",
        ArwModelStage::MicrophysicsFinished => "24.microphysics-finished",
        _ => return,
    };
    emit_expanded(
        stage,
        "t2",
        mass_shape,
        &baseline.potential_temperature,
        view.perturbation_potential_temperature().values(),
    );
    emit_expanded(
        stage,
        "th_phy",
        mass_shape,
        &vec![SENTINEL; mass_shape.point_count()],
        view.full_potential_temperature().values(),
    );
    emit_expanded(
        stage,
        "rho",
        mass_shape,
        &baseline.dry_air_density,
        view.dry_air_density().values(),
    );
    emit_expanded(
        stage,
        "pi_phy",
        mass_shape,
        &vec![SENTINEL; mass_shape.point_count()],
        view.exner_function().values(),
    );
    emit_expanded(
        stage,
        "z",
        mass_shape,
        &vec![SENTINEL; mass_shape.point_count()],
        view.height().values(),
    );
    emit_expanded_with_top_zero(
        stage,
        "dz8w",
        mass_shape,
        &vec![SENTINEL; mass_shape.point_count()],
        view.vertical_layer_thickness().values(),
    );
    for (name, compact, padded) in [
        (
            "qv",
            &view.moisture_species_fields()[0],
            &baseline.moisture[0],
        ),
        (
            "qc",
            &view.moisture_species_fields()[1],
            &baseline.moisture[1],
        ),
        (
            "qr",
            &view.moisture_species_fields()[2],
            &baseline.moisture[2],
        ),
    ] {
        emit_expanded(stage, name, mass_shape, padded, compact.values());
    }
    emit_expanded(
        stage,
        "h_diabatic",
        mass_shape,
        &baseline.diabatic_heating,
        view.h_diabatic().values(),
    );
    emit_expanded(
        stage,
        "qv_diabatic",
        mass_shape,
        &baseline.water_vapor_tendency,
        view.qv_diabatic().values(),
    );
    emit_expanded(
        stage,
        "qc_diabatic",
        mass_shape,
        &baseline.cloud_water_tendency,
        view.qc_diabatic().values(),
    );
    if stage == "24.microphysics-finished" {
        emit_expanded(
            stage,
            "th_phy_m_t0",
            mass_shape,
            &baseline.dry_potential_temperature,
            view.perturbation_dry_potential_temperature().values(),
        );
    } else {
        emit_volume(stage, "th_phy_m_t0", &baseline.dry_potential_temperature);
    }
    emit_horizontal(stage, "rainnc", view.accumulated_precipitation().values());
    emit_horizontal(stage, "rainncv", view.step_precipitation().values());
}

fn emit_expanded(
    stage: &str,
    name: &str,
    padded_shape: GridShape,
    baseline: &[f32],
    compact: &[f32],
) {
    let values = expanded_values(padded_shape, baseline, compact);
    emit_volume(stage, name, &values);
}

fn emit_expanded_with_top_zero(
    stage: &str,
    name: &str,
    padded_shape: GridShape,
    baseline: &[f32],
    compact: &[f32],
) {
    let mut values = expanded_values(padded_shape, baseline, compact);
    let west_east_points = padded_shape.west_east_points();
    let top_level = padded_shape.bottom_top_points() - 1;
    for south_north in 1..padded_shape.south_north_points() - 1 {
        for west_east in 1..padded_shape.west_east_points() - 1 {
            let index = (south_north * padded_shape.bottom_top_points() + top_level)
                * west_east_points
                + west_east;
            values[index] = 0.0;
        }
    }
    emit_volume(stage, name, &values);
}

fn expanded_values(padded_shape: GridShape, baseline: &[f32], compact: &[f32]) -> Vec<f32> {
    let mut values = baseline.to_vec();
    let west_east_points = padded_shape.west_east_points();
    let compact_levels = padded_shape.bottom_top_points() - 2;
    for south_north in 1..padded_shape.south_north_points() - 1 {
        for compact_level in 0..compact_levels {
            for west_east in 1..padded_shape.west_east_points() - 1 {
                let compact_index =
                    (south_north * compact_levels + compact_level) * west_east_points + west_east;
                let padded_index =
                    (south_north * padded_shape.bottom_top_points() + compact_level + 1)
                        * west_east_points
                        + west_east;
                values[padded_index] = compact[compact_index];
            }
        }
    }
    values
}

fn emit_volume(stage: &str, name: &str, values: &[f32]) {
    for (index, value) in values.iter().enumerate() {
        println!("{stage} {name} {index} {:08X}", value.to_bits());
    }
}

fn emit_horizontal(stage: &str, name: &str, values: &[f32]) {
    emit_volume(stage, name, values);
}
