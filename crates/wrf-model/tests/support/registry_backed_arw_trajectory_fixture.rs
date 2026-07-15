use wrf_compute::{CpuBackend, GridShape};
use wrf_dynamics::{
    AcousticHorizontalBoundaryPolicy, AcousticMassThetaBoundaryPolicy,
    AcousticMassThetaLateralDomain, AcousticMassThetaWestEastPeriodicity, AcousticPressureMode,
    AcousticRelaxationZone, AcousticSouthNorthBoundary, AcousticStepFinalizationControls,
    AcousticStepFinalizationPhase, AcousticStepPreparationPhase, AcousticTrajectoryControls,
    AcousticVerticalAdvection, AcousticVerticalBoundaryPolicy, AcousticVerticalDamping,
    AcousticVerticalLateralDomain, AcousticVerticalWestEastPeriodicity, AcousticWestEastBoundary,
    AcousticWestEastPeriodicity, VerticalAcousticTopBoundary,
};
use wrf_model::{
    ArwColumnField, ArwGeopotentialField, ArwMapField, ArwMassField, ArwModelCoefficients,
    ArwModelControls, ArwModelGeometry, ArwModelState, ArwModelWorkspace, ArwRegistryBinding,
    ArwRestartVolumeField, ArwWorkspaceColumnField, ArwWorkspaceVolumeField, RegistryBoundArwModel,
};
use wrf_physics::ArwMicrophysicsControls;
use wrf_registry::{RegistryParser, RuntimeConfigurationChoice};

const REGISTRY: &str =
    include_str!("../../../../parity/registry-backed-arw-trajectory/Registry.model");
const SENTINEL: f32 = -7_777.0;

/// Exact Rust input fixture for the pinned accepted-stage Fortran projection.
pub struct RegistryBackedArwTrajectoryFixture {
    /// Registry-bound accepted-stage model.
    pub model: RegistryBoundArwModel,
    /// Restart-owned state initialized from `initialize_fixture`.
    pub state: ArwModelState,
    /// Reusable diagnostics and tendencies initialized from `initialize_fixture`.
    pub workspace: ArwModelWorkspace,
}

impl RegistryBackedArwTrajectoryFixture {
    /// Parses the pinned Registry projection and initializes every allocated value.
    pub fn new(backend: &CpuBackend) -> Self {
        let registry = RegistryParser::parse("Registry.model", REGISTRY).unwrap();
        let binding = ArwRegistryBinding::try_new(
            &registry,
            &[RuntimeConfigurationChoice::new("mp_physics", 1)],
        )
        .unwrap();
        let geometry = ArwModelGeometry::try_new(4, 4, 4).unwrap();
        let coefficients = coefficients(geometry.shape());
        let model =
            RegistryBoundArwModel::try_kessler(binding.clone(), geometry, coefficients, controls())
                .unwrap();
        let mut state = model.create_state(backend).unwrap();
        let mut workspace = model.create_workspace(backend).unwrap();
        initialize_state(&binding, &mut state);
        initialize_workspace(&mut workspace);
        Self {
            model,
            state,
            workspace,
        }
    }

    /// Recreates nonrestart storage and restores identical projected stage inputs.
    #[allow(dead_code)]
    pub fn recreate_workspace(&mut self, backend: &CpuBackend) {
        self.workspace = self.model.create_workspace(backend).unwrap();
        initialize_workspace(&mut self.workspace);
    }
}

fn coefficients(shape: GridShape) -> ArwModelCoefficients {
    let level_count = shape.bottom_top_points();
    let mut c1h = Vec::with_capacity(level_count);
    let mut c2h = Vec::with_capacity(level_count);
    let mut c1f = Vec::with_capacity(level_count);
    let mut c2f = Vec::with_capacity(level_count);
    for level in 0..level_count {
        let level = level as f32;
        c1h.push(0.60 + 0.002 * level);
        c2h.push(0.40 - 0.001 * level);
        c1f.push(0.55 + 0.002 * level);
        c2f.push(0.45 - 0.001 * level);
    }
    ArwModelCoefficients::try_new(
        level_count,
        c1h,
        c2h,
        c1f,
        c2f,
        vec![1.0; level_count],
        vec![0.20; level_count],
        vec![1.0; level_count],
        vec![1.0; level_count],
        vec![0.60; level_count],
        vec![0.40; level_count],
    )
    .unwrap()
}

