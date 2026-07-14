use wrf_compute::{CpuBackend, CpuField};

use crate::{
    AcousticTrajectoryCoefficients, AcousticTrajectoryControls, AcousticTrajectoryDiagnostics,
    AcousticTrajectoryInputs, AcousticTrajectoryKernels, AcousticTrajectoryRegions,
    AcousticTrajectoryResult, AcousticTrajectorySavedState, AcousticTrajectoryTimeLevels,
    AcousticTrajectoryWorkspace,
};

mod arguments;
mod execution;
mod preflight;

pub(crate) use execution::AcousticTrajectoryExecution;

pub(crate) fn validate_acoustic_trajectory(
    execution: &mut AcousticTrajectoryExecution<'_, '_, '_, '_>,
) -> AcousticTrajectoryResult<()> {
    preflight::validate(execution)
}

impl AcousticTrajectoryKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn advance_acoustic_trajectory(
        &self,
        time_levels: AcousticTrajectoryTimeLevels<'_, Self::Field>,
        saved_state: AcousticTrajectorySavedState<'_, Self::Field>,
        diagnostics: AcousticTrajectoryDiagnostics<'_, Self::Field>,
        workspace: AcousticTrajectoryWorkspace<'_, Self::Field>,
        inputs: AcousticTrajectoryInputs<'_, Self::Field>,
        coefficients: AcousticTrajectoryCoefficients<'_>,
        controls: AcousticTrajectoryControls,
        regions: AcousticTrajectoryRegions<'_>,
    ) -> AcousticTrajectoryResult<()> {
        let mut execution = AcousticTrajectoryExecution::new(
            self,
            time_levels,
            saved_state,
            diagnostics,
            workspace,
            inputs,
            coefficients,
            controls,
            regions,
        );
        validate_acoustic_trajectory(&mut execution)?;
        execution.run()
    }
}

#[cfg(test)]
mod tests {
    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;
    use crate::{
        AcousticFluxAccumulationRegion, AcousticHorizontalBoundaryPolicy,
        AcousticHorizontalMomentumRegion, AcousticMassThetaBoundaryPolicy,
        AcousticMassThetaLateralDomain, AcousticMassThetaRegion,
        AcousticMassThetaWestEastPeriodicity, AcousticPressureMode, AcousticPressureRegion,
        AcousticRelaxationZone, AcousticSouthNorthBoundary, AcousticStepPreparationPhase,
        AcousticStepPreparationRegion, AcousticTrajectoryError, AcousticTrajectoryMapFactors,
        AcousticTrajectoryMassInputs, AcousticTrajectoryMoistureCoefficients,
        AcousticTrajectoryPressureInputs, AcousticTrajectoryTendencies, AcousticVerticalAdvection,
        AcousticVerticalBoundaryPolicy, AcousticVerticalDamping, AcousticVerticalLateralDomain,
        AcousticVerticalRegion, AcousticVerticalWestEastPeriodicity, AcousticWestEastBoundary,
        AcousticWestEastPeriodicity, VerticalAcousticCoefficientRegion,
        VerticalAcousticTopBoundary,
    };

    const VOLUME_FIELD_COUNT: usize = 29;
    const HORIZONTAL_FIELD_COUNT: usize = 8;

    #[test]
    fn one_and_four_workers_produce_identical_complete_trajectory_bits() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();
        let mut one_worker_fixture = TrajectoryFixture::new(&one_worker);
        let mut four_worker_fixture = TrajectoryFixture::new(&four_workers);

        one_worker_fixture.execute(&one_worker).unwrap();
        four_worker_fixture.execute(&four_workers).unwrap();

