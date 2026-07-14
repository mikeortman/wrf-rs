use wrf_compute::{ComputeBackend, CpuBackend, CpuField, FieldStorage, GridShape};
use wrf_physics::{
    ArwMicrophysicsControls, ArwMicrophysicsState, ArwMicrophysicsTrajectory,
    ArwMicrophysicsWorkspace, MicrophysicsBoundaryPolicy, MicrophysicsDriverDomain,
    MicrophysicsTile,
};
use wrf_registry::{RegistryParser, RuntimeConfigurationChoice};

const WEST_EAST_POINTS: usize = 128;
const SOUTH_NORTH_POINTS: usize = 128;
const BOTTOM_TOP_POINTS: usize = 40;
const TRAJECTORY_STEPS: usize = 3;
const GRAVITY: f32 = 9.81;

pub struct KesslerPrecipitationTrajectoryFixture {
    trajectory: ArwMicrophysicsTrajectory,
    tiles: [MicrophysicsTile; 1],
    workspace: ArwMicrophysicsWorkspace,
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
    initial_potential_temperature: Vec<f32>,
    initial_moisture_species_values: Vec<Vec<f32>>,
    initial_accumulated_precipitation: Vec<f32>,
}

impl KesslerPrecipitationTrajectoryFixture {
    pub fn new(backend: &CpuBackend) -> Self {
        let mass_shape =
            GridShape::try_new(WEST_EAST_POINTS, SOUTH_NORTH_POINTS, BOTTOM_TOP_POINTS).unwrap();
        let w_shape =
            GridShape::try_new(WEST_EAST_POINTS, SOUTH_NORTH_POINTS, BOTTOM_TOP_POINTS + 1)
                .unwrap();
        let domain = MicrophysicsDriverDomain::try_new(
            mass_shape,
            0..WEST_EAST_POINTS,
            0..SOUTH_NORTH_POINTS,
            0..BOTTOM_TOP_POINTS,
            MicrophysicsBoundaryPolicy::open(),
        )
        .unwrap();
        let layout = kessler_layout();
        let trajectory = ArwMicrophysicsTrajectory::try_kessler(
            domain,
            &layout,
            ArwMicrophysicsControls::try_from_wrf_defaults(60.0).unwrap(),
        )
        .unwrap();
        let workspace = trajectory.create_workspace(backend).unwrap();
        let mut fixture = Self {
            trajectory,
            tiles: [MicrophysicsTile::new(
                0..WEST_EAST_POINTS,
                0..SOUTH_NORTH_POINTS,
            )],
            workspace,
            perturbation_potential_temperature: backend.create_field(mass_shape, 0.0).unwrap(),
            moisture_species_fields: vec![
                backend.create_field(mass_shape, 0.0).unwrap(),
                backend.create_field(mass_shape, 0.0).unwrap(),
                backend.create_field(mass_shape, 0.0).unwrap(),
            ],
            perturbation_inverse_density: backend.create_field(mass_shape, 0.0).unwrap(),
            base_inverse_density: backend.create_field(mass_shape, 0.0).unwrap(),
            perturbation_pressure: backend.create_field(mass_shape, 0.0).unwrap(),
            base_pressure: backend.create_field(mass_shape, 0.0).unwrap(),
            perturbation_geopotential: backend.create_field(w_shape, 0.0).unwrap(),
            base_geopotential: backend.create_field(w_shape, 0.0).unwrap(),
            accumulated_precipitation: backend
                .create_field(mass_shape.horizontal_shape(), 10.0)
                .unwrap(),
            step_precipitation: backend
                .create_field(mass_shape.horizontal_shape(), 0.0)
                .unwrap(),
            initial_potential_temperature: Vec::new(),
            initial_moisture_species_values: Vec::new(),
            initial_accumulated_precipitation: Vec::new(),
        };
        fixture.initialize_fields();
        fixture.capture_initial_values();
        fixture
    }

    #[allow(dead_code)]
    pub const fn updated_value_count() -> usize {
        WEST_EAST_POINTS * SOUTH_NORTH_POINTS * BOTTOM_TOP_POINTS * TRAJECTORY_STEPS
    }