fn controls() -> ArwModelControls {
    let horizontal_boundaries = AcousticHorizontalBoundaryPolicy::new(
        AcousticRelaxationZone::Disabled,
        AcousticWestEastPeriodicity::Nonperiodic,
        AcousticWestEastBoundary::Closed,
        AcousticWestEastBoundary::Closed,
        AcousticSouthNorthBoundary::Closed,
        AcousticSouthNorthBoundary::Closed,
    );
    let mass_theta_boundaries = AcousticMassThetaBoundaryPolicy::new(
        AcousticMassThetaLateralDomain::Global,
        AcousticMassThetaWestEastPeriodicity::Nonperiodic,
    );
    let vertical_boundaries = AcousticVerticalBoundaryPolicy::new(
        AcousticVerticalLateralDomain::Global,
        AcousticVerticalWestEastPeriodicity::Nonperiodic,
    );
    let acoustic = AcousticTrajectoryControls::try_new(
        AcousticStepPreparationPhase::FirstSubstep,
        3,
        0.1,
        0.1,
        0.000_001,
        9.81,
        300.0,
        0.1,
        0.0,
        0.0,
        [0.5, 0.3, 0.2],
        AcousticPressureMode::Nonhydrostatic,
        VerticalAcousticTopBoundary::Nonrigid,
        horizontal_boundaries,
        mass_theta_boundaries,
        vertical_boundaries,
        AcousticVerticalAdvection::StaggeredGeopotentialGradient,
        AcousticVerticalDamping::Disabled,
    )
    .unwrap();
    let finalization = AcousticStepFinalizationControls::try_new(
        3,
        0.000_001,
        AcousticStepFinalizationPhase::Intermediate,
    )
    .unwrap();
    let microphysics = ArwMicrophysicsControls::try_new(1.0, true, true, 10.0).unwrap();
    ArwModelControls::new(acoustic, finalization, microphysics)
}

fn initialize_state(binding: &ArwRegistryBinding, state: &mut ArwModelState) {
    let shape = state.mass_shape();
    for south_north in 0..shape.south_north_points() {
        for bottom_top in 0..shape.bottom_top_points() {
            for west_east in 0..shape.west_east_points() {
                let index = linear_index(shape, west_east, bottom_top, south_north);
                let i = west_east as i32;
                let j = south_north as i32;
                let k = bottom_top as i32;
                let u = 0.20 + 0.001 * (i + 2 * k - j) as f32;
                let v = 0.15 + 0.001 * (2 * i - k + j) as f32;
                let w = 0.05 + 0.001 * (i + k + j) as f32;
                let t = -10.0 + 0.02 * (i + k - j) as f32;
                state.mass_field_values_mut(ArwMassField::PreviousWestEastVelocity)[index] = u;
                state.mass_field_values_mut(ArwMassField::CurrentWestEastVelocity)[index] = u;
                state.mass_field_values_mut(ArwMassField::PreviousSouthNorthVelocity)[index] = v;
                state.mass_field_values_mut(ArwMassField::CurrentSouthNorthVelocity)[index] = v;
                state.mass_field_values_mut(ArwMassField::PreviousVerticalVelocity)[index] = w;
                state.mass_field_values_mut(ArwMassField::CurrentVerticalVelocity)[index] = w;
                state.mass_field_values_mut(ArwMassField::PreviousPotentialTemperature)[index] = t;
                state.mass_field_values_mut(ArwMassField::CurrentPotentialTemperature)[index] = t;
                state.mass_field_values_mut(ArwMassField::PerturbationPressure)[index] =
                    100.0 + (i - j) as f32;
                state.mass_field_values_mut(ArwMassField::PerturbationInverseDensity)[index] =
                    0.03 + 0.001 * (i + j) as f32;
                state.mass_field_values_mut(ArwMassField::BasePressure)[index] =
                    80_000.0 - 1_000.0 * k as f32;
                state.mass_field_values_mut(ArwMassField::BaseInverseDensity)[index] =
                    0.84 + 0.02 * k as f32;
                for (field, value) in [
                    (ArwRestartVolumeField::VerticalMassFlux, SENTINEL),
                    (ArwRestartVolumeField::AverageVerticalMassFlux, 0.2),
                    (ArwRestartVolumeField::PressurePointGeopotential, SENTINEL),
                    (ArwRestartVolumeField::DiabaticHeating, 0.0001),
                    (ArwRestartVolumeField::WaterVaporDiabaticTendency, 0.0),
                    (ArwRestartVolumeField::CloudWaterDiabaticTendency, 0.0),
                    (ArwRestartVolumeField::DryAirDensity, SENTINEL),
                    (
                        ArwRestartVolumeField::PerturbationDryPotentialTemperature,
                        SENTINEL,
                    ),
                ] {
                    state.restart_volume_field_values_mut(field)[index] = value;
                }

                for member in binding.moisture_layout().members() {
                    let value = match member.name() {
                        "qv" => 0.006 + 0.0001 * ((i + 2 * k + j) % 7) as f32,
                        "qc" => 0.001 + 0.0001 * ((i + k) % 3) as f32,
                        "qr" => 0.0005 + 0.0002 * ((i + j) % 4) as f32,
                        name => panic!("unexpected Kessler moisture member {name}"),
                    };
                    state
                        .moisture_field_values_mut(member.rust_dense_scalar_index())
                        .unwrap()[index] = value;
                }
            }
        }
    }

    let w_shape = state.geopotential_shape();
    for south_north in 0..w_shape.south_north_points() {
        for bottom_top in 0..w_shape.bottom_top_points() {
            for west_east in 0..w_shape.west_east_points() {
                let index = linear_index(w_shape, west_east, bottom_top, south_north);
                let i = west_east as i32;
                let j = south_north as i32;
                let k = bottom_top as i32;
                let ph = 10.0 + k as f32 + 0.01 * (i - j) as f32;
                let phb = 1_000.0 + 50.0 * k as f32;
                state.geopotential_field_values_mut(ArwGeopotentialField::PreviousPerturbation)
                    [index] = ph;
                state.geopotential_field_values_mut(ArwGeopotentialField::CurrentPerturbation)
                    [index] = ph;
                state.geopotential_field_values_mut(ArwGeopotentialField::BaseState)[index] = phb;
            }
        }
    }

    let horizontal = shape.horizontal_shape();
    for south_north in 0..horizontal.south_north_points() {
        for west_east in 0..horizontal.west_east_points() {
            let index = linear_index(horizontal, west_east, 0, south_north);
            let i = west_east as i32;
            let j = south_north as i32;
            let mu = 1.0 + 0.01 * (i + j) as f32;
            state.column_field_values_mut(ArwColumnField::PreviousPerturbationMass)[index] = mu;
            state.column_field_values_mut(ArwColumnField::CurrentPerturbationMass)[index] = mu;
            state.column_field_values_mut(ArwColumnField::BaseMass)[index] =
                10.0 + 0.1 * i as f32 + 0.2 * j as f32;
            state.column_field_values_mut(ArwColumnField::AccumulatedPrecipitation)[index] =
                1.0 + 0.1 * (i + j) as f32;
            state.column_field_values_mut(ArwColumnField::StepPrecipitation)[index] = SENTINEL;
            for field in [
                ArwMapField::WestEastVelocityX,
                ArwMapField::WestEastVelocityY,
                ArwMapField::SouthNorthVelocityX,
                ArwMapField::InverseSouthNorthVelocityX,
                ArwMapField::SouthNorthVelocityY,
                ArwMapField::MassPointX,
                ArwMapField::MassPointY,
            ] {
                state.map_field_values_mut(field)[index] = 1.0;
            }
            state.map_field_values_mut(ArwMapField::TerrainHeight)[index] = 0.0;
        }
    }
}