        assert_eq!(
            one_worker_fixture.mutable_bits(),
            four_worker_fixture.mutable_bits()
        );
    }

    #[test]
    fn matches_the_direct_pinned_fortran_trajectory_bits() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let mut fixture = TrajectoryFixture::new(&backend);

        fixture.execute(&backend).unwrap();

        let expected = include_str!("../../../test-data/acoustic_trajectory.out.correct")
            .lines()
            .map(|line| u32::from_str_radix(line.split_whitespace().last().unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        let actual = [1, 3, 5, 7, 9]
            .into_iter()
            .flat_map(|index| field_bits(&fixture.volume_fields[index]))
            .chain(field_bits(&fixture.horizontal_fields[1]))
            .chain(
                [18, 19, 25, 26, 27]
                    .into_iter()
                    .flat_map(|index| field_bits(&fixture.volume_fields[index])),
            )
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn late_flux_validation_failure_preserves_every_mutable_field() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut fixture = TrajectoryFixture::new(&backend);
        fixture.volume_fields[27] = backend
            .create_field(GridShape::try_new(6, 6, 1).unwrap(), -17.0)
            .unwrap();
        let before = fixture.mutable_bits();

        let result = fixture.execute(&backend);

        assert!(matches!(
            result,
            Err(AcousticTrajectoryError::FluxAccumulation(_))
        ));
        assert_eq!(fixture.mutable_bits(), before);
    }

    #[derive(Clone)]
    struct TrajectoryFixture {
        volume_fields: [CpuField<f32>; VOLUME_FIELD_COUNT],
        horizontal_fields: [CpuField<f32>; HORIZONTAL_FIELD_COUNT],
        volume_inputs: [CpuField<f32>; 12],
        horizontal_inputs: [CpuField<f32>; 13],
        coefficients: [Vec<f32>; 10],
    }

    impl TrajectoryFixture {
        fn new(backend: &CpuBackend) -> Self {
            let shape = GridShape::try_new(6, 6, 6).unwrap();
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
                    vec![0.60; 6],
                    vec![0.40; 6],
                    vec![0.55; 6],
                    vec![0.45; 6],
                    vec![1.00; 6],
                    vec![0.20; 6],
                    vec![1.00; 6],
                    vec![1.00; 6],
                    vec![0.60; 6],
                    vec![0.40; 6],
                ],
            }
        }

        fn execute(&mut self, backend: &CpuBackend) -> AcousticTrajectoryResult<()> {
            let shape = GridShape::try_new(6, 6, 6).unwrap();
            let preparation_region =
                AcousticStepPreparationRegion::try_new(shape, 1..5, 1..5, 1..5, 1..6, 1..6, 1..6)
                    .unwrap();
            let pressure_region =
                AcousticPressureRegion::try_new(shape, 1..5, 1..5, 1..5, 1..5, 1..5, 1..5).unwrap();
            let vertical_coefficient_region =
                VerticalAcousticCoefficientRegion::try_new(shape, 1..5, 1..5, 1..5, 1..5, 1..5)
                    .unwrap();
            let horizontal_momentum_region = AcousticHorizontalMomentumRegion::try_new(
                shape,
                1..5,
                1..5,
                1..5,
                1..6,
                1..6,
                1..6,
            )
            .unwrap();
            let mass_theta_region =
                AcousticMassThetaRegion::try_new(shape, 1..5, 1..5, 1..5, 1..5, 1..5, 1..6)
                    .unwrap();
            let vertical_momentum_region =
                AcousticVerticalRegion::try_new(shape, 1..5, 1..5, 1..5, 1..5, 1..5, 1..6).unwrap();
            let flux_accumulation_region =
                AcousticFluxAccumulationRegion::try_new(shape, 1..5, 1..5, 1..5, 1..6, 1..6, 1..6)
                    .unwrap();
            let controls = controls();
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

            backend.advance_acoustic_trajectory(
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
                controls,
                AcousticTrajectoryRegions::new(
                    &preparation_region,
                    &pressure_region,
                    &vertical_coefficient_region,
                    &horizontal_momentum_region,
                    &mass_theta_region,
                    &vertical_momentum_region,
                    &flux_accumulation_region,
                ),
            )
        }

        fn mutable_bits(&self) -> Vec<u32> {
            self.volume_fields
                .iter()
                .chain(&self.horizontal_fields)
                .flat_map(|field| field.values().iter().map(|value| value.to_bits()))
                .collect()
        }
    }

    fn controls() -> AcousticTrajectoryControls {
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

    fn field_bits(field: &CpuField<f32>) -> impl Iterator<Item = u32> + '_ {
        field.values().iter().map(|value| value.to_bits())
    }
}
