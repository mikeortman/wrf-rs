//! Emits stage and checkpoint raw bits for the pinned ARW Kessler trajectory.
//!
//! Exceptional sentinels in inactive storage retain their IEEE-754 sign,
//! payload, and infinity bits. Active non-finite Kessler evolution is outside
//! the cross-toolchain contract because WRF `MIN`/`MAX` propagation differs
//! between supported GNU Fortran versions.

use std::error::Error;

use wrf_compute::{ComputeBackend, CpuBackend, CpuField, FieldStorage, GridShape};
use wrf_physics::{
    ArwMicrophysicsControls, ArwMicrophysicsStage, ArwMicrophysicsStageView, ArwMicrophysicsState,
    ArwMicrophysicsTrajectory, MicrophysicsBoundaryPolicy, MicrophysicsDriverDomain,
    MicrophysicsTile,
};
use wrf_registry::{RegistryParser, RuntimeConfigurationChoice};

const WEST_EAST_POINTS: usize = 6;
const SOUTH_NORTH_POINTS: usize = 5;
const BOTTOM_TOP_POINTS: usize = 5;
const W_LEVEL_POINTS: usize = BOTTOM_TOP_POINTS + 1;
const STEP_COUNT: usize = 3;
const TIME_STEP: f32 = 60.0;
const GRAVITY: f32 = 9.81;
const QUIET_NAN_BITS: u32 = 0x7FC0_0000;
const POSITIVE_INFINITY_BITS: u32 = 0x7F80_0000;

const REGISTRY: &str = r#"dimspec i 1 standard_domain x west_east
dimspec k 2 standard_domain z bottom_top
dimspec j 3 standard_domain y south_north
rconfig integer mp_physics namelist,physics 1 -1 - mp_physics "" ""
state real - ikjftb moist 1 - - - - -
state real qv ikjftb moist 1 - i0rh01usdf=(bdy_interp:dt) QVAPOR "Water vapor mixing ratio" "kg kg-1"
state real qc ikjftb moist 1 - i0rhusdf=(bdy_interp:dt) QCLOUD "Cloud water mixing ratio" "kg kg-1"
state real qr ikjftb moist 1 - i0rhusdf=(bdy_interp:dt) QRAIN "Rain water mixing ratio" "kg kg-1"
package kesslerscheme mp_physics==1 - moist:qr,qv,qc
"#;

const QRAIN_INDEX: usize = 0;
const WATER_VAPOR_INDEX: usize = 1;
const CLOUD_WATER_INDEX: usize = 2;

fn main() -> Result<(), Box<dyn Error>> {
    let worker_count = std::env::var("WRF_ORACLE_WORKERS")
        .map(|value| value.parse())
        .unwrap_or(Ok(4))?;
    let backend = CpuBackend::try_with_worker_count(worker_count)?;
    let tiles = [MicrophysicsTile::new(1..5, 1..4)];
    let trajectory = create_trajectory(ArwMicrophysicsControls::try_new(
        TIME_STEP, true, true, 0.5,
    )?)?;

    let mut continuous = OracleState::new(&backend)?;
    let mut continuous_workspace = trajectory.create_workspace(&backend)?;
    for step in 1..=STEP_COUNT {
        apply_observed_step(
            "moist_heating_full.continuous",
            step,
            &backend,
            &trajectory,
            &mut continuous,
            &tiles,
            &mut continuous_workspace,
        )?;
    }
    emit_state("moist_heating_full.continuous.final", &continuous);

    let mut restarted = OracleState::new(&backend)?;
    let mut first_workspace = trajectory.create_workspace(&backend)?;
    apply_observed_step(
        "moist_heating_full.restarted",
        1,
        &backend,
        &trajectory,
        &mut restarted,
        &tiles,
        &mut first_workspace,
    )?;
    let checkpoint = restarted.clone();
    emit_state("moist_heating_full.restarted.checkpoint", &checkpoint);

    restarted = OracleState::new(&backend)?;
    restarted.restore_mutable_state(&checkpoint);
    let mut restarted_workspace = trajectory.create_workspace(&backend)?;
    for step in 2..=STEP_COUNT {
        apply_observed_step(
            "moist_heating_full.restarted",
            step,
            &backend,
            &trajectory,
            &mut restarted,
            &tiles,
            &mut restarted_workspace,
        )?;
    }
    emit_state("moist_heating_full.restarted.final", &restarted);

    assert_mutable_state_eq(&continuous, &restarted);
    run_single_step_case(
        "dry_heating_clamped",
        &backend,
        ArwMicrophysicsControls::try_new(TIME_STEP, false, true, 0.000_01)?,
        &tiles,
    )?;
    run_single_step_case(
        "moist_no_heating",
        &backend,
        ArwMicrophysicsControls::try_new(TIME_STEP, true, false, 0.5)?,
        &tiles,
    )?;
    run_single_step_case(
        "partial_with_inactive_tile",
        &backend,
        ArwMicrophysicsControls::try_new(TIME_STEP, true, true, 0.5)?,
        &[
            MicrophysicsTile::new(8..10, 8..10),
            MicrophysicsTile::new(2..4, 2..4),
        ],
    )?;
    run_exceptional_case(&backend, &tiles)?;
    Ok(())
}

