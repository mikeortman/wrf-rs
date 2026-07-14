use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, GridShape, ParallelExecutionError};

use crate::{
    OmegaDiagnosisCoefficient, OmegaDiagnosisCoefficients, OmegaDiagnosisError,
    OmegaDiagnosisField, OmegaDiagnosisGridMetrics, OmegaDiagnosisKernels,
    OmegaDiagnosisMapFactors, OmegaDiagnosisMasses, OmegaDiagnosisRegion, OmegaDiagnosisResult,
    OmegaDiagnosisVelocities,
};

use super::row::{
    OmegaDiagnosisLevelCoefficients, OmegaDiagnosisMapFactorRows, OmegaDiagnosisMassRows,
    OmegaDiagnosisOutputRows, OmegaDiagnosisVelocityRows,
};

impl OmegaDiagnosisKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn diagnose_omega(
        &self,
        omega: &mut Self::Field,
        velocities: OmegaDiagnosisVelocities<'_, Self::Field>,
        masses: OmegaDiagnosisMasses<'_, Self::Field>,
        map_factors: OmegaDiagnosisMapFactors<'_, Self::Field>,
        coefficients: OmegaDiagnosisCoefficients<'_>,
        grid_metrics: OmegaDiagnosisGridMetrics,
        region: &OmegaDiagnosisRegion,
    ) -> OmegaDiagnosisResult<()> {
        validate_operation(
            omega,
            &velocities,
            &masses,
            &map_factors,
            coefficients,
            region,
        )?;

        let shape = region.shape();
        let plane_length = shape.point_count() / shape.south_north_points();

        self.try_for_each_output_block(
            omega.values_mut(),
            plane_length,
            |south_north_index, omega_plane| {
                if region.active_south_north().contains(&south_north_index) {
                    diagnose_plane(
                        omega_plane,
                        south_north_index,
                        shape,
                        &velocities,
                        &masses,
                        &map_factors,
                        coefficients,
                        grid_metrics,
                        region,
                    );
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
    }
}

pub(crate) fn validate_operation(
    omega: &CpuField<f32>,
    velocities: &OmegaDiagnosisVelocities<'_, CpuField<f32>>,
    masses: &OmegaDiagnosisMasses<'_, CpuField<f32>>,
    map_factors: &OmegaDiagnosisMapFactors<'_, CpuField<f32>>,
    coefficients: OmegaDiagnosisCoefficients<'_>,
    region: &OmegaDiagnosisRegion,
) -> OmegaDiagnosisResult<()> {
    validate_fields(omega, velocities, masses, map_factors, region)?;
    validate_coefficients(coefficients, region.shape().bottom_top_points())
}

fn validate_fields(
    omega: &CpuField<f32>,
    velocities: &OmegaDiagnosisVelocities<'_, CpuField<f32>>,
    masses: &OmegaDiagnosisMasses<'_, CpuField<f32>>,
    map_factors: &OmegaDiagnosisMapFactors<'_, CpuField<f32>>,
    region: &OmegaDiagnosisRegion,
) -> OmegaDiagnosisResult<()> {
    let volume_shape = region.shape();
    let horizontal_shape = volume_shape.horizontal_shape();
    for (field, role) in [
        (omega, OmegaDiagnosisField::OmegaOutput),
        (velocities.west_east, OmegaDiagnosisField::WestEastVelocity),
        (
            velocities.south_north,
            OmegaDiagnosisField::SouthNorthVelocity,
        ),
    ] {
        validate_field_shape(field, role, volume_shape)?;
    }
    for (field, role) in [
        (
            masses.perturbation,
            OmegaDiagnosisField::PerturbationColumnMass,
        ),
        (masses.base_state, OmegaDiagnosisField::BaseColumnMass),
        (
            map_factors.mass_point_west_east,
            OmegaDiagnosisField::MassPointWestEastMapFactor,
        ),
        (
            map_factors.west_east_momentum_south_north,
            OmegaDiagnosisField::WestEastMomentumSouthNorthMapFactor,
        ),
        (
            map_factors.inverse_south_north_momentum_west_east,
            OmegaDiagnosisField::InverseSouthNorthMomentumWestEastMapFactor,
        ),
    ] {
        validate_field_shape(field, role, horizontal_shape)?;
    }
    Ok(())
}

fn validate_field_shape(
    field: &CpuField<f32>,
    role: OmegaDiagnosisField,
    expected: GridShape,
) -> OmegaDiagnosisResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(OmegaDiagnosisError::FieldShapeMismatch {
            field: role,
            expected,
            actual,
        });
    }
    Ok(())
}

