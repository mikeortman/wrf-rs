#![allow(missing_docs)]

use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_dynamics::{
    AcousticBoundaryStageControls, AcousticBoundaryStageKernels, AcousticBoundaryStageRegions,
    AcousticFluxAccumulationRegion, AcousticHorizontalBoundaryPolicy,
    AcousticHorizontalMomentumRegion, AcousticMassThetaBoundaryPolicy,
    AcousticMassThetaLateralDomain, AcousticMassThetaRegion, AcousticMassThetaWestEastPeriodicity,
    AcousticPressureMode, AcousticPressureRegion, AcousticRelaxationZone,
    AcousticSouthNorthBoundary, AcousticStepPreparationPhase, AcousticStepPreparationRegion,
    AcousticTrajectoryCoefficients, AcousticTrajectoryControls, AcousticTrajectoryDiagnostics,
    AcousticTrajectoryInputs, AcousticTrajectoryMapFactors, AcousticTrajectoryMassInputs,
    AcousticTrajectoryMoistureCoefficients, AcousticTrajectoryPressureInputs,
    AcousticTrajectoryRegions, AcousticTrajectorySavedState, AcousticTrajectoryTendencies,
    AcousticTrajectoryTimeLevels, AcousticTrajectoryWorkspace, AcousticVerticalAdvection,
    AcousticVerticalBoundaryPolicy, AcousticVerticalDamping, AcousticVerticalLateralDomain,
    AcousticVerticalRegion, AcousticVerticalWestEastPeriodicity, AcousticWestEastBoundary,
    AcousticWestEastPeriodicity, PhysicalBoundaryConditions, PhysicalBoundaryRegion,
    SpecifiedBoundaryFieldLocation, SpecifiedBoundaryUpdateRegion,
    VerticalAcousticCoefficientRegion, VerticalAcousticTopBoundary,
};

const VOLUME_FIELD_COUNT: usize = 29;
const HORIZONTAL_FIELD_COUNT: usize = 8;

pub struct AcousticBoundaryStageFixture {
    volume_fields: [CpuField<f32>; VOLUME_FIELD_COUNT],
    horizontal_fields: [CpuField<f32>; HORIZONTAL_FIELD_COUNT],
    volume_inputs: [CpuField<f32>; 12],
    horizontal_inputs: [CpuField<f32>; 13],
    coefficients: [Vec<f32>; 10],
    regions: FixtureRegions,
}

impl AcousticBoundaryStageFixture {
    pub fn new(
        backend: &CpuBackend,
        active_west_east_points: usize,
        active_south_north_points: usize,
        half_levels: usize,
    ) -> Self {
        let shape = GridShape::try_new(
            active_west_east_points + 9,
            active_south_north_points + 9,
            half_levels + 2,
        )
        .unwrap();
        let horizontal_shape = shape.horizontal_shape();
        let volume = |value| backend.create_field(shape, value).unwrap();
        let horizontal = |value| backend.create_field(horizontal_shape, value).unwrap();
        let mut volume_fields = std::array::from_fn(|_| volume(0.2));
        volume_fields[6] = volume(300.0);
        volume_fields[7] = volume(300.0);
        volume_fields[8] = volume(10.0);
        volume_fields[9] = volume(10.0);
        let mut horizontal_fields = std::array::from_fn(|_| horizontal(0.0));
        horizontal_fields[0] = horizontal(1.0);
        horizontal_fields[1] = horizontal(1.0);
        Self {
            volume_fields,
            horizontal_fields,
            volume_inputs: [
                volume(80_000.0),
                volume(1.0),
                volume(10.0),
                volume(1_000.0),
                volume(0.01),
                volume(0.01),
                volume(0.01),
                volume(0.01),
                volume(0.01),
                volume(1.0),
                volume(1.0),
                volume(1.0),
            ],
            horizontal_inputs: [
                horizontal(10.0),
                horizontal(10.0),
                horizontal(10.0),
                horizontal(10.0),
                horizontal(0.01),
                horizontal(1.0),
                horizontal(1.0),
                horizontal(1.0),
                horizontal(1.0),
                horizontal(1.0),
                horizontal(1.0),
                horizontal(1.0),
                horizontal(0.0),
            ],
            coefficients: [
                vec![0.60; shape.bottom_top_points()],
                vec![0.40; shape.bottom_top_points()],
                vec![0.55; shape.bottom_top_points()],
                vec![0.45; shape.bottom_top_points()],
                vec![1.00; shape.bottom_top_points()],
                vec![0.20; shape.bottom_top_points()],
                vec![1.00; shape.bottom_top_points()],
                vec![1.00; shape.bottom_top_points()],
                vec![0.60; shape.bottom_top_points()],
                vec![0.40; shape.bottom_top_points()],
            ],
            regions: FixtureRegions::new(
                shape,
                active_west_east_points,
                active_south_north_points,
                half_levels,
            ),
        }
    }