fn run_exceptional_case(
    backend: &CpuBackend,
    tiles: &[MicrophysicsTile],
) -> Result<(), Box<dyn Error>> {
    let trajectory = create_trajectory(ArwMicrophysicsControls::try_new(
        TIME_STEP, true, true, 0.5,
    )?)?;
    let mut state = OracleState::new(backend)?;
    state.seed_exceptional_values();
    let mut workspace = trajectory.create_workspace(backend)?;
    apply_observed_step(
        "exceptional",
        1,
        backend,
        &trajectory,
        &mut state,
        tiles,
        &mut workspace,
    )
}

fn run_single_step_case(
    name: &str,
    backend: &CpuBackend,
    controls: ArwMicrophysicsControls,
    tiles: &[MicrophysicsTile],
) -> Result<(), Box<dyn Error>> {
    let trajectory = create_trajectory(controls)?;
    let mut state = OracleState::new(backend)?;
    let mut workspace = trajectory.create_workspace(backend)?;
    apply_observed_step(
        name,
        1,
        backend,
        &trajectory,
        &mut state,
        tiles,
        &mut workspace,
    )
}

fn create_trajectory(
    controls: ArwMicrophysicsControls,
) -> Result<ArwMicrophysicsTrajectory, Box<dyn Error>> {
    let registry = RegistryParser::parse("Registry.kessler-trajectory", REGISTRY)?;
    let layouts = registry
        .resolve_scalar_array_layouts(&[RuntimeConfigurationChoice::new("mp_physics", 1)])?;
    let moist = layouts
        .iter()
        .find(|layout| layout.scalar_array_name() == "moist")
        .ok_or("Registry did not resolve the Kessler moist layout")?;
    let shape = mass_shape()?;
    let domain = MicrophysicsDriverDomain::try_new(
        shape,
        1..5,
        1..4,
        0..BOTTOM_TOP_POINTS,
        MicrophysicsBoundaryPolicy::open(),
    )?;
    Ok(ArwMicrophysicsTrajectory::try_kessler(
        domain, moist, controls,
    )?)
}

fn apply_observed_step(
    prefix: &str,
    step: usize,
    backend: &CpuBackend,
    trajectory: &ArwMicrophysicsTrajectory,
    state: &mut OracleState,
    tiles: &[MicrophysicsTile],
    workspace: &mut wrf_physics::ArwMicrophysicsWorkspace,
) -> Result<(), Box<dyn Error>> {
    trajectory.apply_step_with_observer(
        backend,
        state.view(),
        tiles,
        workspace,
        |stage, view| emit_stage(prefix, step, stage, &view),
    )?;
    Ok(())
}