fn validate_coefficients(
    coefficients: OmegaDiagnosisCoefficients<'_>,
    expected: usize,
) -> OmegaDiagnosisResult<()> {
    for (values, coefficient) in [
        (
            coefficients.half_level_mass_multiplier,
            OmegaDiagnosisCoefficient::HalfLevelMassMultiplier,
        ),
        (
            coefficients.half_level_mass_offset,
            OmegaDiagnosisCoefficient::HalfLevelMassOffset,
        ),
        (
            coefficients.eta_layer_thickness,
            OmegaDiagnosisCoefficient::EtaLayerThickness,
        ),
    ] {
        if values.len() != expected {
            return Err(OmegaDiagnosisError::CoefficientLengthMismatch {
                coefficient,
                expected,
                actual: values.len(),
            });
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn diagnose_plane(
    omega_plane: &mut [f32],
    south_north_index: usize,
    shape: GridShape,
    velocities: &OmegaDiagnosisVelocities<'_, CpuField<f32>>,
    masses: &OmegaDiagnosisMasses<'_, CpuField<f32>>,
    map_factors: &OmegaDiagnosisMapFactors<'_, CpuField<f32>>,
    coefficients: OmegaDiagnosisCoefficients<'_>,
    grid_metrics: OmegaDiagnosisGridMetrics,
    region: &OmegaDiagnosisRegion,
) {
    let west_east_points = shape.west_east_points();
    let half_levels = region.half_levels();
    let bottom_full_level = half_levels.start;
    let top_full_level = region.top_full_level();
    let west_east_tile = region.west_east_tile();
    let active_west_east = region.active_west_east();

    for west_east_index in west_east_tile {
        omega_plane[plane_index(west_east_index, bottom_full_level, west_east_points)] = 0.0;
        omega_plane[plane_index(west_east_index, top_full_level, west_east_points)] = 0.0;
    }

    accumulate_horizontal_divergence(
        omega_plane,
        south_north_index,
        shape,
        velocities,
        masses,
        map_factors,
        coefficients,
        grid_metrics,
        region,
    );

    for west_east_index in active_west_east {
        let tendency_index = plane_index(west_east_index, top_full_level, west_east_points);
        let column_mass_tendency = omega_plane[tendency_index];
        let mut previous_omega = 0.0_f32;

        for half_level_index in half_levels.start..half_levels.end - 1 {
            let output_index = plane_index(west_east_index, half_level_index, west_east_points);
            let divergence = omega_plane[output_index];
            omega_plane[output_index] = previous_omega;
            previous_omega = previous_omega
                - coefficients.eta_layer_thickness[half_level_index]
                    * coefficients.half_level_mass_multiplier[half_level_index]
                    * column_mass_tendency
                - divergence;
        }

        let last_internal_level = half_levels.end - 1;
        omega_plane[plane_index(west_east_index, last_internal_level, west_east_points)] =
            previous_omega;
        omega_plane[tendency_index] = 0.0;
    }
}

#[allow(clippy::too_many_arguments)]
fn accumulate_horizontal_divergence(
    omega_plane: &mut [f32],
    south_north_index: usize,
    shape: GridShape,
    velocities: &OmegaDiagnosisVelocities<'_, CpuField<f32>>,
    masses: &OmegaDiagnosisMasses<'_, CpuField<f32>>,
    map_factors: &OmegaDiagnosisMapFactors<'_, CpuField<f32>>,
    coefficients: OmegaDiagnosisCoefficients<'_>,
    grid_metrics: OmegaDiagnosisGridMetrics,
    region: &OmegaDiagnosisRegion,
) {
    let west_east_points = shape.west_east_points();
    let bottom_top_points = shape.bottom_top_points();
    let active_west_east = region.active_west_east();
    let active_start = active_west_east.start;
    let active_end = active_west_east.end;
    let current_horizontal_row_start = south_north_index * west_east_points;
    let south_horizontal_row_start = (south_north_index - 1) * west_east_points;
    let north_horizontal_row_start = (south_north_index + 1) * west_east_points;
    let perturbation = masses.perturbation.values();
    let base_state = masses.base_state.values();
    let mass_rows = OmegaDiagnosisMassRows::new(
        &perturbation[current_horizontal_row_start + active_start
            ..current_horizontal_row_start + active_end],
        &perturbation[current_horizontal_row_start + active_start - 1
            ..current_horizontal_row_start + active_end - 1],
        &perturbation[current_horizontal_row_start + active_start + 1
            ..current_horizontal_row_start + active_end + 1],
        &perturbation
            [south_horizontal_row_start + active_start..south_horizontal_row_start + active_end],
        &perturbation
            [north_horizontal_row_start + active_start..north_horizontal_row_start + active_end],
        &base_state[current_horizontal_row_start + active_start
            ..current_horizontal_row_start + active_end],
        &base_state[current_horizontal_row_start + active_start - 1
            ..current_horizontal_row_start + active_end - 1],
        &base_state[current_horizontal_row_start + active_start + 1
            ..current_horizontal_row_start + active_end + 1],
        &base_state
            [south_horizontal_row_start + active_start..south_horizontal_row_start + active_end],
        &base_state
            [north_horizontal_row_start + active_start..north_horizontal_row_start + active_end],
    );
    let mass_point_map_factor = map_factors.mass_point_west_east.values();
    let west_east_map_factor = map_factors.west_east_momentum_south_north.values();
    let inverse_south_north_map_factor =
        map_factors.inverse_south_north_momentum_west_east.values();
    let map_factor_rows = OmegaDiagnosisMapFactorRows::new(
        &mass_point_map_factor[current_horizontal_row_start + active_start
            ..current_horizontal_row_start + active_end],
        &west_east_map_factor[current_horizontal_row_start + active_start
            ..current_horizontal_row_start + active_end],
        &west_east_map_factor[current_horizontal_row_start + active_start + 1
            ..current_horizontal_row_start + active_end + 1],
        &inverse_south_north_map_factor[current_horizontal_row_start + active_start
            ..current_horizontal_row_start + active_end],
        &inverse_south_north_map_factor
            [north_horizontal_row_start + active_start..north_horizontal_row_start + active_end],
    );

    let west_east_velocity = velocities.west_east.values();
    let south_north_velocity = velocities.south_north.values();
    let current_volume_plane_start = south_north_index * bottom_top_points * west_east_points;
    let north_volume_plane_start = (south_north_index + 1) * bottom_top_points * west_east_points;
    let top_row_start = region.top_full_level() * west_east_points;

    for bottom_top_index in region.half_levels() {
        let current_volume_row_start =
            current_volume_plane_start + bottom_top_index * west_east_points;
        let north_volume_row_start = north_volume_plane_start + bottom_top_index * west_east_points;
        let velocity_rows = OmegaDiagnosisVelocityRows::new(
            &west_east_velocity
                [current_volume_row_start + active_start..current_volume_row_start + active_end],
            &west_east_velocity[current_volume_row_start + active_start + 1
                ..current_volume_row_start + active_end + 1],
            &south_north_velocity
                [current_volume_row_start + active_start..current_volume_row_start + active_end],
            &south_north_velocity
                [north_volume_row_start + active_start..north_volume_row_start + active_end],
        );
        let (before_top, top_and_after) = omega_plane.split_at_mut(top_row_start);
        let divergence_row_start = bottom_top_index * west_east_points;
        let divergence =
            &mut before_top[divergence_row_start + active_start..divergence_row_start + active_end];
        let column_mass_tendency = &mut top_and_after[active_start..active_end];
        let mut output_rows = OmegaDiagnosisOutputRows::new(divergence, column_mass_tendency);
        output_rows.calculate_and_accumulate(
            &velocity_rows,
            &mass_rows,
            &map_factor_rows,
            OmegaDiagnosisLevelCoefficients::new(
                coefficients.half_level_mass_multiplier[bottom_top_index],
                coefficients.half_level_mass_offset[bottom_top_index],
                coefficients.eta_layer_thickness[bottom_top_index],
            ),
            grid_metrics,
        );
    }
}

fn plane_index(west_east_index: usize, bottom_top_index: usize, west_east_points: usize) -> usize {
    bottom_top_index * west_east_points + west_east_index
}

#[cfg(test)]
fn volume_index(
    west_east_index: usize,
    bottom_top_index: usize,
    south_north_index: usize,
    shape: GridShape,
) -> usize {
    (south_north_index * shape.bottom_top_points() + bottom_top_index) * shape.west_east_points()
        + west_east_index
}

fn map_parallel_error(error: ParallelExecutionError<Infallible>) -> OmegaDiagnosisError {
    match error {
        ParallelExecutionError::Kernel(never) => match never {},
        ParallelExecutionError::WorkerPanicked => OmegaDiagnosisError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. }
        | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
            unreachable!("validated field shapes produce complete non-empty omega planes")
        }
    }
}

#[cfg(test)]
mod tests {
    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;

    #[test]
    fn matches_upstream_fortran_clipping_boundaries_and_exceptional_values() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();

        for case in oracle_cases() {
            let (mut fixture, region) = create_oracle_fixture(&backend, case);
            apply_fixture(&backend, &mut fixture, &region).unwrap();
            assert_matches_fortran(case.name, &fixture.omega);
        }
    }

    #[test]
    fn is_bitwise_deterministic_across_worker_counts() {
        let single_worker_backend = CpuBackend::try_with_worker_count(1).unwrap();
        let four_worker_backend = CpuBackend::try_with_worker_count(4).unwrap();
        let case = oracle_cases()[3];
        let (mut single_worker_fixture, region) =
            create_oracle_fixture(&single_worker_backend, case);
        let (mut four_worker_fixture, _) = create_oracle_fixture(&four_worker_backend, case);

        apply_fixture(&single_worker_backend, &mut single_worker_fixture, &region).unwrap();
        apply_fixture(&four_worker_backend, &mut four_worker_fixture, &region).unwrap();

        assert_eq!(single_worker_fixture.omega, four_worker_fixture.omega);
    }

    #[test]
    fn rejects_shape_and_coefficient_mismatches_before_mutating_output() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = oracle_cases()[3];
        let (mut fixture, region) = create_oracle_fixture(&backend, case);
        let wrong_shape = GridShape::try_new(7, 7, 1).unwrap();
        fixture.perturbation_mass = backend.create_field(wrong_shape, 1.0).unwrap();

        assert_eq!(
            apply_fixture(&backend, &mut fixture, &region),
            Err(OmegaDiagnosisError::FieldShapeMismatch {
                field: OmegaDiagnosisField::PerturbationColumnMass,
                expected: region.shape().horizontal_shape(),
                actual: wrong_shape,
            })
        );
        assert_output_retains_sentinel(&fixture);

        let (mut fixture, region) = create_oracle_fixture(&backend, case);
        fixture.eta_layer_thickness.pop();
        assert_eq!(
            apply_fixture(&backend, &mut fixture, &region),
            Err(OmegaDiagnosisError::CoefficientLengthMismatch {
                coefficient: OmegaDiagnosisCoefficient::EtaLayerThickness,
                expected: 7,
                actual: 6,
            })
        );
        assert_output_retains_sentinel(&fixture);
    }

    #[test]
    fn validates_every_field_and_coefficient_role() {
        let backend = CpuBackend::try_with_worker_count(1).unwrap();
        let expected_shape = GridShape::try_new(5, 5, 5).unwrap();
        let actual_shape = GridShape::try_new(4, 5, 5).unwrap();
        let field = backend.create_field(actual_shape, 0.0).unwrap();
        let field_roles = [
            OmegaDiagnosisField::OmegaOutput,
            OmegaDiagnosisField::WestEastVelocity,
            OmegaDiagnosisField::SouthNorthVelocity,
            OmegaDiagnosisField::PerturbationColumnMass,
            OmegaDiagnosisField::BaseColumnMass,
            OmegaDiagnosisField::MassPointWestEastMapFactor,
            OmegaDiagnosisField::WestEastMomentumSouthNorthMapFactor,
            OmegaDiagnosisField::InverseSouthNorthMomentumWestEastMapFactor,
        ];
        for role in field_roles {
            assert_eq!(
                validate_field_shape(&field, role, expected_shape),
                Err(OmegaDiagnosisError::FieldShapeMismatch {
                    field: role,
                    expected: expected_shape,
                    actual: actual_shape,
                })
            );
        }

        let complete = [1.0_f32; 5];
        let short = [1.0_f32; 4];
        let cases = [
            (
                OmegaDiagnosisCoefficients::new(&short, &complete, &complete),
                OmegaDiagnosisCoefficient::HalfLevelMassMultiplier,
            ),
            (
                OmegaDiagnosisCoefficients::new(&complete, &short, &complete),
                OmegaDiagnosisCoefficient::HalfLevelMassOffset,
            ),
            (
                OmegaDiagnosisCoefficients::new(&complete, &complete, &short),
                OmegaDiagnosisCoefficient::EtaLayerThickness,
            ),
        ];
        for (coefficients, expected_role) in cases {
            assert_eq!(
                validate_coefficients(coefficients, 5),
                Err(OmegaDiagnosisError::CoefficientLengthMismatch {
                    coefficient: expected_role,
                    expected: 5,
                    actual: 4,
                })
            );
        }
    }

    #[derive(Clone, Copy)]
    struct OracleCase {
        name: &'static str,
        west_east_tile: (i32, i32),
        south_north_tile: (i32, i32),
        exceptional: bool,
    }

    fn oracle_cases() -> [OracleCase; 5] {
        [
            OracleCase {
                name: "interior",
                west_east_tile: (0, 2),
                south_north_tile: (5, 6),
                exceptional: false,
            },
            OracleCase {
                name: "x_upper",
                west_east_tile: (-1, 4),
                south_north_tile: (5, 6),
                exceptional: false,
            },
            OracleCase {
                name: "y_upper",
                west_east_tile: (0, 2),
                south_north_tile: (4, 8),
                exceptional: false,
            },
            OracleCase {
                name: "all_boundaries",
                west_east_tile: (-1, 4),
                south_north_tile: (4, 8),
                exceptional: false,
            },
            OracleCase {
                name: "exceptional_finite",
                west_east_tile: (-1, 4),
                south_north_tile: (4, 8),
                exceptional: true,
            },
        ]
    }

    struct OmegaDiagnosisFixture {
        shape: GridShape,
        omega: CpuField<f32>,
        west_east_velocity: CpuField<f32>,
        south_north_velocity: CpuField<f32>,
        perturbation_mass: CpuField<f32>,
        base_mass: CpuField<f32>,
        mass_point_west_east_map_factor: CpuField<f32>,
        west_east_momentum_south_north_map_factor: CpuField<f32>,
        inverse_south_north_momentum_west_east_map_factor: CpuField<f32>,
        half_level_mass_multiplier: Vec<f32>,
        half_level_mass_offset: Vec<f32>,
        eta_layer_thickness: Vec<f32>,
    }

    fn create_oracle_fixture(
        backend: &CpuBackend,
        case: OracleCase,
    ) -> (OmegaDiagnosisFixture, OmegaDiagnosisRegion) {
        let shape = GridShape::try_new(8, 7, 7).unwrap();
        let horizontal_shape = shape.horizontal_shape();
        let mut fixture = OmegaDiagnosisFixture {
            shape,
            omega: backend.create_field(shape, -999.0).unwrap(),
            west_east_velocity: backend.create_field(shape, 0.0).unwrap(),
            south_north_velocity: backend.create_field(shape, 0.0).unwrap(),
            perturbation_mass: backend.create_field(horizontal_shape, 0.0).unwrap(),
            base_mass: backend.create_field(horizontal_shape, 0.0).unwrap(),
            mass_point_west_east_map_factor: backend.create_field(horizontal_shape, 0.0).unwrap(),
            west_east_momentum_south_north_map_factor: backend
                .create_field(horizontal_shape, 0.0)
                .unwrap(),
            inverse_south_north_momentum_west_east_map_factor: backend
                .create_field(horizontal_shape, 0.0)
                .unwrap(),
            half_level_mass_multiplier: vec![0.0; 7],
            half_level_mass_offset: vec![0.0; 7],
            eta_layer_thickness: vec![0.0; 7],
        };
        initialize_fixture(&mut fixture);
        if case.exceptional {
            apply_exceptional_inputs(&mut fixture);
        }
        let region = OmegaDiagnosisRegion::try_new(
            shape,
            1..6,
            1..5,
            2..5,
            inclusive_fortran_range(case.west_east_tile, -2),
            inclusive_fortran_range(case.south_north_tile, 3),
            inclusive_fortran_range((1, 4), -1),
        )
        .unwrap();
        (fixture, region)
    }

    fn initialize_fixture(fixture: &mut OmegaDiagnosisFixture) {
        for bottom_top_index in 0..7 {
            let bottom_top = (bottom_top_index as i32 - 1) as f32;
            fixture.half_level_mass_multiplier[bottom_top_index] = 0.65 + bottom_top * 0.03;
            fixture.half_level_mass_offset[bottom_top_index] = 1.4 - bottom_top * 0.08;
            fixture.eta_layer_thickness[bottom_top_index] = -0.2 - bottom_top * 0.015;
        }
        for south_north_index in 0..7 {
            let south_north = (south_north_index as i32 + 3) as f32;
            for west_east_index in 0..8 {
                let west_east = (west_east_index as i32 - 2) as f32;
                let horizontal_index = south_north_index * 8 + west_east_index;
                fixture.perturbation_mass.values_mut()[horizontal_index] =
                    -4.0 + west_east * 0.45 - south_north * 0.2;
                fixture.base_mass.values_mut()[horizontal_index] =
                    95.0 + west_east * 0.3 + south_north * 0.75;
                fixture.mass_point_west_east_map_factor.values_mut()[horizontal_index] =
                    0.9 + west_east_index as f32 * 0.007 + south_north_index as f32 * 0.004;
                fixture
                    .west_east_momentum_south_north_map_factor
                    .values_mut()[horizontal_index] =
                    1.1 + west_east_index as f32 * 0.009 + south_north_index as f32 * 0.003;
                fixture
                    .inverse_south_north_momentum_west_east_map_factor
                    .values_mut()[horizontal_index] = 1.0
                    / (1.05 + west_east_index as f32 * 0.006 + south_north_index as f32 * 0.005);

                for bottom_top_index in 0..7 {
                    let bottom_top = (bottom_top_index as i32 - 1) as f32;
                    let index = volume_index(
                        west_east_index,
                        bottom_top_index,
                        south_north_index,
                        fixture.shape,
                    );
                    fixture.west_east_velocity.values_mut()[index] =
                        -2.5 + west_east * 0.17 + bottom_top * 0.29 + south_north * 0.11;
                    fixture.south_north_velocity.values_mut()[index] =
                        1.75 - west_east * 0.13 + bottom_top * 0.21 - south_north * 0.07;
                }
            }
        }
    }

    fn apply_exceptional_inputs(fixture: &mut OmegaDiagnosisFixture) {
        let west_east_index = 1;
        let bottom_top_index = 2;
        let south_north_index = 1;
        let index = volume_index(
            west_east_index,
            bottom_top_index,
            south_north_index,
            fixture.shape,
        );
        let east_index = volume_index(
            west_east_index + 1,
            bottom_top_index,
            south_north_index,
            fixture.shape,
        );
        fixture.west_east_velocity.values_mut()[index] = f32::MAX;
        fixture.west_east_velocity.values_mut()[east_index] = -f32::MAX;
        fixture.south_north_velocity.values_mut()[index] = f32::MAX;
        fixture
            .west_east_momentum_south_north_map_factor
            .values_mut()[9] = 0.0;
        fixture
            .inverse_south_north_momentum_west_east_map_factor
            .values_mut()[17] = 0.0;
        fixture.half_level_mass_multiplier[2] = 2.0;
        fixture.eta_layer_thickness[2] = -2.0;
    }

    fn apply_fixture(
        backend: &CpuBackend,
        fixture: &mut OmegaDiagnosisFixture,
        region: &OmegaDiagnosisRegion,
    ) -> OmegaDiagnosisResult<()> {
        backend.diagnose_omega(
            &mut fixture.omega,
            OmegaDiagnosisVelocities::new(
                &fixture.west_east_velocity,
                &fixture.south_north_velocity,
            ),
            OmegaDiagnosisMasses::new(&fixture.perturbation_mass, &fixture.base_mass),
            OmegaDiagnosisMapFactors::new(
                &fixture.mass_point_west_east_map_factor,
                &fixture.west_east_momentum_south_north_map_factor,
                &fixture.inverse_south_north_momentum_west_east_map_factor,
            ),
            OmegaDiagnosisCoefficients::new(
                &fixture.half_level_mass_multiplier,
                &fixture.half_level_mass_offset,
                &fixture.eta_layer_thickness,
            ),
            OmegaDiagnosisGridMetrics::new(0.125, 0.2),
            region,
        )
    }

    fn assert_output_retains_sentinel(fixture: &OmegaDiagnosisFixture) {
        assert!(fixture.omega.values().iter().all(|value| *value == -999.0));
    }

    fn assert_matches_fortran(case_name: &str, actual: &CpuField<f32>) {
        let expected = expected_fortran_bits(case_name);
        assert_eq!(actual.values().len(), expected.len());
        for (index, (actual_value, expected_bits)) in
            actual.values().iter().zip(expected).enumerate()
        {
            let expected_value = f32::from_bits(expected_bits);
            if expected_value.is_nan() {
                assert!(
                    actual_value.is_nan(),
                    "non-NaN at {case_name} index {index}"
                );
            } else {
                assert_eq!(
                    actual_value.to_bits(),
                    expected_bits,
                    "raw-bit mismatch at {case_name} index {index}"
                );
            }
        }
    }

    fn expected_fortran_bits(case_name: &str) -> Vec<u32> {
        let bits: Vec<_> = include_str!("../../test-data/omega_diagnosis.out.correct")
            .lines()
            .filter_map(|line| {
                let mut columns = line.split_whitespace();
                if columns.next().unwrap() != case_name {
                    return None;
                }
                columns.next();
                columns.next();
                columns.next();
                Some(u32::from_str_radix(columns.next().unwrap(), 16).unwrap())
            })
            .collect();
        assert_eq!(bits.len(), 392, "incomplete {case_name} oracle");
        bits
    }

    fn inclusive_fortran_range(bounds: (i32, i32), memory_start: i32) -> std::ops::Range<usize> {
        ((bounds.0 - memory_start) as usize)..((bounds.1 - memory_start + 1) as usize)
    }
}
