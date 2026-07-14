use std::convert::Infallible;
use std::ops::Range;

use wrf_compute::{CpuBackend, CpuField, FieldStorage};

use crate::{
    AcousticStepPreparationColumnMassTimeLevels, AcousticStepPreparationMassInputs,
    AcousticStepPreparationMassOutputs, AcousticStepPreparationPhase,
    AcousticStepPreparationRegion, AcousticStepPreparationResult,
    AcousticStepPreparationSavedOutputs,
};

use super::map_parallel_error;

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare_column_masses(
    backend: &CpuBackend,
    time_levels: &mut AcousticStepPreparationColumnMassTimeLevels<'_, CpuField<f32>>,
    saved: &mut AcousticStepPreparationSavedOutputs<'_, CpuField<f32>>,
    outputs: &mut AcousticStepPreparationMassOutputs<'_, CpuField<f32>>,
    inputs: &AcousticStepPreparationMassInputs<'_, CpuField<f32>>,
    phase: AcousticStepPreparationPhase,
    region: &AcousticStepPreparationRegion,
) -> AcousticStepPreparationResult<()> {
    let active_x = region.active_west_east();
    let active_y = region.active_south_north();
    let u_x = region.west_east_velocity_range();
    let v_y = region.south_north_velocity_range();

    if phase.switches_time_levels() {
        copy_horizontal(
            backend,
            time_levels.previous,
            time_levels.current,
            active_x.clone(),
            active_y.clone(),
        )?;
        assign_from_inputs(
            backend,
            outputs.saved_mass_point,
            active_x.clone(),
            active_y.clone(),
            |index| inputs.base.values()[index] + time_levels.current.values()[index],
        )?;
        copy_horizontal(
            backend,
            outputs.saved_west_east,
            inputs.west_east,
            u_x,
            active_y.clone(),
        )?;
        copy_horizontal(
            backend,
            outputs.saved_south_north,
            inputs.south_north,
            active_x.clone(),
            v_y,
        )?;
        fill_horizontal(
            backend,
            outputs.divergence_damping,
            active_x.clone(),
            active_y.clone(),
            0.0,
        )?;
        save_and_replace_current(
            backend,
            saved.column_mass,
            time_levels.current,
            time_levels.previous,
            active_x,
            active_y,
            true,
        )
    } else {
        assign_from_inputs(
            backend,
            outputs.saved_mass_point,
            active_x.clone(),
            active_y.clone(),
            |index| inputs.base.values()[index] + time_levels.previous.values()[index],
        )?;
        assign_from_inputs(
            backend,
            outputs.saved_west_east,
            u_x,
            active_y.clone(),
            |index| {
                0.5 * (inputs.base.values()[index]
                    + time_levels.previous.values()[index]
                    + inputs.base.values()[index - 1]
                    + time_levels.previous.values()[index - 1])
            },
        )?;
        let west_east_points = region.shape().west_east_points();
        assign_from_inputs(
            backend,
            outputs.saved_south_north,
            active_x.clone(),
            v_y,
            |index| {
                0.5 * (inputs.base.values()[index]
                    + time_levels.previous.values()[index]
                    + inputs.base.values()[index - west_east_points]
                    + time_levels.previous.values()[index - west_east_points])
            },
        )?;
        save_and_replace_current(
            backend,
            saved.column_mass,
            time_levels.current,
            time_levels.previous,
            active_x,
            active_y,
            false,
        )
    }
}

fn copy_horizontal(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    input: &CpuField<f32>,
    west_east: Range<usize>,
    south_north: Range<usize>,
) -> AcousticStepPreparationResult<()> {
    assign_from_inputs(backend, output, west_east, south_north, |index| {
        input.values()[index]
    })
}

fn fill_horizontal(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    west_east: Range<usize>,
    south_north: Range<usize>,
    value: f32,
) -> AcousticStepPreparationResult<()> {
    assign_from_inputs(backend, output, west_east, south_north, |_| value)
}

fn assign_from_inputs<Operation>(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    west_east: Range<usize>,
    south_north: Range<usize>,
    operation: Operation,
) -> AcousticStepPreparationResult<()>
where
    Operation: Fn(usize) -> f32 + Send + Sync,
{
    let west_east_points = output.shape().west_east_points();
    backend
        .try_for_each_output_block(output.values_mut(), west_east_points, |row_index, row| {
            if south_north.contains(&row_index) {
                let row_start = row_index * west_east_points;
                for west_east_index in west_east.clone() {
                    row[west_east_index] = operation(row_start + west_east_index);
                }
            }
            Ok::<(), Infallible>(())
        })
        .map_err(map_parallel_error)
}

#[allow(clippy::too_many_arguments)]
fn save_and_replace_current(
    backend: &CpuBackend,
    saved: &mut CpuField<f32>,
    current: &mut CpuField<f32>,
    previous: &CpuField<f32>,
    west_east: Range<usize>,
    south_north: Range<usize>,
    reset_to_zero: bool,
) -> AcousticStepPreparationResult<()> {
    let west_east_points = current.shape().west_east_points();
    let previous = previous.values();
    backend
        .try_for_each_output_pair_block(
            saved.values_mut(),
            current.values_mut(),
            west_east_points,
            |row_index, saved_row, current_row| {
                if south_north.contains(&row_index) {
                    let row_start = row_index * west_east_points;
                    for west_east_index in west_east.clone() {
                        let current_value = current_row[west_east_index];
                        saved_row[west_east_index] = current_value;
                        current_row[west_east_index] = if reset_to_zero {
                            0.0
                        } else {
                            previous[row_start + west_east_index] - current_value
                        };
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}