fn emit_stage(
    prefix: &str,
    step: usize,
    stage: ArwMicrophysicsStage,
    view: &ArwMicrophysicsStageView<'_>,
) {
    match stage {
        ArwMicrophysicsStage::Prepared => {
            let stage_prefix = format!("{prefix}.step{step}.prepared");
            emit_volume(
                &stage_prefix,
                "perturbation_theta",
                view.perturbation_potential_temperature(),
            );
            emit_moisture(&stage_prefix, view);
            emit_volume(
                &stage_prefix,
                "full_theta",
                view.full_potential_temperature(),
            );
            emit_volume(&stage_prefix, "density", view.dry_air_density());
            emit_volume(&stage_prefix, "exner", view.exner_function());
            emit_volume(&stage_prefix, "height", view.height());
            emit_volume(&stage_prefix, "dz8w", view.vertical_layer_thickness());
            emit_horizontal(&stage_prefix, "rainnc", view.accumulated_precipitation());
            emit_horizontal(&stage_prefix, "rainncv", view.step_precipitation());
        }
        ArwMicrophysicsStage::MicrophysicsApplied => {
            let stage_prefix = format!("{prefix}.step{step}.microphysics");
            emit_volume(
                &stage_prefix,
                "full_theta",
                view.full_potential_temperature(),
            );
            emit_moisture(&stage_prefix, view);
            emit_horizontal(&stage_prefix, "rainnc", view.accumulated_precipitation());
            emit_horizontal(&stage_prefix, "rainncv", view.step_precipitation());
        }
        ArwMicrophysicsStage::Finished => {
            let stage_prefix = format!("{prefix}.step{step}.finished");
            emit_volume(
                &stage_prefix,
                "perturbation_theta",
                view.perturbation_potential_temperature(),
            );
            emit_moisture(&stage_prefix, view);
            emit_volume(&stage_prefix, "theta_tendency", view.h_diabatic());
            emit_volume(&stage_prefix, "qv_tendency", view.qv_diabatic());
            emit_volume(&stage_prefix, "qc_tendency", view.qc_diabatic());
            emit_volume(
                &stage_prefix,
                "dry_theta_perturbation",
                view.perturbation_dry_potential_temperature(),
            );
            emit_horizontal(&stage_prefix, "rainnc", view.accumulated_precipitation());
            emit_horizontal(&stage_prefix, "rainncv", view.step_precipitation());
        }
    }
}

fn emit_moisture(prefix: &str, view: &ArwMicrophysicsStageView<'_>) {
    let fields = view.moisture_species_fields();
    emit_volume(prefix, "qv", &fields[WATER_VAPOR_INDEX]);
    emit_volume(prefix, "qc", &fields[CLOUD_WATER_INDEX]);
    emit_volume(prefix, "qr", &fields[QRAIN_INDEX]);
}

#[derive(Clone)]
struct OracleState {
    perturbation_potential_temperature: CpuField<f32>,
    moisture_species_fields: Vec<CpuField<f32>>,
    perturbation_inverse_density: CpuField<f32>,
    base_inverse_density: CpuField<f32>,
    perturbation_pressure: CpuField<f32>,
    base_pressure: CpuField<f32>,
    perturbation_geopotential: CpuField<f32>,
    base_geopotential: CpuField<f32>,
    accumulated_precipitation: CpuField<f32>,
    step_precipitation: CpuField<f32>,
}