    pub fn apply(&mut self, backend: &CpuBackend) {
        let [
            previous_west_east_momentum,
            current_west_east_momentum,
            previous_south_north_momentum,
            current_south_north_momentum,
            previous_vertical_momentum,
            current_vertical_momentum,
            previous_potential_temperature,
            current_potential_temperature,
            previous_perturbation_geopotential,
            current_perturbation_geopotential,
            saved_west_east_momentum,
            saved_south_north_momentum,
            saved_vertical_momentum,
            saved_potential_temperature,
            saved_perturbation_geopotential,
            saved_vertical_mass_flux,
            saved_pressure_coefficient,
            vertical_mass_flux,
            inverse_density_perturbation,
            pressure_perturbation,
            previous_pressure_perturbation,
            lower_diagonal,
            inverse_eliminated_diagonal,
            upper_elimination_factor,
            time_averaged_thermodynamics,
            average_west_east_mass_flux,
            average_south_north_mass_flux,
            average_vertical_mass_flux,
            geopotential_right_hand_side,
        ] = &mut self.volume_fields;
        let [
            previous_perturbation_column_mass,
            current_perturbation_column_mass,
            saved_perturbation_column_mass,
            saved_west_east_column_mass,
            saved_south_north_column_mass,
            coupled_column_mass,
            divergence_damping_column_mass,
            time_centered_column_mass,
        ] = &mut self.horizontal_fields;
        let [
            base_pressure,
            full_inverse_density,
            pressure_point_geopotential,
            base_geopotential,
            west_east_momentum_tendency,
            south_north_momentum_tendency,
            vertical_momentum_tendency,
            potential_temperature_tendency,
            perturbation_geopotential_tendency,
            west_east_moisture,
            south_north_moisture,
            vertical_moisture,
        ] = &mut self.volume_inputs;
        let [
            base_column_mass,
            west_east_column_mass,
            south_north_column_mass,
            full_column_mass,
            column_mass_tendency,
            west_east_x_map,
            west_east_y_map,
            south_north_x_map,
            inverse_south_north_x_map,
            south_north_y_map,
            mass_point_x_map,
            mass_point_y_map,
            terrain_height,
        ] = &self.horizontal_inputs;
        let [
            half_level_mass_multiplier,
            half_level_mass_offset,
            full_level_mass_multiplier,
            full_level_mass_offset,
            hydrostatic_pressure_multiplier,
            half_level_eta_thickness,
            inverse_half_level_spacing,
            inverse_full_level_spacing,
            upper_full_level_weight,
            lower_full_level_weight,
        ] = &self.coefficients;

        backend
            .advance_acoustic_boundary_stage(
                AcousticTrajectoryTimeLevels::new(
                    previous_west_east_momentum,
                    current_west_east_momentum,
                    previous_south_north_momentum,
                    current_south_north_momentum,
                    previous_vertical_momentum,
                    current_vertical_momentum,
                    previous_potential_temperature,
                    current_potential_temperature,
                    previous_perturbation_geopotential,
                    current_perturbation_geopotential,
                    previous_perturbation_column_mass,
                    current_perturbation_column_mass,
                ),
                AcousticTrajectorySavedState::new(
                    saved_west_east_momentum,
                    saved_south_north_momentum,
                    saved_vertical_momentum,
                    saved_potential_temperature,
                    saved_perturbation_geopotential,
                    saved_perturbation_column_mass,
                    saved_vertical_mass_flux,
                    saved_pressure_coefficient,
                ),
                AcousticTrajectoryDiagnostics::new(
                    vertical_mass_flux,
                    saved_west_east_column_mass,
                    saved_south_north_column_mass,
                    coupled_column_mass,
                    divergence_damping_column_mass,
                    inverse_density_perturbation,
                    pressure_perturbation,
                    previous_pressure_perturbation,
                    lower_diagonal,
                    inverse_eliminated_diagonal,
                    upper_elimination_factor,
                    time_centered_column_mass,
                    time_averaged_thermodynamics,
                    average_west_east_mass_flux,
                    average_south_north_mass_flux,
                    average_vertical_mass_flux,
                ),
                AcousticTrajectoryWorkspace::new(geopotential_right_hand_side),
                AcousticTrajectoryInputs::new(
                    AcousticTrajectoryMassInputs::new(
                        base_column_mass,
                        west_east_column_mass,
                        south_north_column_mass,
                        full_column_mass,
                        column_mass_tendency,
                    ),
                    AcousticTrajectoryPressureInputs::new(
                        base_pressure,
                        full_inverse_density,
                        pressure_point_geopotential,
                        base_geopotential,
                    ),
                    AcousticTrajectoryTendencies::new(
                        west_east_momentum_tendency,
                        south_north_momentum_tendency,
                        vertical_momentum_tendency,
                        potential_temperature_tendency,
                        perturbation_geopotential_tendency,
                    ),
                    AcousticTrajectoryMoistureCoefficients::new(
                        west_east_moisture,
                        south_north_moisture,
                        vertical_moisture,
                    ),
                    AcousticTrajectoryMapFactors::new(
                        west_east_x_map,
                        west_east_y_map,
                        south_north_x_map,
                        inverse_south_north_x_map,
                        south_north_y_map,
                        mass_point_x_map,
                        mass_point_y_map,
                        terrain_height,
                    ),
                ),
                AcousticTrajectoryCoefficients::new(
                    half_level_mass_multiplier,
                    half_level_mass_offset,
                    full_level_mass_multiplier,
                    full_level_mass_offset,
                    hydrostatic_pressure_multiplier,
                    half_level_eta_thickness,
                    inverse_half_level_spacing,
                    inverse_full_level_spacing,
                    upper_full_level_weight,
                    lower_full_level_weight,
                ),
                AcousticBoundaryStageControls::new(
                    trajectory_controls(),
                    PhysicalBoundaryConditions::specified_lateral(),
                    2,
                ),
                self.regions.as_stage_regions(),
            )
            .unwrap();
    }

