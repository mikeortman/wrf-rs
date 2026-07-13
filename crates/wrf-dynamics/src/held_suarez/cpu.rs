use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, GridShape, ParallelExecutionError};

use crate::{
    HeldSuarezDampingError, HeldSuarezDampingField, HeldSuarezDampingFields,
    HeldSuarezDampingKernels, HeldSuarezDampingRegion, HeldSuarezDampingResult,
};

const SIGMA_BOUNDARY: f32 = 0.7;
const DAY_LENGTH_SECONDS: f32 = 60.0 * 60.0 * 24.0;
const FRICTION_RATE: f32 = 1.0 / DAY_LENGTH_SECONDS;

impl HeldSuarezDampingKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn apply_held_suarez_damping(
        &self,
        fields: HeldSuarezDampingFields<'_, Self::Field>,
        region: &HeldSuarezDampingRegion,
    ) -> HeldSuarezDampingResult<()> {
        let HeldSuarezDampingFields {
            west_east_momentum_tendency,
            south_north_momentum_tendency,
            west_east_momentum,
            south_north_momentum,
            perturbation_pressure,
            base_pressure,
        } = fields;
        let expected_shape = region.shape();
        validate_field_shape(
            west_east_momentum_tendency,
            HeldSuarezDampingField::WestEastMomentumTendency,
            expected_shape,
        )?;
        validate_field_shape(
            south_north_momentum_tendency,
            HeldSuarezDampingField::SouthNorthMomentumTendency,
            expected_shape,
        )?;
        validate_field_shape(
            west_east_momentum,
            HeldSuarezDampingField::WestEastMomentum,
            expected_shape,
        )?;
        validate_field_shape(
            south_north_momentum,
            HeldSuarezDampingField::SouthNorthMomentum,
            expected_shape,
        )?;
        validate_field_shape(
            perturbation_pressure,
            HeldSuarezDampingField::PerturbationPressure,
            expected_shape,
        )?;
        validate_field_shape(
            base_pressure,
            HeldSuarezDampingField::BasePressure,
            expected_shape,
        )?;

        let west_east_points = expected_shape.west_east_points();
        let bottom_top_points = expected_shape.bottom_top_points();
        let west_east_range = region.west_east_range();
        let bottom_top_range = region.bottom_top_range();
        let surface_level = region.surface_level();
        let perturbation_pressure = perturbation_pressure.values();
        let base_pressure = base_pressure.values();

        let south_north_range = region.south_north_momentum_south_north_range();
        let south_north_momentum = south_north_momentum.values();
        self.try_for_each_output_block(
            south_north_momentum_tendency.values_mut(),
            west_east_points,
            |line_index, tendency_line| {
                let south_north_index = line_index / bottom_top_points;
                let bottom_top_index = line_index % bottom_top_points;
                if south_north_range.contains(&south_north_index)
                    && bottom_top_range.contains(&bottom_top_index)
                {
                    for west_east_index in west_east_range.clone() {
                        let current_index = linear_index(
                            west_east_index,
                            bottom_top_index,
                            south_north_index,
                            west_east_points,
                            bottom_top_points,
                        );
                        let preceding_index = linear_index(
                            west_east_index,
                            bottom_top_index,
                            south_north_index - 1,
                            west_east_points,
                            bottom_top_points,
                        );
                        let current_surface_index = linear_index(
                            west_east_index,
                            surface_level,
                            south_north_index,
                            west_east_points,
                            bottom_top_points,
                        );
                        let preceding_surface_index = linear_index(
                            west_east_index,
                            surface_level,
                            south_north_index - 1,
                            west_east_points,
                            bottom_top_points,
                        );
                        let sigma = (perturbation_pressure[preceding_index]
                            + base_pressure[preceding_index]
                            + perturbation_pressure[current_index]
                            + base_pressure[current_index])
                            / (perturbation_pressure[preceding_surface_index]
                                + base_pressure[preceding_surface_index]
                                + perturbation_pressure[current_surface_index]
                                + base_pressure[current_surface_index]);
                        let sigma_term =
                            0.0_f32.max((sigma - SIGMA_BOUNDARY) / (1.0 - SIGMA_BOUNDARY));
                        let vertical_damping = FRICTION_RATE * sigma_term;
                        tendency_line[west_east_index] -=
                            vertical_damping * south_north_momentum[current_index];
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)?;

        let south_north_range = region.west_east_momentum_south_north_range();
        let west_east_momentum = west_east_momentum.values();
        self.try_for_each_output_block(
            west_east_momentum_tendency.values_mut(),
            west_east_points,
            |line_index, tendency_line| {
                let south_north_index = line_index / bottom_top_points;
                let bottom_top_index = line_index % bottom_top_points;
                if south_north_range.contains(&south_north_index)
                    && bottom_top_range.contains(&bottom_top_index)
                {
                    for west_east_index in west_east_range.clone() {
                        let current_index = linear_index(
                            west_east_index,
                            bottom_top_index,
                            south_north_index,
                            west_east_points,
                            bottom_top_points,
                        );
                        let preceding_index = linear_index(
                            west_east_index - 1,
                            bottom_top_index,
                            south_north_index,
                            west_east_points,
                            bottom_top_points,
                        );
                        let current_surface_index = linear_index(
                            west_east_index,
                            surface_level,
                            south_north_index,
                            west_east_points,
                            bottom_top_points,
                        );
                        let preceding_surface_index = linear_index(
                            west_east_index - 1,
                            surface_level,
                            south_north_index,
                            west_east_points,
                            bottom_top_points,
                        );
                        let sigma = (perturbation_pressure[preceding_index]
                            + base_pressure[preceding_index]
                            + perturbation_pressure[current_index]
                            + base_pressure[current_index])
                            / (perturbation_pressure[preceding_surface_index]
                                + base_pressure[preceding_surface_index]
                                + perturbation_pressure[current_surface_index]
                                + base_pressure[current_surface_index]);
                        let sigma_term =
                            0.0_f32.max((sigma - SIGMA_BOUNDARY) / (1.0 - SIGMA_BOUNDARY));
                        let vertical_damping = FRICTION_RATE * sigma_term;
                        tendency_line[west_east_index] -=
                            vertical_damping * west_east_momentum[current_index];
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
    }
}

fn validate_field_shape(
    field: &CpuField<f32>,
    field_name: HeldSuarezDampingField,
    expected: GridShape,
) -> HeldSuarezDampingResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(HeldSuarezDampingError::FieldShapeMismatch {
            field: field_name,
            expected,
            actual,
        });
    }
    Ok(())
}

const fn linear_index(
    west_east_index: usize,
    bottom_top_index: usize,
    south_north_index: usize,
    west_east_points: usize,
    bottom_top_points: usize,
) -> usize {
    (south_north_index * bottom_top_points + bottom_top_index) * west_east_points + west_east_index
}

fn map_parallel_error(error: ParallelExecutionError<Infallible>) -> HeldSuarezDampingError {
    match error {
        ParallelExecutionError::Kernel(never) => match never {},
        ParallelExecutionError::WorkerPanicked => HeldSuarezDampingError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. } => {
            unreachable!("validated field shapes always produce complete non-empty lines")
        }
    }
}

#[cfg(test)]
mod tests {
    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;

    #[test]
    fn matches_upstream_fortran_boundary_and_active_point_bits() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let (mut fields, region) = create_fortran_fixture(&backend);

        apply_fixture(&backend, &mut fields, &region).unwrap();

        let actual_bits = selected_fortran_fixture_bits(&fields);
        assert_eq!(actual_bits, expected_fortran_bits());
    }

    #[test]
    fn is_bitwise_deterministic_across_worker_counts() {
        let single_worker_backend = CpuBackend::try_with_worker_count(1).unwrap();
        let four_worker_backend = CpuBackend::try_with_worker_count(4).unwrap();
        let (mut single_worker_fields, region) = create_fortran_fixture(&single_worker_backend);
        let (mut four_worker_fields, _) = create_fortran_fixture(&four_worker_backend);

        apply_fixture(&single_worker_backend, &mut single_worker_fields, &region).unwrap();
        apply_fixture(&four_worker_backend, &mut four_worker_fields, &region).unwrap();

        assert_eq!(
            single_worker_fields.west_east_momentum_tendency.values(),
            four_worker_fields.west_east_momentum_tendency.values()
        );
        assert_eq!(
            single_worker_fields.south_north_momentum_tendency.values(),
            four_worker_fields.south_north_momentum_tendency.values()
        );
    }

    #[test]
    fn rejects_a_field_shape_mismatch_before_mutating_outputs() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let (mut fields, region) = create_fortran_fixture(&backend);
        let original_west_east_tendency = fields.west_east_momentum_tendency.values().to_vec();
        let wrong_shape = GridShape::try_new(5, 6, 4).unwrap();
        fields.base_pressure = backend.create_field(wrong_shape, 0.0).unwrap();

        let HeldSuarezFixture {
            ref mut west_east_momentum_tendency,
            ref mut south_north_momentum_tendency,
            ref west_east_momentum,
            ref south_north_momentum,
            ref perturbation_pressure,
            ref base_pressure,
        } = fields;
        let result = backend.apply_held_suarez_damping(
            HeldSuarezDampingFields::new(
                west_east_momentum_tendency,
                south_north_momentum_tendency,
                west_east_momentum,
                south_north_momentum,
                perturbation_pressure,
                base_pressure,
            ),
            &region,
        );

        assert_eq!(
            result,
            Err(HeldSuarezDampingError::FieldShapeMismatch {
                field: HeldSuarezDampingField::BasePressure,
                expected: region.shape(),
                actual: wrong_shape,
            })
        );
        assert_eq!(
            west_east_momentum_tendency.values(),
            original_west_east_tendency
        );
    }

    struct HeldSuarezFixture {
        west_east_momentum_tendency: CpuField<f32>,
        south_north_momentum_tendency: CpuField<f32>,
        west_east_momentum: CpuField<f32>,
        south_north_momentum: CpuField<f32>,
        perturbation_pressure: CpuField<f32>,
        base_pressure: CpuField<f32>,
    }

    fn create_fortran_fixture(
        backend: &CpuBackend,
    ) -> (HeldSuarezFixture, HeldSuarezDampingRegion) {
        let shape = GridShape::try_new(6, 6, 4).unwrap();
        let mut fields = HeldSuarezFixture {
            west_east_momentum_tendency: backend.create_field(shape, 0.0).unwrap(),
            south_north_momentum_tendency: backend.create_field(shape, 0.0).unwrap(),
            west_east_momentum: backend.create_field(shape, 0.0).unwrap(),
            south_north_momentum: backend.create_field(shape, 0.0).unwrap(),
            perturbation_pressure: backend.create_field(shape, 0.0).unwrap(),
            base_pressure: backend.create_field(shape, 0.0).unwrap(),
        };

        for south_north_index in 0..shape.south_north_points() {
            let fortran_j = south_north_index as i32 - 1;
            for bottom_top_index in 0..shape.bottom_top_points() {
                let fortran_k = bottom_top_index as i32;
                for west_east_index in 0..shape.west_east_points() {
                    let fortran_i = west_east_index as i32 - 1;
                    let index = linear_index(
                        west_east_index,
                        bottom_top_index,
                        south_north_index,
                        shape.west_east_points(),
                        shape.bottom_top_points(),
                    );
                    fields.perturbation_pressure.values_mut()[index] =
                        (10 * fortran_i + 3 * fortran_j + 2 * fortran_k) as f32;
                    fields.base_pressure.values_mut()[index] = match bottom_top_index {
                        0 => 110_000.0,
                        1 => 100_000.0,
                        2 => 80_000.0,
                        _ => 50_000.0,
                    };
                    fields.west_east_momentum.values_mut()[index] =
                        (2 * fortran_i + 3 * fortran_k + 5 * fortran_j) as f32;
                    fields.south_north_momentum.values_mut()[index] =
                        (-fortran_i + 4 * fortran_k + 2 * fortran_j) as f32;
                    fields.west_east_momentum_tendency.values_mut()[index] =
                        (100 + fortran_i + 2 * fortran_k + 3 * fortran_j) as f32;
                    fields.south_north_momentum_tendency.values_mut()[index] =
                        (200 + 2 * fortran_i + fortran_k + 4 * fortran_j) as f32;
                }
            }
        }

        let region = HeldSuarezDampingRegion::try_new(shape, 1..5, 1..4, 1..5, 2..5, 1).unwrap();
        (fields, region)
    }

    fn apply_fixture(
        backend: &CpuBackend,
        fields: &mut HeldSuarezFixture,
        region: &HeldSuarezDampingRegion,
    ) -> HeldSuarezDampingResult<()> {
        backend.apply_held_suarez_damping(
            HeldSuarezDampingFields::new(
                &mut fields.west_east_momentum_tendency,
                &mut fields.south_north_momentum_tendency,
                &fields.west_east_momentum,
                &fields.south_north_momentum,
                &fields.perturbation_pressure,
                &fields.base_pressure,
            ),
            region,
        )
    }

    fn selected_fortran_fixture_bits(fields: &HeldSuarezFixture) -> Vec<u32> {
        let west_east_points = 6;
        let bottom_top_points = 4;
        let selected_west_east_tendency_points = [
            (0, 1, 0),
            (0, 2, 0),
            (3, 3, 3),
            (4, 1, 0),
            (0, 0, 0),
            (0, 1, 4),
            (2, 2, 2),
            (3, 1, 3),
        ];
        let selected_south_north_tendency_points = [
            (0, 1, 1),
            (0, 2, 1),
            (3, 3, 3),
            (0, 1, 0),
            (0, 1, 4),
            (4, 1, 1),
            (0, 0, 1),
            (2, 2, 2),
        ];
        selected_west_east_tendency_points
            .into_iter()
            .map(|point| fortran_point_index(point, west_east_points, bottom_top_points))
            .map(|index| fields.west_east_momentum_tendency.values()[index].to_bits())
            .chain(
                selected_south_north_tendency_points
                    .into_iter()
                    .map(|point| fortran_point_index(point, west_east_points, bottom_top_points))
                    .map(|index| fields.south_north_momentum_tendency.values()[index].to_bits()),
            )
            .collect()
    }

    fn fortran_point_index(
        (fortran_i, fortran_k, fortran_j): (i32, usize, i32),
        west_east_points: usize,
        bottom_top_points: usize,
    ) -> usize {
        linear_index(
            (fortran_i + 1) as usize,
            fortran_k,
            (fortran_j + 1) as usize,
            west_east_points,
            bottom_top_points,
        )
    }

    fn expected_fortran_bits() -> Vec<u32> {
        include_str!("../../test-data/held_suarez_damp.out.correct")
            .split_whitespace()
            .skip(1)
            .map(|value| u32::from_str_radix(value, 16).unwrap())
            .collect()
    }
}