impl OracleState {
    fn new(backend: &CpuBackend) -> Result<Self, Box<dyn Error>> {
        let shape = mass_shape()?;
        let w_shape = GridShape::try_new(WEST_EAST_POINTS, SOUTH_NORTH_POINTS, W_LEVEL_POINTS)?;
        let mut state = Self {
            perturbation_potential_temperature: backend.create_field(shape, 0.0)?,
            moisture_species_fields: (0..3)
                .map(|_| backend.create_field(shape, 0.0))
                .collect::<Result<Vec<_>, _>>()?,
            perturbation_inverse_density: backend.create_field(shape, 0.0)?,
            base_inverse_density: backend.create_field(shape, 0.0)?,
            perturbation_pressure: backend.create_field(shape, 0.0)?,
            base_pressure: backend.create_field(shape, 0.0)?,
            perturbation_geopotential: backend.create_field(w_shape, 0.0)?,
            base_geopotential: backend.create_field(w_shape, 0.0)?,
            accumulated_precipitation: backend.create_field(shape.horizontal_shape(), 0.0)?,
            step_precipitation: backend.create_field(shape.horizontal_shape(), 0.0)?,
        };
        state.initialize();
        Ok(state)
    }

    fn initialize(&mut self) {
        let shape = self.perturbation_potential_temperature.shape();
        for south_north_index in 0..SOUTH_NORTH_POINTS {
            for bottom_top_index in 0..BOTTOM_TOP_POINTS {
                for west_east_index in 0..WEST_EAST_POINTS {
                    let index =
                        linear_index(shape, west_east_index, bottom_top_index, south_north_index);
                    self.perturbation_potential_temperature.values_mut()[index] =
                        -20.0 + 0.7 * west_east_index as f32 + 0.3 * bottom_top_index as f32
                            - 0.4 * south_north_index as f32;
                    self.moisture_species_fields[WATER_VAPOR_INDEX].values_mut()[index] =
                        0.002 + 0.001 * ((west_east_index + 2 * bottom_top_index) % 8) as f32;
                    self.moisture_species_fields[CLOUD_WATER_INDEX].values_mut()[index] =
                        if (west_east_index + bottom_top_index) % 3 == 0 {
                            0.002
                        } else {
                            0.0002
                        };
                    self.moisture_species_fields[QRAIN_INDEX].values_mut()[index] =
                        [0.0, 0.0005, 0.005, 0.02][(west_east_index + south_north_index) % 4];
                    self.perturbation_inverse_density.values_mut()[index] =
                        0.03 + 0.002 * west_east_index as f32 + 0.001 * south_north_index as f32;
                    self.base_inverse_density.values_mut()[index] =
                        0.84 + 0.07 * bottom_top_index as f32;
                    self.perturbation_pressure.values_mut()[index] =
                        120.0 + 3.0 * west_east_index as f32 - 2.0 * south_north_index as f32;
                    self.base_pressure.values_mut()[index] =
                        92_000.0 - 7_000.0 * bottom_top_index as f32;
                }
            }
            for w_level_index in 0..W_LEVEL_POINTS {
                for west_east_index in 0..WEST_EAST_POINTS {
                    let index = (south_north_index * W_LEVEL_POINTS + w_level_index)
                        * WEST_EAST_POINTS
                        + west_east_index;
                    let height = 35.0 + 150.0 * w_level_index as f32 + 2.0 * west_east_index as f32;
                    self.perturbation_geopotential.values_mut()[index] =
                        GRAVITY * (0.25 * south_north_index as f32);
                    self.base_geopotential.values_mut()[index] = GRAVITY * height;
                }
            }
            for west_east_index in 0..WEST_EAST_POINTS {
                let index = south_north_index * WEST_EAST_POINTS + west_east_index;
                self.accumulated_precipitation.values_mut()[index] =
                    10.0 + 0.25 * west_east_index as f32 + 0.5 * south_north_index as f32;
                self.step_precipitation.values_mut()[index] = -777.0;
            }
        }
    }