    pub fn reset(&mut self) {
        for field in &mut self.volume_fields {
            field.values_mut().fill(0.2);
        }
        self.volume_fields[6].values_mut().fill(300.0);
        self.volume_fields[7].values_mut().fill(300.0);
        self.volume_fields[8].values_mut().fill(10.0);
        self.volume_fields[9].values_mut().fill(10.0);
        for field in &mut self.horizontal_fields {
            field.values_mut().fill(0.0);
        }
        self.horizontal_fields[0].values_mut().fill(1.0);
        self.horizontal_fields[1].values_mut().fill(1.0);
        self.volume_inputs[4].values_mut().fill(0.01);
        self.volume_inputs[5].values_mut().fill(0.01);
    }
}

struct FixtureRegions {
    preparation: AcousticStepPreparationRegion,
    pressure: AcousticPressureRegion,
    vertical_coefficient: VerticalAcousticCoefficientRegion,
    horizontal_momentum: AcousticHorizontalMomentumRegion,
    mass_theta: AcousticMassThetaRegion,
    vertical_momentum: AcousticVerticalRegion,
    flux_accumulation: AcousticFluxAccumulationRegion,
    physical: PhysicalBoundaryRegion,
    west_east: SpecifiedBoundaryUpdateRegion,
    south_north: SpecifiedBoundaryUpdateRegion,
    potential_temperature: SpecifiedBoundaryUpdateRegion,
    column_mass: SpecifiedBoundaryUpdateRegion,
    full_level: SpecifiedBoundaryUpdateRegion,
}

