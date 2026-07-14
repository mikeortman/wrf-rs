use wrf_compute::{CpuBackend, CpuField};

use crate::acoustic_trajectory::AcousticTrajectoryExecution;
use crate::{
    AcousticBoundaryStageControls, AcousticBoundaryStageKernels, AcousticBoundaryStageRegions,
    AcousticBoundaryStageResult, AcousticTrajectoryCoefficients, AcousticTrajectoryDiagnostics,
    AcousticTrajectoryInputs, AcousticTrajectorySavedState, AcousticTrajectoryTimeLevels,
    AcousticTrajectoryWorkspace,
};

mod execution;
mod preflight;

use execution::AcousticBoundaryStageExecution;

impl AcousticBoundaryStageKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn advance_acoustic_boundary_stage(
        &self,
        time_levels: AcousticTrajectoryTimeLevels<'_, Self::Field>,
        saved_state: AcousticTrajectorySavedState<'_, Self::Field>,
        diagnostics: AcousticTrajectoryDiagnostics<'_, Self::Field>,
        workspace: AcousticTrajectoryWorkspace<'_, Self::Field>,
        inputs: AcousticTrajectoryInputs<'_, Self::Field>,
        coefficients: AcousticTrajectoryCoefficients<'_>,
        controls: AcousticBoundaryStageControls,
        regions: AcousticBoundaryStageRegions<'_>,
    ) -> AcousticBoundaryStageResult<()> {
        let trajectory = AcousticTrajectoryExecution::new(
            self,
            time_levels,
            saved_state,
            diagnostics,
            workspace,
            inputs,
            coefficients,
            controls.trajectory_controls(),
            regions.trajectory,
        );
        let mut execution = AcousticBoundaryStageExecution::new(trajectory, controls, regions);
        preflight::validate(&mut execution)?;
        execution.run()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::ops::Range;

    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;
    use crate::{
        AcousticBoundaryStageError, AcousticFluxAccumulationRegion,
        AcousticHorizontalBoundaryPolicy, AcousticHorizontalMomentumRegion,
        AcousticMassThetaBoundaryPolicy, AcousticMassThetaLateralDomain, AcousticMassThetaRegion,
        AcousticMassThetaWestEastPeriodicity, AcousticPressureMode, AcousticPressureRegion,
        AcousticRelaxationZone, AcousticSouthNorthBoundary, AcousticStepPreparationPhase,
        AcousticStepPreparationRegion, AcousticTrajectoryControls, AcousticTrajectoryMapFactors,
        AcousticTrajectoryMassInputs, AcousticTrajectoryMoistureCoefficients,
        AcousticTrajectoryPressureInputs, AcousticTrajectoryRegions, AcousticTrajectoryTendencies,
        AcousticVerticalAdvection, AcousticVerticalBoundaryPolicy, AcousticVerticalDamping,
        AcousticVerticalLateralDomain, AcousticVerticalRegion, AcousticVerticalWestEastPeriodicity,
        AcousticWestEastBoundary, AcousticWestEastPeriodicity, PhysicalBoundaryConditions,
        PhysicalBoundaryRegion, SpecifiedBoundaryFieldLocation, SpecifiedBoundaryUpdateRegion,
        VerticalAcousticCoefficientRegion, VerticalAcousticTopBoundary,
    };

    const VOLUME_FIELD_COUNT: usize = 29;
    const HORIZONTAL_FIELD_COUNT: usize = 8;

    #[derive(Clone, Debug)]
    struct StageOracleCase {
        name: &'static str,
        conditions: PhysicalBoundaryConditions,
        west_east_tile: Range<usize>,
        south_north_tile: Range<usize>,
        injects_ieee_values: bool,
    }

    #[test]
    fn complete_stage_matches_direct_pinned_fortran_bits_for_every_oracle_case() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        for case in oracle_cases() {
            let mut fixture = StageFixture::new(&backend);

            fixture.execute_oracle_case(&backend, &case).unwrap();

            for (field_name, field_index) in [
                ("u2", 1),
                ("v2", 3),
                ("w2", 5),
                ("t2", 7),
                ("ph2", 9),
                ("al", 18),
                ("p", 19),
                ("ru_m", 25),
                ("rv_m", 26),
                ("ww_m", 27),
            ] {
                assert_field_matches_oracle(
                    case.name,
                    field_name,
                    fixture.volume_fields[field_index].values(),
                );
            }
            for (field_name, field_index) in [("mu2", 1), ("muts", 5), ("mudf", 6)] {
                assert_field_matches_oracle(
                    case.name,
                    field_name,
                    fixture.horizontal_fields[field_index].values(),
                );
            }
        }
    }

    #[test]
    fn oracle_fixture_discriminates_roles_and_coordinates() {
        let backend = CpuBackend::try_with_worker_count(1).unwrap();
        let fixture = StageFixture::new(&backend);
        let volume_sample = volume_index(5, 2, 5);
        let horizontal_sample = horizontal_index(5, 5);
        let volume_role_bits = fixture
            .volume_fields
            .iter()
            .chain(&fixture.volume_inputs)
            .map(|field| field.values()[volume_sample].to_bits())
            .collect::<HashSet<_>>();
        let horizontal_role_bits = fixture
            .horizontal_fields
            .iter()
            .chain(&fixture.horizontal_inputs)
            .map(|field| field.values()[horizontal_sample].to_bits())
            .collect::<HashSet<_>>();
        let coefficient_role_bits = fixture
            .coefficients
            .iter()
            .map(|coefficient| coefficient[2].to_bits())
            .collect::<HashSet<_>>();

        assert_eq!(volume_role_bits.len(), VOLUME_FIELD_COUNT + 12);
        assert_eq!(horizontal_role_bits.len(), HORIZONTAL_FIELD_COUNT + 13);
        assert_eq!(coefficient_role_bits.len(), 10);
        for field in fixture.volume_fields.iter().chain(&fixture.volume_inputs) {
            assert_ne!(
                field.values()[volume_index(4, 1, 4)].to_bits(),
                field.values()[volume_index(5, 2, 5)].to_bits()
            );
        }
        for field in fixture
            .horizontal_fields
            .iter()
            .chain(&fixture.horizontal_inputs)
        {
            assert_ne!(
                field.values()[horizontal_index(4, 4)].to_bits(),
                field.values()[horizontal_index(5, 5)].to_bits()
            );
        }
    }

    fn oracle_cases() -> [StageOracleCase; 6] {
        [
            StageOracleCase {
                name: "periodic",
                conditions: PhysicalBoundaryConditions::periodic_xy(),
                west_east_tile: 4..11,
                south_north_tile: 4..11,
                injects_ieee_values: false,
            },
            StageOracleCase {
                name: "specified",
                conditions: PhysicalBoundaryConditions::specified_lateral(),
                west_east_tile: 4..11,
                south_north_tile: 4..11,
                injects_ieee_values: false,
            },
            StageOracleCase {
                name: "nested",
                conditions: PhysicalBoundaryConditions::nested_lateral(),
                west_east_tile: 4..11,
                south_north_tile: 4..11,
                injects_ieee_values: false,
            },
            StageOracleCase {
                name: "partial",
                conditions: PhysicalBoundaryConditions::specified_lateral(),
                west_east_tile: 4..7,
                south_north_tile: 5..9,
                injects_ieee_values: false,
            },
            StageOracleCase {
                name: "inactive",
                conditions: PhysicalBoundaryConditions::specified_lateral(),
                west_east_tile: 5..9,
                south_north_tile: 5..9,
                injects_ieee_values: false,
            },
            StageOracleCase {
                name: "ieee",
                conditions: PhysicalBoundaryConditions::specified_lateral(),
                west_east_tile: 4..11,
                south_north_tile: 4..11,
                injects_ieee_values: true,
            },
        ]
    }

    fn initialized_volume(backend: &CpuBackend, base: f32, role: usize) -> CpuField<f32> {
        let shape = stage_shape();
        let mut field = backend.create_field(shape, 0.0).unwrap();
        for south_north in 0..shape.south_north_points() {
            for bottom_top in 0..shape.bottom_top_points() {
                for west_east in 0..shape.west_east_points() {
                    field.values_mut()[volume_index(west_east, bottom_top, south_north)] =
                        volume_pattern(base, role, west_east, bottom_top, south_north);
                }
            }
        }
        field
    }

    fn initialized_horizontal(backend: &CpuBackend, base: f32, role: usize) -> CpuField<f32> {
        let shape = stage_shape().horizontal_shape();
        let mut field = backend.create_field(shape, 0.0).unwrap();
        for south_north in 0..shape.south_north_points() {
            for west_east in 0..shape.west_east_points() {
                field.values_mut()[horizontal_index(west_east, south_north)] =
                    horizontal_pattern(base, role, west_east, south_north);
            }
        }
        field
    }

    fn initialized_coefficient(base: f32, role: usize) -> Vec<f32> {
        (0..stage_shape().bottom_top_points())
            .map(|bottom_top| coefficient_pattern(base, role, bottom_top))
            .collect()
    }

    fn volume_pattern(
        base: f32,
        role: usize,
        west_east: usize,
        bottom_top: usize,
        south_north: usize,
    ) -> f32 {
        base + (role * 64 + west_east + 16 * bottom_top + 128 * south_north) as f32 / 4096.0
    }

    fn horizontal_pattern(base: f32, role: usize, west_east: usize, south_north: usize) -> f32 {
        base + (role * 64 + west_east + 128 * south_north) as f32 / 4096.0
    }

    fn coefficient_pattern(base: f32, role: usize, bottom_top: usize) -> f32 {
        base + (role * 8 + bottom_top) as f32 / 65_536.0
    }

    fn volume_index(west_east: usize, bottom_top: usize, south_north: usize) -> usize {
        let shape = stage_shape();
        south_north * shape.west_east_points() * shape.bottom_top_points()
            + bottom_top * shape.west_east_points()
            + west_east
    }

    fn horizontal_index(west_east: usize, south_north: usize) -> usize {
        south_north * stage_shape().west_east_points() + west_east
    }

    #[test]
    fn every_affected_field_snapshot_matches_direct_pinned_fortran_bits() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        for case in oracle_cases() {
            let mut fixture = StageFixture::new(&backend);
            execution::begin_stage_snapshot_capture();

            fixture.execute_oracle_case(&backend, &case).unwrap();

            let actual = execution::take_stage_snapshots();
            let expected = expected_stage_snapshots(case.name);
            assert_eq!(actual.len(), 149, "case {}", case.name);
            assert_eq!(actual.len(), expected.len(), "case {}", case.name);
            for (actual, (expected_label, expected_bits)) in actual.iter().zip(&expected) {
                assert_eq!(&actual.label, expected_label, "case {}", case.name);
                assert_eq!(
                    actual.bits.len(),
                    expected_bits.len(),
                    "case {}, stage {expected_label}",
                    case.name
                );
                for (index, (&actual_bits, &expected_bits)) in
                    actual.bits.iter().zip(expected_bits).enumerate()
                {
                    if f32::from_bits(expected_bits).is_nan() {
                        assert!(
                            f32::from_bits(actual_bits).is_nan(),
                            "case {}, stage {expected_label}, index {index}",
                            case.name
                        );
                    } else {
                        assert_eq!(
                            actual_bits, expected_bits,
                            "case {}, stage {expected_label}, index {index}",
                            case.name
                        );
                    }
                }
            }
            if case.injects_ieee_values {
                assert_ieee_snapshot_coverage(&actual);
            }
        }
    }

    #[test]
    fn one_and_four_workers_produce_identical_bits_for_every_oracle_case() {
        for case in oracle_cases() {
            let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
            let four_workers = CpuBackend::try_with_worker_count(4).unwrap();
            let mut one_worker_fixture = StageFixture::new(&one_worker);
            let mut four_worker_fixture = StageFixture::new(&four_workers);

            one_worker_fixture
                .execute_oracle_case(&one_worker, &case)
                .unwrap();
            four_worker_fixture
                .execute_oracle_case(&four_workers, &case)
                .unwrap();

            assert_eq!(
                one_worker_fixture.mutable_bits(),
                four_worker_fixture.mutable_bits(),
                "case {}",
                case.name
            );
        }
    }

    #[test]
    fn late_boundary_region_failure_preserves_every_mutable_field() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut fixture = StageFixture::new(&backend);
        let before = fixture.mutable_bits();

        let result = fixture.execute(
            &backend,
            PhysicalBoundaryConditions::specified_lateral(),
            true,
        );

        assert!(matches!(
            result,
            Err(AcousticBoundaryStageError::RegionLocationMismatch { .. })
        ));
        assert_eq!(fixture.mutable_bits(), before);
    }

    #[test]
    fn unsupported_polar_filter_fails_before_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut fixture = StageFixture::new(&backend);
        let before = fixture.mutable_bits();
        let mut conditions = PhysicalBoundaryConditions::specified_lateral();
        conditions.polar = true;

        let result = fixture.execute(&backend, conditions, false);

        assert_eq!(
            result.unwrap_err(),
            AcousticBoundaryStageError::PolarFilteringUnsupported
        );
        assert_eq!(fixture.mutable_bits(), before);
    }

    #[derive(Clone)]
    struct StageFixture {
        volume_fields: [CpuField<f32>; VOLUME_FIELD_COUNT],
        horizontal_fields: [CpuField<f32>; HORIZONTAL_FIELD_COUNT],
        volume_inputs: [CpuField<f32>; 12],
        horizontal_inputs: [CpuField<f32>; 13],
        coefficients: [Vec<f32>; 10],
    }

    impl StageFixture {
        fn new(backend: &CpuBackend) -> Self {
            let volume_field_bases = [
                0.10, 0.20, 0.30, 0.40, 0.50, 0.60, 299.0, 300.0, 9.0, 10.0, 0.11, 0.12, 0.13,
                299.5, 9.5, 0.16, 0.17, 0.18, 0.19, 0.20, 0.21, 0.22, 0.23, 0.24, 299.75, 0.26,
                0.27, 0.28, 0.29,
            ];
            let horizontal_field_bases = [1.0, 1.1, 0.03, 0.09, 0.10, 0.11, 0.12, 0.13];
            let horizontal_field_roles = [1, 2, 3, 9, 10, 11, 12, 13];
            let volume_input_bases = [
                80_000.0, 1.0, 10.0, 1_000.0, 0.01, 0.01, 0.01, 0.01, 0.01, 1.0, 1.0, 1.0,
            ];
            let horizontal_input_bases = [
                10.0, 10.1, 10.2, 10.3, 0.01, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0,
            ];
            let horizontal_input_roles = [4, 5, 6, 7, 8, 14, 15, 16, 17, 18, 19, 20, 21];
            let coefficient_bases = [0.60, 0.40, 0.55, 0.45, 1.0, 0.20, 1.0, 1.0, 0.60, 0.40];
            let coefficient_roles = [1, 2, 3, 4, 9, 10, 11, 12, 13, 14];
            Self {
                volume_fields: std::array::from_fn(|index| {
                    initialized_volume(backend, volume_field_bases[index], index + 1)
                }),
                horizontal_fields: std::array::from_fn(|index| {
                    initialized_horizontal(
                        backend,
                        horizontal_field_bases[index],
                        horizontal_field_roles[index],
                    )
                }),
                volume_inputs: std::array::from_fn(|index| {
                    initialized_volume(backend, volume_input_bases[index], index + 30)
                }),
                horizontal_inputs: std::array::from_fn(|index| {
                    initialized_horizontal(
                        backend,
                        horizontal_input_bases[index],
                        horizontal_input_roles[index],
                    )
                }),
                coefficients: std::array::from_fn(|index| {
                    initialized_coefficient(coefficient_bases[index], coefficient_roles[index])
                }),
            }
        }

        fn execute(
            &mut self,
            backend: &CpuBackend,
            conditions: PhysicalBoundaryConditions,
            wrong_potential_temperature_location: bool,
        ) -> AcousticBoundaryStageResult<()> {
            self.execute_with_tiles(
                backend,
                conditions,
                4..11,
                4..11,
                wrong_potential_temperature_location,
            )
        }

        fn execute_oracle_case(
            &mut self,
            backend: &CpuBackend,
            case: &StageOracleCase,
        ) -> AcousticBoundaryStageResult<()> {
            if case.injects_ieee_values {
                self.install_ieee_values();
            }
            self.execute_with_tiles(
                backend,
                case.conditions,
                case.west_east_tile.clone(),
                case.south_north_tile.clone(),
                false,
            )
        }

        fn execute_with_tiles(
            &mut self,
            backend: &CpuBackend,
            conditions: PhysicalBoundaryConditions,
            west_east_tile: Range<usize>,
            south_north_tile: Range<usize>,
            wrong_potential_temperature_location: bool,
        ) -> AcousticBoundaryStageResult<()> {
            let shape = stage_shape();
            let mass_west_east_tile = west_east_tile.start..west_east_tile.end.min(10);
            let mass_south_north_tile = south_north_tile.start..south_north_tile.end.min(10);
            let preparation_region = AcousticStepPreparationRegion::try_new(
                shape,
                4..10,
                4..10,
                1..6,
                west_east_tile.clone(),
                south_north_tile.clone(),
                1..7,
            )
            .unwrap();
            let pressure_region = AcousticPressureRegion::try_new(
                shape,
                4..10,
                4..10,
                1..6,
                mass_west_east_tile.clone(),
                mass_south_north_tile.clone(),
                1..6,
            )
            .unwrap();
            let vertical_coefficient_region = VerticalAcousticCoefficientRegion::try_new(
                shape,
                4..10,
                4..10,
                1..6,
                mass_west_east_tile.clone(),
                mass_south_north_tile.clone(),
            )
            .unwrap();
            let horizontal_momentum_region = AcousticHorizontalMomentumRegion::try_new(
                shape,
                4..10,
                4..10,
                1..6,
                west_east_tile.clone(),
                south_north_tile.clone(),
                1..7,
            )
            .unwrap();
            let mass_theta_region = AcousticMassThetaRegion::try_new(
                shape,
                4..10,
                4..10,
                1..6,
                mass_west_east_tile.clone(),
                mass_south_north_tile.clone(),
                1..7,
            )
            .unwrap();
            let vertical_momentum_region = AcousticVerticalRegion::try_new(
                shape,
                4..10,
                4..10,
                1..6,
                mass_west_east_tile,
                mass_south_north_tile,
                1..7,
            )
            .unwrap();
            let flux_accumulation_region = AcousticFluxAccumulationRegion::try_new(
                shape,
                4..10,
                4..10,
                1..6,
                west_east_tile.clone(),
                south_north_tile.clone(),
                1..7,
            )
            .unwrap();
            let physical_region = PhysicalBoundaryRegion::try_new(
                shape,
                4..10,
                4..10,
                1..6,
                west_east_tile.clone(),
                south_north_tile.clone(),
                1..7,
            )
            .unwrap();
            let west_east_boundary_region = specified_region(
                shape,
                SpecifiedBoundaryFieldLocation::WestEastFace,
                1..6,
                west_east_tile.clone(),
                south_north_tile.clone(),
                1..7,
            );
            let south_north_boundary_region = specified_region(
                shape,
                SpecifiedBoundaryFieldLocation::SouthNorthFace,
                1..6,
                west_east_tile.clone(),
                south_north_tile.clone(),
                1..7,
            );
            let potential_temperature_location = if wrong_potential_temperature_location {
                SpecifiedBoundaryFieldLocation::FullLevel
            } else {
                SpecifiedBoundaryFieldLocation::MassHalfLevel
            };
            let potential_temperature_region = specified_region(
                shape,
                potential_temperature_location,
                1..6,
                west_east_tile.clone(),
                south_north_tile.clone(),
                if wrong_potential_temperature_location {
                    1..7
                } else {
                    1..6
                },
            );
            let column_mass_region = specified_region(
                shape.horizontal_shape(),
                SpecifiedBoundaryFieldLocation::HorizontalMass,
                0..1,
                west_east_tile.clone(),
                south_north_tile.clone(),
                0..1,
            );
            let full_level_region = specified_region(
                shape,
                SpecifiedBoundaryFieldLocation::FullLevel,
                1..6,
                west_east_tile,
                south_north_tile,
                1..7,
            );
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

            backend.advance_acoustic_boundary_stage(
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
                AcousticBoundaryStageControls::new(trajectory_controls(), conditions, 2),
                AcousticBoundaryStageRegions::new(
                    AcousticTrajectoryRegions::new(
                        &preparation_region,
                        &pressure_region,
                        &vertical_coefficient_region,
                        &horizontal_momentum_region,
                        &mass_theta_region,
                        &vertical_momentum_region,
                        &flux_accumulation_region,
                    ),
                    &physical_region,
                    &west_east_boundary_region,
                    &south_north_boundary_region,
                    &potential_temperature_region,
                    &column_mass_region,
                    &full_level_region,
                ),
            )
        }

        fn install_ieee_values(&mut self) {
            let tendency = &mut self.volume_inputs[4];
            for ((west_east, bottom_top, south_north), bits) in [
                ((0, 0, 0), 0x8000_0000),
                ((4, 1, 4), 0x7f80_0000),
                ((5, 2, 5), 0x7fc0_002a),
                ((9, 4, 9), 0xff80_0000),
                ((14, 6, 14), 0x7f7f_ffff),
            ] {
                tendency.values_mut()[volume_index(west_east, bottom_top, south_north)] =
                    f32::from_bits(bits);
            }
        }

        fn mutable_bits(&self) -> Vec<u32> {
            self.volume_fields
                .iter()
                .chain(&self.horizontal_fields)
                .chain([&self.volume_inputs[4], &self.volume_inputs[5]])
                .flat_map(|field| field.values().iter().map(|value| value.to_bits()))
                .collect()
        }
    }

    fn specified_region(
        shape: GridShape,
        location: SpecifiedBoundaryFieldLocation,
        half_level_domain: std::ops::Range<usize>,
        west_east_tile: std::ops::Range<usize>,
        south_north_tile: std::ops::Range<usize>,
        bottom_top_tile: std::ops::Range<usize>,
    ) -> SpecifiedBoundaryUpdateRegion {
        SpecifiedBoundaryUpdateRegion::try_new(
            shape,
            location,
            4..10,
            4..10,
            half_level_domain,
            west_east_tile,
            south_north_tile,
            bottom_top_tile,
        )
        .unwrap()
    }

    fn stage_shape() -> GridShape {
        GridShape::try_new(15, 15, 7).unwrap()
    }

    fn assert_field_matches_oracle(case_name: &str, field_name: &str, actual: &[f32]) {
        let label = format!("{case_name}:final:{field_name}");
        let matching_lines = include_str!("../../../test-data/acoustic_boundary_stage.out.correct")
            .lines()
            .filter(|line| line.split_whitespace().next() == Some(label.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(matching_lines.len(), 1, "field {label}");
        let expected = matching_lines[0]
            .split_whitespace()
            .skip(1)
            .map(|bits| u32::from_str_radix(bits, 16).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual.len(), expected.len(), "field {label}");
        for (index, (&actual_value, &expected_bits)) in
            actual.iter().zip(expected.iter()).enumerate()
        {
            let expected_value = f32::from_bits(expected_bits);
            if expected_value.is_nan() {
                assert!(actual_value.is_nan(), "field {label}, index {index}");
            } else {
                assert_eq!(
                    actual_value.to_bits(),
                    expected_bits,
                    "field {label}, index {index}"
                );
            }
        }
    }

    fn expected_stage_snapshots(case_name: &str) -> Vec<(String, Vec<u32>)> {
        let prefix = format!("{case_name}:");
        include_str!("../../../test-data/acoustic_boundary_stage.out.correct")
            .lines()
            .filter(|line| {
                line.starts_with(&prefix) && !line.starts_with(&format!("{prefix}final:"))
            })
            .map(|line| {
                let mut columns = line.split_whitespace();
                let label = columns
                    .next()
                    .unwrap()
                    .strip_prefix(&prefix)
                    .unwrap()
                    .to_owned();
                let bits = columns
                    .map(|bits| u32::from_str_radix(bits, 16).unwrap())
                    .collect();
                (label, bits)
            })
            .collect()
    }

    fn assert_ieee_snapshot_coverage(actual: &[execution::AcousticBoundaryStageSnapshot]) {
        let tendency = actual
            .iter()
            .find(|snapshot| snapshot.label == "physical_initial:ru_tend")
            .unwrap();
        assert_eq!(tendency.bits[volume_index(0, 0, 0)], 0x8000_0000);
        assert_eq!(tendency.bits[volume_index(4, 1, 4)], 0x7f80_0000);
        assert!(f32::from_bits(tendency.bits[volume_index(5, 2, 5)]).is_nan());
        assert_eq!(tendency.bits[volume_index(9, 4, 9)], 0xff80_0000);
        assert_eq!(tendency.bits[volume_index(14, 6, 14)], 0x7f7f_ffff);
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
}