    pub fn reset(&mut self) {
        self.perturbation_potential_temperature
            .values_mut()
            .copy_from_slice(&self.initial_potential_temperature);
        for (field, initial_values) in self
            .moisture_species_fields
            .iter_mut()
            .zip(&self.initial_moisture_species_values)
        {
            field.values_mut().copy_from_slice(initial_values);
        }
        self.accumulated_precipitation
            .values_mut()
            .copy_from_slice(&self.initial_accumulated_precipitation);
        self.step_precipitation.values_mut().fill(0.0);
    }

    pub fn apply_three_steps(&mut self, backend: &CpuBackend) {
        for _ in 0..TRAJECTORY_STEPS {
            self.trajectory
                .apply_step(
                    backend,
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
                    ),
                    &self.tiles,
                    &mut self.workspace,
                )
                .unwrap();
        }
    }

    fn initialize_fields(&mut self) {
        let mass_shape = self.perturbation_potential_temperature.shape();
        for south_north_index in 0..SOUTH_NORTH_POINTS {
            for bottom_top_index in 0..BOTTOM_TOP_POINTS {
                for west_east_index in 0..WEST_EAST_POINTS {
                    let index = linear_index(
                        mass_shape,
                        west_east_index,
                        bottom_top_index,
                        south_north_index,
                    );
                    self.perturbation_potential_temperature.values_mut()[index] =
                        -21.0 + 0.007 * west_east_index as f32 + 0.03 * bottom_top_index as f32
                            - 0.004 * south_north_index as f32;
                    self.moisture_species_fields[0].values_mut()[index] =
                        0.002 + 0.001 * ((west_east_index + 2 * bottom_top_index) % 8) as f32;
                    self.moisture_species_fields[1].values_mut()[index] =
                        if (west_east_index + bottom_top_index) % 3 == 0 {
                            0.002
                        } else {
                            0.0002
                        };
                    self.moisture_species_fields[2].values_mut()[index] =
                        [0.0, 0.0005, 0.005, 0.02][(west_east_index + south_north_index) % 4];
                    self.perturbation_inverse_density.values_mut()[index] =
                        0.83 + 0.006 * bottom_top_index as f32;
                    self.base_inverse_density.values_mut()[index] = 0.02;
                    self.perturbation_pressure.values_mut()[index] =
                        -500.0 * bottom_top_index as f32;
                    self.base_pressure.values_mut()[index] =
                        100_000.0 - 1_500.0 * bottom_top_index as f32;
                }
            }
        }

        let w_shape = self.perturbation_geopotential.shape();
        for south_north_index in 0..SOUTH_NORTH_POINTS {
            for bottom_top_index in 0..=BOTTOM_TOP_POINTS {
                for west_east_index in 0..WEST_EAST_POINTS {
                    let index = linear_index(
                        w_shape,
                        west_east_index,
                        bottom_top_index,
                        south_north_index,
                    );
                    self.perturbation_geopotential.values_mut()[index] =
                        GRAVITY * (50.0 + 150.0 * bottom_top_index as f32);
                }
            }
        }
    }

    fn capture_initial_values(&mut self) {
        self.initial_potential_temperature =
            self.perturbation_potential_temperature.values().to_vec();
        self.initial_moisture_species_values = self
            .moisture_species_fields
            .iter()
            .map(|field| field.values().to_vec())
            .collect();
        self.initial_accumulated_precipitation = self.accumulated_precipitation.values().to_vec();
    }
}

fn kessler_layout() -> wrf_registry::ResolvedScalarArrayLayout {
    let source = "\
dimspec i 1 standard_domain x west_east
    dimspec k 2 standard_domain z bottom_top
    dimspec j 3 standard_domain y south_north
    rconfig integer mp_physics namelist,physics 1 -1 - mp_physics \"\" \"\"
    state real - ikjftb moist 1 - - - - -
    state real qv ikjftb moist 1 - - QVAPOR vapor 1
    state real qc ikjftb moist 1 - - QCLOUD cloud 1
    state real qr ikjftb moist 1 - - QRAIN rain 1
    package kesslerscheme mp_physics==1 - moist:qv,qc,qr
    ";
    let document = RegistryParser::parse("Registry.benchmark", source).unwrap();
    document
        .resolve_scalar_array_layouts(&[RuntimeConfigurationChoice::new("mp_physics", 1)])
        .unwrap()
        .remove(0)
}

const fn linear_index(
    shape: GridShape,
    west_east_index: usize,
    bottom_top_index: usize,
    south_north_index: usize,
) -> usize {
    (south_north_index * shape.bottom_top_points() + bottom_top_index) * shape.west_east_points()
        + west_east_index
}