impl FixtureRegions {
    fn new(shape: GridShape, west_east: usize, south_north: usize, levels: usize) -> Self {
        let mass_west_east = 4..west_east + 4;
        let mass_south_north = 4..south_north + 4;
        let half_level = 1..levels + 1;
        let tile_west_east = 4..west_east + 5;
        let tile_south_north = 4..south_north + 5;
        let tile_bottom_top = 1..levels + 2;
        Self {
            preparation: AcousticStepPreparationRegion::try_new(
                shape,
                mass_west_east.clone(),
                mass_south_north.clone(),
                half_level.clone(),
                tile_west_east.clone(),
                tile_south_north.clone(),
                tile_bottom_top.clone(),
            )
            .unwrap(),
            pressure: AcousticPressureRegion::try_new(
                shape,
                mass_west_east.clone(),
                mass_south_north.clone(),
                half_level.clone(),
                mass_west_east.clone(),
                mass_south_north.clone(),
                half_level.clone(),
            )
            .unwrap(),
            vertical_coefficient: VerticalAcousticCoefficientRegion::try_new(
                shape,
                mass_west_east.clone(),
                mass_south_north.clone(),
                half_level.clone(),
                mass_west_east.clone(),
                mass_south_north.clone(),
            )
            .unwrap(),
            horizontal_momentum: AcousticHorizontalMomentumRegion::try_new(
                shape,
                mass_west_east.clone(),
                mass_south_north.clone(),
                half_level.clone(),
                tile_west_east.clone(),
                tile_south_north.clone(),
                tile_bottom_top.clone(),
            )
            .unwrap(),
            mass_theta: AcousticMassThetaRegion::try_new(
                shape,
                mass_west_east.clone(),
                mass_south_north.clone(),
                half_level.clone(),
                mass_west_east.clone(),
                mass_south_north.clone(),
                tile_bottom_top.clone(),
            )
            .unwrap(),
            vertical_momentum: AcousticVerticalRegion::try_new(
                shape,
                mass_west_east.clone(),
                mass_south_north.clone(),
                half_level.clone(),
                mass_west_east.clone(),
                mass_south_north.clone(),
                tile_bottom_top.clone(),
            )
            .unwrap(),
            flux_accumulation: AcousticFluxAccumulationRegion::try_new(
                shape,
                mass_west_east.clone(),
                mass_south_north.clone(),
                half_level.clone(),
                tile_west_east.clone(),
                tile_south_north.clone(),
                tile_bottom_top.clone(),
            )
            .unwrap(),
            physical: PhysicalBoundaryRegion::try_new(
                shape,
                mass_west_east.clone(),
                mass_south_north.clone(),
                half_level.clone(),
                tile_west_east.clone(),
                tile_south_north.clone(),
                tile_bottom_top.clone(),
            )
            .unwrap(),
            west_east: specified_region(
                shape,
                SpecifiedBoundaryFieldLocation::WestEastFace,
                mass_west_east.clone(),
                mass_south_north.clone(),
                half_level.clone(),
                tile_west_east.clone(),
                tile_south_north.clone(),
                tile_bottom_top.clone(),
            ),
            south_north: specified_region(
                shape,
                SpecifiedBoundaryFieldLocation::SouthNorthFace,
                mass_west_east.clone(),
                mass_south_north.clone(),
                half_level.clone(),
                tile_west_east.clone(),
                tile_south_north.clone(),
                tile_bottom_top.clone(),
            ),
            potential_temperature: specified_region(
                shape,
                SpecifiedBoundaryFieldLocation::MassHalfLevel,
                mass_west_east.clone(),
                mass_south_north.clone(),
                half_level.clone(),
                tile_west_east.clone(),
                tile_south_north.clone(),
                half_level.clone(),
            ),
            column_mass: specified_region(
                shape.horizontal_shape(),
                SpecifiedBoundaryFieldLocation::HorizontalMass,
                mass_west_east.clone(),
                mass_south_north.clone(),
                0..1,
                tile_west_east.clone(),
                tile_south_north.clone(),
                0..1,
            ),
            full_level: specified_region(
                shape,
                SpecifiedBoundaryFieldLocation::FullLevel,
                mass_west_east,
                mass_south_north,
                half_level,
                tile_west_east,
                tile_south_north,
                tile_bottom_top,
            ),
        }
    }

