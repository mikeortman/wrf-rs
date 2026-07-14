use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, GridShape, ParallelExecutionError};

use crate::{
    MomentumCouplingCoefficient, MomentumCouplingCoefficients, MomentumCouplingError,
    MomentumCouplingField, MomentumCouplingKernels, MomentumCouplingMapFactors,
    MomentumCouplingMasses, MomentumCouplingOutputs, MomentumCouplingRegion,
    MomentumCouplingResult, MomentumCouplingVelocities,
};

impl MomentumCouplingKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn couple_momentum(
        &self,
        outputs: MomentumCouplingOutputs<'_, Self::Field>,
        velocities: MomentumCouplingVelocities<'_, Self::Field>,
        masses: MomentumCouplingMasses<'_, Self::Field>,
        map_factors: MomentumCouplingMapFactors<'_, Self::Field>,
        coefficients: MomentumCouplingCoefficients<'_>,
        region: &MomentumCouplingRegion,
    ) -> MomentumCouplingResult<()> {
        validate_operation(
            &outputs,
            &velocities,
            &masses,
            &map_factors,
            coefficients,
            region,
        )?;

        let MomentumCouplingOutputs {
            west_east: west_east_output,
            south_north: south_north_output,
            vertical: vertical_output,
        } = outputs;
        couple_west_east_momentum(
            self,
            west_east_output,
            velocities.west_east,
            masses.west_east,
            map_factors.west_east,
            coefficients,
            region,
        )?;
        couple_south_north_momentum(
            self,
            south_north_output,
            velocities.south_north,
            masses.south_north,
            map_factors.inverse_south_north,
            coefficients,
            region,
        )?;
        couple_vertical_momentum(
            self,
            vertical_output,
            velocities.vertical,
            masses.mass_point,
            map_factors.mass_point,
            coefficients,
            region,
        )
    }
}

pub(crate) fn validate_operation(
    outputs: &MomentumCouplingOutputs<'_, CpuField<f32>>,
    velocities: &MomentumCouplingVelocities<'_, CpuField<f32>>,
    masses: &MomentumCouplingMasses<'_, CpuField<f32>>,
    map_factors: &MomentumCouplingMapFactors<'_, CpuField<f32>>,
    coefficients: MomentumCouplingCoefficients<'_>,
    region: &MomentumCouplingRegion,
) -> MomentumCouplingResult<()> {
    validate_borrowed_operation(
        outputs.west_east,
        outputs.south_north,
        outputs.vertical,
        velocities,
        masses,
        map_factors,
        coefficients,
        region,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_borrowed_operation(
    west_east_output: &CpuField<f32>,
    south_north_output: &CpuField<f32>,
    vertical_output: &CpuField<f32>,
    velocities: &MomentumCouplingVelocities<'_, CpuField<f32>>,
    masses: &MomentumCouplingMasses<'_, CpuField<f32>>,
    map_factors: &MomentumCouplingMapFactors<'_, CpuField<f32>>,
    coefficients: MomentumCouplingCoefficients<'_>,
    region: &MomentumCouplingRegion,
) -> MomentumCouplingResult<()> {
    validate_fields(
        west_east_output,
        south_north_output,
        vertical_output,
        velocities,
        masses,
        map_factors,
        region,
    )?;
    validate_coefficients(coefficients, region.shape().bottom_top_points())
}

fn validate_fields(
    west_east_output: &CpuField<f32>,
    south_north_output: &CpuField<f32>,
    vertical_output: &CpuField<f32>,
    velocities: &MomentumCouplingVelocities<'_, CpuField<f32>>,
    masses: &MomentumCouplingMasses<'_, CpuField<f32>>,
    map_factors: &MomentumCouplingMapFactors<'_, CpuField<f32>>,
    region: &MomentumCouplingRegion,
) -> MomentumCouplingResult<()> {
    let volume_shape = region.shape();
    let horizontal_shape = volume_shape.horizontal_shape();
    for (field, role) in [
        (
            west_east_output,
            MomentumCouplingField::WestEastMomentumOutput,
        ),
        (
            south_north_output,
            MomentumCouplingField::SouthNorthMomentumOutput,
        ),
        (
            vertical_output,
            MomentumCouplingField::VerticalMomentumOutput,
        ),
        (
            velocities.west_east,
            MomentumCouplingField::WestEastVelocity,
        ),
        (
            velocities.south_north,
            MomentumCouplingField::SouthNorthVelocity,
        ),
        (velocities.vertical, MomentumCouplingField::VerticalVelocity),
    ] {
        validate_field_shape(field, role, volume_shape)?;
    }
    for (field, role) in [
        (
            masses.west_east,
            MomentumCouplingField::WestEastMomentumMass,
        ),
        (
            masses.south_north,
            MomentumCouplingField::SouthNorthMomentumMass,
        ),
        (
            masses.mass_point,
            MomentumCouplingField::MassPointColumnMass,
        ),
        (
            map_factors.west_east,
            MomentumCouplingField::WestEastMapFactor,
        ),
        (
            map_factors.inverse_south_north,
            MomentumCouplingField::InverseSouthNorthMapFactor,
        ),
        (
            map_factors.mass_point,
            MomentumCouplingField::MassPointMapFactor,
        ),
    ] {
        validate_field_shape(field, role, horizontal_shape)?;
    }
    Ok(())
}

fn validate_field_shape(
    field: &CpuField<f32>,
    role: MomentumCouplingField,
    expected: GridShape,
) -> MomentumCouplingResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(MomentumCouplingError::FieldShapeMismatch {
            field: role,
            expected,
            actual,
        });
    }
    Ok(())
}