fn initialize_workspace(workspace: &mut ArwModelWorkspace) {
    let shape = workspace.shape();
    for south_north in 0..shape.south_north_points() {
        for bottom_top in 0..shape.bottom_top_points() {
            for west_east in 0..shape.west_east_points() {
                let index = linear_index(shape, west_east, bottom_top, south_north);
                let i = west_east as i32;
                let j = south_north as i32;
                let k = bottom_top as i32;
                set_volume(
                    workspace,
                    ArwWorkspaceVolumeField::PreviousPerturbationGeopotential,
                    index,
                    10.0 + k as f32 + 0.01 * (i - j) as f32,
                );
                set_volume(
                    workspace,
                    ArwWorkspaceVolumeField::CurrentPerturbationGeopotential,
                    index,
                    10.0 + k as f32 + 0.01 * (i - j) as f32,
                );
                set_volume(
                    workspace,
                    ArwWorkspaceVolumeField::BaseGeopotential,
                    index,
                    1_000.0 + 50.0 * k as f32,
                );
                for field in [
                    ArwWorkspaceVolumeField::CoupledWestEastMomentum,
                    ArwWorkspaceVolumeField::CoupledSouthNorthMomentum,
                    ArwWorkspaceVolumeField::CoupledVerticalMomentum,
                    ArwWorkspaceVolumeField::WestEastMoistureCoefficient,
                    ArwWorkspaceVolumeField::SouthNorthMoistureCoefficient,
                    ArwWorkspaceVolumeField::VerticalMoistureCoefficient,
                    ArwWorkspaceVolumeField::FullInverseDensity,
                    ArwWorkspaceVolumeField::GeopotentialRightHandSide,
                ] {
                    set_volume(workspace, field, index, SENTINEL);
                }
                for (field, value) in [
                    (ArwWorkspaceVolumeField::SavedWestEastVelocity, 0.09),
                    (ArwWorkspaceVolumeField::SavedSouthNorthVelocity, -0.08),
                    (ArwWorkspaceVolumeField::SavedVerticalVelocity, 0.07),
                    (ArwWorkspaceVolumeField::SavedPotentialTemperature, 0.05),
                    (
                        ArwWorkspaceVolumeField::SavedPerturbationGeopotential,
                        -0.06,
                    ),
                    (ArwWorkspaceVolumeField::SavedVerticalMassFlux, 0.2),
                    (ArwWorkspaceVolumeField::SavedPressureCoefficient, 0.2),
                    (ArwWorkspaceVolumeField::PreviousPressurePerturbation, 0.2),
                    (ArwWorkspaceVolumeField::LowerDiagonal, 0.2),
                    (ArwWorkspaceVolumeField::InverseEliminatedDiagonal, 0.2),
                    (ArwWorkspaceVolumeField::UpperEliminationFactor, 0.2),
                    (ArwWorkspaceVolumeField::TimeAveragedThermodynamics, 0.2),
                    (ArwWorkspaceVolumeField::AverageWestEastMassFlux, 0.2),
                    (ArwWorkspaceVolumeField::AverageSouthNorthMassFlux, 0.2),
                    (
                        ArwWorkspaceVolumeField::ForwardWestEastMomentumTendency,
                        0.003,
                    ),
                    (
                        ArwWorkspaceVolumeField::ForwardSouthNorthMomentumTendency,
                        0.004,
                    ),
                    (
                        ArwWorkspaceVolumeField::ForwardVerticalMomentumTendency,
                        0.005,
                    ),
                    (ArwWorkspaceVolumeField::ForwardGeopotentialTendency, 0.006),
                    (
                        ArwWorkspaceVolumeField::ForwardPotentialTemperatureTendency,
                        0.007,
                    ),
                ] {
                    set_volume(workspace, field, index, value);
                }
                let tendency_offset = 0.0001 * (i + k + j) as f32;
                for (field, base) in [
                    (ArwWorkspaceVolumeField::WestEastMomentumTendency, 0.010),
                    (ArwWorkspaceVolumeField::SouthNorthMomentumTendency, 0.011),
                    (ArwWorkspaceVolumeField::VerticalMomentumTendency, 0.012),
                    (ArwWorkspaceVolumeField::GeopotentialTendency, 0.013),
                    (ArwWorkspaceVolumeField::PotentialTemperatureTendency, 0.014),
                ] {
                    set_volume(workspace, field, index, base + tendency_offset);
                }
            }
        }
    }

    let horizontal = shape.horizontal_shape();
    for south_north in 0..horizontal.south_north_points() {
        for west_east in 0..horizontal.west_east_points() {
            let index = linear_index(horizontal, west_east, 0, south_north);
            let i = west_east as i32;
            let j = south_north as i32;
            for field in [
                ArwWorkspaceColumnField::FullColumnMass,
                ArwWorkspaceColumnField::WestEastColumnMass,
                ArwWorkspaceColumnField::SouthNorthColumnMass,
            ] {
                workspace.column_field_values_mut(field)[index] = SENTINEL;
            }
            workspace.column_field_values_mut(ArwWorkspaceColumnField::ColumnMassTendency)[index] =
                0.01 + 0.001 * (i - j) as f32;
            workspace.column_field_values_mut(ArwWorkspaceColumnField::ForwardColumnMassTendency)
                [index] = 0.02;
            for field in [
                ArwWorkspaceColumnField::FinalWestEastColumnMass,
                ArwWorkspaceColumnField::FinalSouthNorthColumnMass,
                ArwWorkspaceColumnField::FinalFullColumnMass,
                ArwWorkspaceColumnField::SavedPerturbationColumnMass,
            ] {
                workspace.column_field_values_mut(field)[index] = 0.2;
            }
            workspace
                .column_field_values_mut(ArwWorkspaceColumnField::DivergenceDampingColumnMass)
                [index] = 0.0;
            workspace.column_field_values_mut(ArwWorkspaceColumnField::TimeCenteredColumnMass)
                [index] = 0.0;
        }
    }
}

fn set_volume(
    workspace: &mut ArwModelWorkspace,
    field: ArwWorkspaceVolumeField,
    index: usize,
    value: f32,
) {
    workspace.volume_field_values_mut(field)[index] = value;
}

const fn linear_index(
    shape: GridShape,
    west_east: usize,
    bottom_top: usize,
    south_north: usize,
) -> usize {
    (south_north * shape.bottom_top_points() + bottom_top) * shape.west_east_points() + west_east
}