    fn as_stage_regions(&self) -> AcousticBoundaryStageRegions<'_> {
        AcousticBoundaryStageRegions::new(
            AcousticTrajectoryRegions::new(
                &self.preparation,
                &self.pressure,
                &self.vertical_coefficient,
                &self.horizontal_momentum,
                &self.mass_theta,
                &self.vertical_momentum,
                &self.flux_accumulation,
            ),
            &self.physical,
            &self.west_east,
            &self.south_north,
            &self.potential_temperature,
            &self.column_mass,
            &self.full_level,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn specified_region(
    shape: GridShape,
    location: SpecifiedBoundaryFieldLocation,
    mass_west_east: std::ops::Range<usize>,
    mass_south_north: std::ops::Range<usize>,
    half_level: std::ops::Range<usize>,
    tile_west_east: std::ops::Range<usize>,
    tile_south_north: std::ops::Range<usize>,
    tile_bottom_top: std::ops::Range<usize>,
) -> SpecifiedBoundaryUpdateRegion {
    SpecifiedBoundaryUpdateRegion::try_new(
        shape,
        location,
        mass_west_east,
        mass_south_north,
        half_level,
        tile_west_east,
        tile_south_north,
        tile_bottom_top,
    )
    .unwrap()
}

fn trajectory_controls() -> AcousticTrajectoryControls {
    AcousticTrajectoryControls::try_new(
        AcousticStepPreparationPhase::FirstSubstep,
        3,
        0.1,
        0.1,
        0.01,
        9.81,
        300.0,
        0.1,
        0.0,
        0.0,
        [0.5, 0.3, 0.2],
        AcousticPressureMode::Nonhydrostatic,
        VerticalAcousticTopBoundary::Nonrigid,
        AcousticHorizontalBoundaryPolicy::new(
            AcousticRelaxationZone::Disabled,
            AcousticWestEastPeriodicity::Nonperiodic,
            AcousticWestEastBoundary::Closed,
            AcousticWestEastBoundary::Closed,
            AcousticSouthNorthBoundary::Closed,
            AcousticSouthNorthBoundary::Closed,
        ),
        AcousticMassThetaBoundaryPolicy::new(
            AcousticMassThetaLateralDomain::Global,
            AcousticMassThetaWestEastPeriodicity::Nonperiodic,
        ),
        AcousticVerticalBoundaryPolicy::new(
            AcousticVerticalLateralDomain::Global,
            AcousticVerticalWestEastPeriodicity::Nonperiodic,
        ),
        AcousticVerticalAdvection::StaggeredGeopotentialGradient,
        AcousticVerticalDamping::Disabled,
    )
    .unwrap()
}
