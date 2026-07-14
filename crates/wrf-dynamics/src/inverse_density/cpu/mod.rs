use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, GridShape, ParallelExecutionError};

use crate::{
    InverseDensityError, InverseDensityField, InverseDensityKernels, InverseDensityRegion,
    InverseDensityResult,
};

impl InverseDensityKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn calculate_full_inverse_density(
        &self,
        full_inverse_density: &mut Self::Field,
        perturbation_inverse_density: &Self::Field,
        base_state_inverse_density: &Self::Field,
        region: &InverseDensityRegion,
    ) -> InverseDensityResult<()> {
        validate_shape(
            full_inverse_density,
            InverseDensityField::FullOutput,
            region.shape(),
        )?;
        validate_shape(
            perturbation_inverse_density,
            InverseDensityField::Perturbation,
            region.shape(),
        )?;
        validate_shape(
            base_state_inverse_density,
            InverseDensityField::BaseState,
            region.shape(),
        )?;

        calculate_rows(
            self,
            full_inverse_density,
            perturbation_inverse_density,
            base_state_inverse_density,
            region,
        )
    }
}

fn calculate_rows(
    backend: &CpuBackend,
    full_inverse_density: &mut CpuField<f32>,
    perturbation_inverse_density: &CpuField<f32>,
    base_state_inverse_density: &CpuField<f32>,
    region: &InverseDensityRegion,
) -> InverseDensityResult<()> {
    let shape = region.shape();
    let west_east_points = shape.west_east_points();
    let bottom_top_points = shape.bottom_top_points();
    let perturbation_values = perturbation_inverse_density.values();
    let base_state_values = base_state_inverse_density.values();
    let (west_east, south_north, bottom_top) = region.output_ranges();

    backend
        .try_for_each_output_block(
            full_inverse_density.values_mut(),
            west_east_points,
            |row_index, output_row| {
                let south_north_index = row_index / bottom_top_points;
                let bottom_top_index = row_index % bottom_top_points;
                if south_north.contains(&south_north_index)
                    && bottom_top.contains(&bottom_top_index)
                {
                    let row_start = row_index * west_east_points;
                    let input_range = row_start + west_east.start..row_start + west_east.end;
                    let output_values = &mut output_row[west_east.clone()];
                    let perturbation_values = &perturbation_values[input_range.clone()];
                    let base_state_values = &base_state_values[input_range];

                    for ((output, perturbation), base_state) in output_values
                        .iter_mut()
                        .zip(perturbation_values)
                        .zip(base_state_values)
                    {
                        *output = *perturbation + *base_state;
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

fn validate_shape(
    field: &CpuField<f32>,
    role: InverseDensityField,
    expected: GridShape,
) -> InverseDensityResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(InverseDensityError::FieldShapeMismatch {
            field: role,
            expected,
            actual,
        });
    }
    Ok(())
}

fn map_parallel_error(error: ParallelExecutionError<Infallible>) -> InverseDensityError {
    match error {
        ParallelExecutionError::Kernel(never) => match never {},
        ParallelExecutionError::WorkerPanicked => InverseDensityError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. } => {
            unreachable!("validated field shapes produce complete non-empty inverse-density rows")
        }
    }
}

#[cfg(test)]
mod oracle_tests;

#[cfg(test)]
mod tests {
    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;

    #[test]
    fn is_bitwise_deterministic_across_worker_counts() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();
        let shape = GridShape::try_new(9, 8, 7).unwrap();
        let region =
            InverseDensityRegion::try_new(shape, 1..8, 1..7, 1..6, 1..9, 1..8, 1..7).unwrap();
        let perturbation = create_pattern(&one_worker, shape, 0.25);
        let base_state = create_pattern(&one_worker, shape, 0.75);
        let mut single_output = one_worker.create_field(shape, -999.0).unwrap();
        let mut parallel_output = four_workers.create_field(shape, -999.0).unwrap();

        one_worker
            .calculate_full_inverse_density(&mut single_output, &perturbation, &base_state, &region)
            .unwrap();
        four_workers
            .calculate_full_inverse_density(
                &mut parallel_output,
                &perturbation,
                &base_state,
                &region,
            )
            .unwrap();

        assert_eq!(single_output, parallel_output);
    }

    #[test]
    fn rejects_each_field_role_before_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let expected = GridShape::try_new(5, 5, 5).unwrap();
        let wrong = GridShape::try_new(4, 5, 5).unwrap();
        let region =
            InverseDensityRegion::try_new(expected, 1..4, 1..4, 1..4, 1..5, 1..5, 1..5).unwrap();

        for role in [
            InverseDensityField::FullOutput,
            InverseDensityField::Perturbation,
            InverseDensityField::BaseState,
        ] {
            let mut output = backend.create_field(expected, -999.0).unwrap();
            let valid = backend.create_field(expected, 1.0).unwrap();
            let invalid = backend.create_field(wrong, 1.0).unwrap();
            let result = match role {
                InverseDensityField::FullOutput => {
                    let mut invalid_output = backend.create_field(wrong, -999.0).unwrap();
                    backend.calculate_full_inverse_density(
                        &mut invalid_output,
                        &valid,
                        &valid,
                        &region,
                    )
                }
                InverseDensityField::Perturbation => {
                    backend.calculate_full_inverse_density(&mut output, &invalid, &valid, &region)
                }
                InverseDensityField::BaseState => {
                    backend.calculate_full_inverse_density(&mut output, &valid, &invalid, &region)
                }
            };

            assert_eq!(
                result,
                Err(InverseDensityError::FieldShapeMismatch {
                    field: role,
                    expected,
                    actual: wrong,
                })
            );
            assert!(output.values().iter().all(|value| *value == -999.0));
        }
    }

    fn create_pattern(backend: &CpuBackend, shape: GridShape, offset: f32) -> CpuField<f32> {
        let mut field = backend.create_field(shape, 0.0).unwrap();
        for (index, value) in field.values_mut().iter_mut().enumerate() {
            *value = offset + index as f32 * 0.000_1;
        }
        field
    }
}