fn validate_coefficients(
    coefficients: MomentumCouplingCoefficients<'_>,
    expected: usize,
) -> MomentumCouplingResult<()> {
    for (values, coefficient) in [
        (
            coefficients.half_level_mass_multiplier,
            MomentumCouplingCoefficient::HalfLevelMassMultiplier,
        ),
        (
            coefficients.half_level_offset,
            MomentumCouplingCoefficient::HalfLevelOffset,
        ),
        (
            coefficients.full_level_mass_multiplier,
            MomentumCouplingCoefficient::FullLevelMassMultiplier,
        ),
        (
            coefficients.full_level_offset,
            MomentumCouplingCoefficient::FullLevelOffset,
        ),
    ] {
        if values.len() != expected {
            return Err(MomentumCouplingError::CoefficientLengthMismatch {
                coefficient,
                expected,
                actual: values.len(),
            });
        }
    }
    Ok(())
}

fn couple_west_east_momentum(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    velocity: &CpuField<f32>,
    mass: &CpuField<f32>,
    map_factor: &CpuField<f32>,
    coefficients: MomentumCouplingCoefficients<'_>,
    region: &MomentumCouplingRegion,
) -> MomentumCouplingResult<()> {
    let shape = region.shape();
    let west_east_points = shape.west_east_points();
    let bottom_top_points = shape.bottom_top_points();
    let ranges = region.west_east_momentum_ranges();
    let velocity_values = velocity.values();
    let mass_values = mass.values();
    let map_factor_values = map_factor.values();
    backend
        .try_for_each_output_block(
            output.values_mut(),
            west_east_points,
            |line_index, output_line| {
                let south_north_index = line_index / bottom_top_points;
                let bottom_top_index = line_index % bottom_top_points;
                if ranges.south_north.contains(&south_north_index)
                    && ranges.bottom_top.contains(&bottom_top_index)
                {
                    let volume_row_start = line_index * west_east_points;
                    let horizontal_row_start = south_north_index * west_east_points;
                    let west_east_range = ranges.west_east.clone();
                    couple_divided_line(
                        &mut output_line[west_east_range.clone()],
                        &velocity_values[translated_range(&west_east_range, volume_row_start)],
                        &mass_values[translated_range(&west_east_range, horizontal_row_start)],
                        &map_factor_values
                            [translated_range(&west_east_range, horizontal_row_start)],
                        coefficients.half_level_mass_multiplier[bottom_top_index],
                        coefficients.half_level_offset[bottom_top_index],
                    );
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

fn couple_south_north_momentum(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    velocity: &CpuField<f32>,
    mass: &CpuField<f32>,
    inverse_map_factor: &CpuField<f32>,
    coefficients: MomentumCouplingCoefficients<'_>,
    region: &MomentumCouplingRegion,
) -> MomentumCouplingResult<()> {
    let shape = region.shape();
    let west_east_points = shape.west_east_points();
    let bottom_top_points = shape.bottom_top_points();
    let ranges = region.south_north_momentum_ranges();
    let velocity_values = velocity.values();
    let mass_values = mass.values();
    let inverse_map_factor_values = inverse_map_factor.values();
    backend
        .try_for_each_output_block(
            output.values_mut(),
            west_east_points,
            |line_index, output_line| {
                let south_north_index = line_index / bottom_top_points;
                let bottom_top_index = line_index % bottom_top_points;
                if ranges.south_north.contains(&south_north_index)
                    && ranges.bottom_top.contains(&bottom_top_index)
                {
                    let volume_row_start = line_index * west_east_points;
                    let horizontal_row_start = south_north_index * west_east_points;
                    let west_east_range = ranges.west_east.clone();
                    couple_multiplied_line(
                        &mut output_line[west_east_range.clone()],
                        &velocity_values[translated_range(&west_east_range, volume_row_start)],
                        &mass_values[translated_range(&west_east_range, horizontal_row_start)],
                        &inverse_map_factor_values
                            [translated_range(&west_east_range, horizontal_row_start)],
                        coefficients.half_level_mass_multiplier[bottom_top_index],
                        coefficients.half_level_offset[bottom_top_index],
                    );
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

fn couple_vertical_momentum(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    velocity: &CpuField<f32>,
    mass: &CpuField<f32>,
    map_factor: &CpuField<f32>,
    coefficients: MomentumCouplingCoefficients<'_>,
    region: &MomentumCouplingRegion,
) -> MomentumCouplingResult<()> {
    let shape = region.shape();
    let west_east_points = shape.west_east_points();
    let bottom_top_points = shape.bottom_top_points();
    let ranges = region.vertical_momentum_ranges();
    let velocity_values = velocity.values();
    let mass_values = mass.values();
    let map_factor_values = map_factor.values();
    backend
        .try_for_each_output_block(
            output.values_mut(),
            west_east_points,
            |line_index, output_line| {
                let south_north_index = line_index / bottom_top_points;
                let bottom_top_index = line_index % bottom_top_points;
                if ranges.south_north.contains(&south_north_index)
                    && ranges.bottom_top.contains(&bottom_top_index)
                {
                    let west_east_range = ranges.west_east.clone();
                    couple_divided_line(
                        &mut output_line[west_east_range.clone()],
                        &velocity_values
                            [translated_range(&west_east_range, line_index * west_east_points)],
                        &mass_values[translated_range(
                            &west_east_range,
                            south_north_index * west_east_points,
                        )],
                        &map_factor_values[translated_range(
                            &west_east_range,
                            south_north_index * west_east_points,
                        )],
                        coefficients.full_level_mass_multiplier[bottom_top_index],
                        coefficients.full_level_offset[bottom_top_index],
                    );
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

fn translated_range(range: &std::ops::Range<usize>, offset: usize) -> std::ops::Range<usize> {
    (offset + range.start)..(offset + range.end)
}

fn couple_divided_line(
    output: &mut [f32],
    velocity: &[f32],
    mass: &[f32],
    map_factor: &[f32],
    mass_multiplier: f32,
    offset: f32,
) {
    for (((output_value, velocity_value), mass_value), map_factor_value) in
        output.iter_mut().zip(velocity).zip(mass).zip(map_factor)
    {
        *output_value =
            *velocity_value * (mass_multiplier * *mass_value + offset) / *map_factor_value;
    }
}

fn couple_multiplied_line(
    output: &mut [f32],
    velocity: &[f32],
    mass: &[f32],
    inverse_map_factor: &[f32],
    mass_multiplier: f32,
    offset: f32,
) {
    for (((output_value, velocity_value), mass_value), inverse_map_factor_value) in output
        .iter_mut()
        .zip(velocity)
        .zip(mass)
        .zip(inverse_map_factor)
    {
        *output_value =
            *velocity_value * (mass_multiplier * *mass_value + offset) * *inverse_map_factor_value;
    }
}

fn map_parallel_error(error: ParallelExecutionError<Infallible>) -> MomentumCouplingError {
    match error {
        ParallelExecutionError::Kernel(never) => match never {},
        ParallelExecutionError::WorkerPanicked => MomentumCouplingError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. } => {
            unreachable!("validated field shapes produce complete non-empty momentum rows")
        }
    }
}

#[cfg(test)]
mod tests {
    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;

    #[test]
    fn matches_all_upstream_fortran_clipping_and_exceptional_bits() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();

        for case in oracle_cases() {
            let (mut fixture, region) = create_oracle_fixture(&backend, case);
            apply_fixture(&backend, &mut fixture, &region).unwrap();

            for (field_name, field) in [
                ("west_east", &fixture.west_east_output),
                ("south_north", &fixture.south_north_output),
                ("vertical", &fixture.vertical_output),
            ] {
                assert_eq!(
                    field_bits(field),
                    expected_fortran_bits(case.name, field_name),
                    "{field_name} differs for {}",
                    case.name
                );
            }

            if case.exceptional {
                let extreme_index = volume_index(1, 1, 1, fixture.volume_shape);
                let zero_map_index = volume_index(2, 1, 1, fixture.volume_shape);
                assert!(fixture.west_east_output.values()[extreme_index].is_infinite());
                assert!(fixture.south_north_output.values()[extreme_index].is_infinite());
                assert!(fixture.vertical_output.values()[extreme_index].is_infinite());
                assert!(fixture.west_east_output.values()[zero_map_index].is_infinite());
                assert_eq!(fixture.south_north_output.values()[zero_map_index], 0.0);
                assert!(fixture.vertical_output.values()[zero_map_index].is_infinite());
            }
        }
    }

    #[test]
    fn is_bitwise_deterministic_across_worker_counts() {
        let single_worker_backend = CpuBackend::try_with_worker_count(1).unwrap();
        let four_worker_backend = CpuBackend::try_with_worker_count(4).unwrap();
        let case = oracle_cases()[4];
        let (mut single_worker_fixture, region) =
            create_oracle_fixture(&single_worker_backend, case);
        let (mut four_worker_fixture, _) = create_oracle_fixture(&four_worker_backend, case);

        apply_fixture(&single_worker_backend, &mut single_worker_fixture, &region).unwrap();
        apply_fixture(&four_worker_backend, &mut four_worker_fixture, &region).unwrap();

        assert_eq!(
            single_worker_fixture.west_east_output,
            four_worker_fixture.west_east_output
        );
        assert_eq!(
            single_worker_fixture.south_north_output,
            four_worker_fixture.south_north_output
        );
        assert_eq!(
            single_worker_fixture.vertical_output,
            four_worker_fixture.vertical_output
        );
    }

    #[test]
    fn rejects_a_field_shape_mismatch_before_mutating_any_output() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = oracle_cases()[4];
        let (mut fixture, region) = create_oracle_fixture(&backend, case);
        let wrong_shape = GridShape::try_new(8, 6, 1).unwrap();
        fixture.west_east_mass = backend.create_field(wrong_shape, 1.0).unwrap();

        assert_eq!(
            apply_fixture(&backend, &mut fixture, &region),
            Err(MomentumCouplingError::FieldShapeMismatch {
                field: MomentumCouplingField::WestEastMomentumMass,
                expected: region.shape().horizontal_shape(),
                actual: wrong_shape,
            })
        );
        assert_outputs_retain_sentinel(&fixture);
    }

    #[test]
    fn rejects_a_coefficient_length_mismatch_before_mutating_any_output() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = oracle_cases()[4];
        let (mut fixture, region) = create_oracle_fixture(&backend, case);
        fixture.full_level_offset.pop();

        assert_eq!(
            apply_fixture(&backend, &mut fixture, &region),
            Err(MomentumCouplingError::CoefficientLengthMismatch {
                coefficient: MomentumCouplingCoefficient::FullLevelOffset,
                expected: 5,
                actual: 4,
            })
        );
        assert_outputs_retain_sentinel(&fixture);
    }

    #[test]
    fn validates_every_field_and_coefficient_role() {
        let backend = CpuBackend::try_with_worker_count(1).unwrap();
        let expected_shape = GridShape::try_new(5, 5, 5).unwrap();
        let actual_shape = GridShape::try_new(4, 5, 5).unwrap();
        let field = backend.create_field(actual_shape, 0.0).unwrap();
        let field_roles = [
            MomentumCouplingField::WestEastMomentumOutput,
            MomentumCouplingField::SouthNorthMomentumOutput,
            MomentumCouplingField::VerticalMomentumOutput,
            MomentumCouplingField::WestEastVelocity,
            MomentumCouplingField::SouthNorthVelocity,
            MomentumCouplingField::VerticalVelocity,
            MomentumCouplingField::WestEastMomentumMass,
            MomentumCouplingField::SouthNorthMomentumMass,
            MomentumCouplingField::MassPointColumnMass,
            MomentumCouplingField::WestEastMapFactor,
            MomentumCouplingField::InverseSouthNorthMapFactor,
            MomentumCouplingField::MassPointMapFactor,
        ];

        for role in field_roles {
            assert_eq!(
                validate_field_shape(&field, role, expected_shape),
                Err(MomentumCouplingError::FieldShapeMismatch {
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
                MomentumCouplingCoefficients::new(&short, &complete, &complete, &complete),
                MomentumCouplingCoefficient::HalfLevelMassMultiplier,
            ),
            (
                MomentumCouplingCoefficients::new(&complete, &short, &complete, &complete),
                MomentumCouplingCoefficient::HalfLevelOffset,
            ),
            (
                MomentumCouplingCoefficients::new(&complete, &complete, &short, &complete),
                MomentumCouplingCoefficient::FullLevelMassMultiplier,
            ),
            (
                MomentumCouplingCoefficients::new(&complete, &complete, &complete, &short),
                MomentumCouplingCoefficient::FullLevelOffset,
            ),
        ];

        for (coefficients, expected_role) in cases {
            assert_eq!(
                validate_coefficients(coefficients, 5),
                Err(MomentumCouplingError::CoefficientLengthMismatch {
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
        bottom_top_tile: (i32, i32),
        exceptional: bool,
    }

    fn oracle_cases() -> [OracleCase; 6] {
        [
            OracleCase {
                name: "interior",
                west_east_tile: (0, 1),
                south_north_tile: (5, 6),
                bottom_top_tile: (1, 3),
                exceptional: false,
            },
            OracleCase {
                name: "x_upper",
                west_east_tile: (-1, 3),
                south_north_tile: (5, 6),
                bottom_top_tile: (1, 3),
                exceptional: false,
            },
            OracleCase {
                name: "y_upper",
                west_east_tile: (0, 1),
                south_north_tile: (4, 7),
                bottom_top_tile: (1, 3),
                exceptional: false,
            },
            OracleCase {
                name: "z_upper",
                west_east_tile: (0, 1),
                south_north_tile: (5, 6),
                bottom_top_tile: (1, 4),
                exceptional: false,
            },
            OracleCase {
                name: "all_upper",
                west_east_tile: (-1, 3),
                south_north_tile: (4, 7),
                bottom_top_tile: (1, 4),
                exceptional: false,
            },
            OracleCase {
                name: "exceptional_finite",
                west_east_tile: (-1, 3),
                south_north_tile: (4, 7),
                bottom_top_tile: (1, 4),
                exceptional: true,
            },
        ]
    }

    struct MomentumCouplingFixture {
        volume_shape: GridShape,
        west_east_output: CpuField<f32>,
        south_north_output: CpuField<f32>,
        vertical_output: CpuField<f32>,
        west_east_velocity: CpuField<f32>,
        south_north_velocity: CpuField<f32>,
        vertical_velocity: CpuField<f32>,
        west_east_mass: CpuField<f32>,
        south_north_mass: CpuField<f32>,
        mass_point_mass: CpuField<f32>,
        west_east_map_factor: CpuField<f32>,
        inverse_south_north_map_factor: CpuField<f32>,
        mass_point_map_factor: CpuField<f32>,
        half_level_mass_multiplier: Vec<f32>,
        half_level_offset: Vec<f32>,
        full_level_mass_multiplier: Vec<f32>,
        full_level_offset: Vec<f32>,
    }

    fn create_oracle_fixture(
        backend: &CpuBackend,
        case: OracleCase,
    ) -> (MomentumCouplingFixture, MomentumCouplingRegion) {
        let volume_shape = GridShape::try_new(7, 6, 5).unwrap();
        let horizontal_shape = volume_shape.horizontal_shape();
        let mut fixture = MomentumCouplingFixture {
            volume_shape,
            west_east_output: backend.create_field(volume_shape, -999.0).unwrap(),
            south_north_output: backend.create_field(volume_shape, -999.0).unwrap(),
            vertical_output: backend.create_field(volume_shape, -999.0).unwrap(),
            west_east_velocity: backend.create_field(volume_shape, 0.0).unwrap(),
            south_north_velocity: backend.create_field(volume_shape, 0.0).unwrap(),
            vertical_velocity: backend.create_field(volume_shape, 0.0).unwrap(),
            west_east_mass: backend.create_field(horizontal_shape, 0.0).unwrap(),
            south_north_mass: backend.create_field(horizontal_shape, 0.0).unwrap(),
            mass_point_mass: backend.create_field(horizontal_shape, 0.0).unwrap(),
            west_east_map_factor: backend.create_field(horizontal_shape, 0.0).unwrap(),
            inverse_south_north_map_factor: backend.create_field(horizontal_shape, 0.0).unwrap(),
            mass_point_map_factor: backend.create_field(horizontal_shape, 0.0).unwrap(),
            half_level_mass_multiplier: vec![0.0; 5],
            half_level_offset: vec![0.0; 5],
            full_level_mass_multiplier: vec![0.0; 5],
            full_level_offset: vec![0.0; 5],
        };
        initialize_fixture(&mut fixture);
        if case.exceptional {
            apply_exceptional_inputs(&mut fixture);
        }
        let region = MomentumCouplingRegion::try_new(
            volume_shape,
            1..5,
            1..4,
            1..4,
            inclusive_fortran_range(case.west_east_tile, -2),
            inclusive_fortran_range(case.south_north_tile, 3),
            inclusive_fortran_range(case.bottom_top_tile, 0),
        )
        .unwrap();
        (fixture, region)
    }

    fn initialize_fixture(fixture: &mut MomentumCouplingFixture) {
        for bottom_top_index in 0..5 {
            let bottom_top = bottom_top_index as f32;
            fixture.half_level_mass_multiplier[bottom_top_index] = 0.7 + bottom_top * 0.03;
            fixture.half_level_offset[bottom_top_index] = 1.5 - bottom_top * 0.1;
            fixture.full_level_mass_multiplier[bottom_top_index] = 0.6 + bottom_top * 0.02;
            fixture.full_level_offset[bottom_top_index] = 2.0 + bottom_top * 0.15;
        }
        for south_north_index in 0..6 {
            let south_north = (south_north_index as i32 + 3) as f32;
            for west_east_index in 0..7 {
                let west_east = (west_east_index as i32 - 2) as f32;
                let horizontal_index = south_north_index * 7 + west_east_index;
                fixture.west_east_mass.values_mut()[horizontal_index] =
                    80.0 + west_east * 0.5 + south_north * 1.25;
                fixture.south_north_mass.values_mut()[horizontal_index] =
                    85.0 + west_east * 0.75 - south_north * 0.5;
                fixture.mass_point_mass.values_mut()[horizontal_index] =
                    90.0 + west_east * 0.25 + south_north * 0.8;
                fixture.west_east_map_factor.values_mut()[horizontal_index] =
                    1.0 + west_east_index as f32 * 0.01 + south_north_index as f32 * 0.005;
                fixture.inverse_south_north_map_factor.values_mut()[horizontal_index] =
                    1.0 / (1.1 + west_east_index as f32 * 0.008 + south_north_index as f32 * 0.004);
                fixture.mass_point_map_factor.values_mut()[horizontal_index] =
                    0.9 + west_east_index as f32 * 0.006 + south_north_index as f32 * 0.003;
                for bottom_top_index in 0..5 {
                    let bottom_top = bottom_top_index as f32;
                    let index = volume_index(
                        west_east_index,
                        bottom_top_index,
                        south_north_index,
                        fixture.volume_shape,
                    );
                    fixture.west_east_velocity.values_mut()[index] =
                        -3.0 + west_east * 0.2 + bottom_top * 0.3 + south_north * 0.1;
                    fixture.south_north_velocity.values_mut()[index] =
                        2.0 - west_east * 0.15 + bottom_top * 0.25 - south_north * 0.05;
                    fixture.vertical_velocity.values_mut()[index] =
                        0.5 + west_east * 0.08 - bottom_top * 0.12 + south_north * 0.07;
                }
            }
        }
    }

    fn apply_exceptional_inputs(fixture: &mut MomentumCouplingFixture) {
        let extreme_index = volume_index(1, 1, 1, fixture.volume_shape);
        fixture.west_east_velocity.values_mut()[extreme_index] = f32::MAX;
        fixture.south_north_velocity.values_mut()[extreme_index] = f32::MAX;
        fixture.vertical_velocity.values_mut()[extreme_index] = f32::MAX;
        fixture.half_level_mass_multiplier[1] = 2.0;
        fixture.full_level_mass_multiplier[1] = 2.0;
        let zero_map_index = 9;
        fixture.west_east_map_factor.values_mut()[zero_map_index] = 0.0;
        fixture.inverse_south_north_map_factor.values_mut()[zero_map_index] = 0.0;
        fixture.mass_point_map_factor.values_mut()[zero_map_index] = 0.0;
    }

    fn apply_fixture(
        backend: &CpuBackend,
        fixture: &mut MomentumCouplingFixture,
        region: &MomentumCouplingRegion,
    ) -> MomentumCouplingResult<()> {
        backend.couple_momentum(
            MomentumCouplingOutputs::new(
                &mut fixture.west_east_output,
                &mut fixture.south_north_output,
                &mut fixture.vertical_output,
            ),
            MomentumCouplingVelocities::new(
                &fixture.west_east_velocity,
                &fixture.south_north_velocity,
                &fixture.vertical_velocity,
            ),
            MomentumCouplingMasses::new(
                &fixture.west_east_mass,
                &fixture.south_north_mass,
                &fixture.mass_point_mass,
            ),
            MomentumCouplingMapFactors::new(
                &fixture.west_east_map_factor,
                &fixture.inverse_south_north_map_factor,
                &fixture.mass_point_map_factor,
            ),
            MomentumCouplingCoefficients::new(
                &fixture.half_level_mass_multiplier,
                &fixture.half_level_offset,
                &fixture.full_level_mass_multiplier,
                &fixture.full_level_offset,
            ),
            region,
        )
    }

    fn assert_outputs_retain_sentinel(fixture: &MomentumCouplingFixture) {
        for field in [
            &fixture.west_east_output,
            &fixture.south_north_output,
            &fixture.vertical_output,
        ] {
            assert!(field.values().iter().all(|value| *value == -999.0));
        }
    }

    fn volume_index(
        west_east: usize,
        bottom_top: usize,
        south_north: usize,
        shape: GridShape,
    ) -> usize {
        ((south_north * shape.bottom_top_points() + bottom_top) * shape.west_east_points())
            + west_east
    }

    fn inclusive_fortran_range(bounds: (i32, i32), memory_start: i32) -> std::ops::Range<usize> {
        ((bounds.0 - memory_start) as usize)..((bounds.1 - memory_start + 1) as usize)
    }

    fn field_bits(field: &CpuField<f32>) -> Vec<u32> {
        field.values().iter().map(|value| value.to_bits()).collect()
    }

    fn expected_fortran_bits(case_name: &str, field_name: &str) -> Vec<u32> {
        let bits: Vec<_> = include_str!("../../test-data/momentum_coupling.out.correct")
            .lines()
            .filter_map(|line| {
                let mut columns = line.split_whitespace();
                if columns.next().unwrap() != case_name || columns.next().unwrap() != field_name {
                    return None;
                }
                columns.next();
                columns.next();
                columns.next();
                Some(u32::from_str_radix(columns.next().unwrap(), 16).unwrap())
            })
            .collect();
        assert_eq!(
            bits.len(),
            210,
            "incomplete {case_name} {field_name} oracle"
        );
        bits
    }
}