    fn seed_exceptional_values(&mut self) {
        let shape = self.perturbation_potential_temperature.shape();
        self.perturbation_potential_temperature.values_mut()[linear_index(shape, 0, 1, 2)] =
            f32::from_bits(QUIET_NAN_BITS);
        self.moisture_species_fields[WATER_VAPOR_INDEX].values_mut()
            [linear_index(shape, 5, 0, 3)] = f32::from_bits(POSITIVE_INFINITY_BITS);
        self.moisture_species_fields[QRAIN_INDEX].values_mut()[linear_index(shape, 4, 2, 4)] =
            f32::from_bits(QUIET_NAN_BITS);
        self.moisture_species_fields[CLOUD_WATER_INDEX].values_mut()
            [linear_index(shape, 0, 0, 0)] = f32::from_bits(QUIET_NAN_BITS);
    }

    fn view(&mut self) -> ArwMicrophysicsState<'_> {
        ArwMicrophysicsState::new(
            &mut self.perturbation_potential_temperature,
            &mut self.moisture_species_fields,
            &self.perturbation_inverse_density,
            &self.base_inverse_density,
            &self.perturbation_pressure,
            &self.base_pressure,
            &self.perturbation_geopotential,
            &self.base_geopotential,
            &mut self.accumulated_precipitation,
            &mut self.step_precipitation,
        )
    }

    fn restore_mutable_state(&mut self, checkpoint: &Self) {
        self.perturbation_potential_temperature =
            checkpoint.perturbation_potential_temperature.clone();
        self.moisture_species_fields = checkpoint.moisture_species_fields.clone();
        self.accumulated_precipitation = checkpoint.accumulated_precipitation.clone();
        self.step_precipitation = checkpoint.step_precipitation.clone();
    }
}

fn emit_state(prefix: &str, state: &OracleState) {
    emit_volume(
        prefix,
        "perturbation_theta",
        &state.perturbation_potential_temperature,
    );
    emit_volume(
        prefix,
        "qv",
        &state.moisture_species_fields[WATER_VAPOR_INDEX],
    );
    emit_volume(
        prefix,
        "qc",
        &state.moisture_species_fields[CLOUD_WATER_INDEX],
    );
    emit_volume(prefix, "qr", &state.moisture_species_fields[QRAIN_INDEX]);
    emit_horizontal(prefix, "rainnc", &state.accumulated_precipitation);
    emit_horizontal(prefix, "rainncv", &state.step_precipitation);
}

fn assert_mutable_state_eq(left: &OracleState, right: &OracleState) {
    assert_eq!(
        field_bits(&left.perturbation_potential_temperature),
        field_bits(&right.perturbation_potential_temperature)
    );
    for (left_field, right_field) in left
        .moisture_species_fields
        .iter()
        .zip(&right.moisture_species_fields)
    {
        assert_eq!(field_bits(left_field), field_bits(right_field));
    }
    assert_eq!(
        field_bits(&left.accumulated_precipitation),
        field_bits(&right.accumulated_precipitation)
    );
    assert_eq!(
        field_bits(&left.step_precipitation),
        field_bits(&right.step_precipitation)
    );
}

fn emit_volume(prefix: &str, name: &str, field: &CpuField<f32>) {
    emit_field(prefix, name, field);
}

fn emit_horizontal(prefix: &str, name: &str, field: &CpuField<f32>) {
    emit_field(prefix, name, field);
}

fn emit_field(prefix: &str, name: &str, field: &CpuField<f32>) {
    for (index, value) in field.values().iter().enumerate() {
        println!("{prefix}.{name} {index} {:08X}", value.to_bits());
    }
}

fn field_bits(field: &CpuField<f32>) -> Vec<u32> {
    field.values().iter().map(|value| value.to_bits()).collect()
}

fn mass_shape() -> Result<GridShape, Box<dyn Error>> {
    Ok(GridShape::try_new(
        WEST_EAST_POINTS,
        SOUTH_NORTH_POINTS,
        BOTTOM_TOP_POINTS,
    )?)
}

fn linear_index(
    shape: GridShape,
    west_east_index: usize,
    bottom_top_index: usize,
    south_north_index: usize,
) -> usize {
    (south_north_index * shape.bottom_top_points() + bottom_top_index) * shape.west_east_points()
        + west_east_index
}
