use std::convert::Infallible;
use std::ops::Range;

use wrf_compute::{CpuBackend, CpuField, FieldStorage};

use crate::{
    AcousticStepPreparationCoefficients, AcousticStepPreparationDiagnosticInputs,
    AcousticStepPreparationMapFactors, AcousticStepPreparationMassInputs,
    AcousticStepPreparationMassOutputs, AcousticStepPreparationRegion,
    AcousticStepPreparationResult, AcousticStepPreparationSavedOutputs,
    AcousticStepPreparationVolumeTimeLevels,
};

use super::map_parallel_error;

const WRF_CP_OVER_CV: f32 = 1.4;

pub(super) fn switch_time_levels(
    backend: &CpuBackend,
    time_levels: &mut AcousticStepPreparationVolumeTimeLevels<'_, CpuField<f32>>,
    region: &AcousticStepPreparationRegion,
) -> AcousticStepPreparationResult<()> {
    let active_x = region.active_west_east();
    let active_y = region.active_south_north();
    let half_levels = region.half_levels();
    let full_levels = region.full_levels();
    copy_volume(
        backend,
        time_levels.previous_west_east_velocity,
        time_levels.current_west_east_velocity,
        region.west_east_velocity_range(),
        active_y.clone(),
        half_levels.clone(),
    )?;
    copy_volume(
        backend,
        time_levels.previous_south_north_velocity,
        time_levels.current_south_north_velocity,
        active_x.clone(),
        region.south_north_velocity_range(),
        half_levels.clone(),
    )?;
    copy_volume(
        backend,
        time_levels.previous_potential_temperature,
        time_levels.current_potential_temperature,
        active_x.clone(),
        active_y.clone(),
        half_levels,
    )?;
    copy_volume(
        backend,
        time_levels.previous_vertical_velocity,
        time_levels.current_vertical_velocity,
        active_x.clone(),
        active_y.clone(),
        full_levels.clone(),
    )?;
    copy_volume(
        backend,
        time_levels.previous_geopotential,
        time_levels.current_geopotential,
        active_x,
        active_y,
        full_levels,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare_coupled_fields(
    backend: &CpuBackend,
    time_levels: &mut AcousticStepPreparationVolumeTimeLevels<'_, CpuField<f32>>,
    saved: &mut AcousticStepPreparationSavedOutputs<'_, CpuField<f32>>,
    mass_outputs: &AcousticStepPreparationMassOutputs<'_, CpuField<f32>>,
    mass_inputs: &AcousticStepPreparationMassInputs<'_, CpuField<f32>>,
    diagnostics: &AcousticStepPreparationDiagnosticInputs<'_, CpuField<f32>>,
    maps: &AcousticStepPreparationMapFactors<'_, CpuField<f32>>,
    coefficients: AcousticStepPreparationCoefficients<'_>,
    region: &AcousticStepPreparationRegion,
) -> AcousticStepPreparationResult<()> {
    let active_x = region.active_west_east();
    let active_y = region.active_south_north();
    let half_levels = region.half_levels();
    let full_levels = region.full_levels();

    assign_volume(
        backend,
        saved.pressure_coefficient,
        active_x.clone(),
        active_y.clone(),
        half_levels.clone(),
        |index, _| {
            WRF_CP_OVER_CV
                * (diagnostics.base_pressure.values()[index]
                    + diagnostics.perturbation_pressure.values()[index])
                / diagnostics.inverse_density.values()[index]
        },
    )?;

    let previous_u = time_levels.previous_west_east_velocity.values();
    let saved_u_mass = mass_outputs.saved_west_east.values();
    let current_u_mass = mass_inputs.west_east.values();
    let u_map = maps.west_east_momentum_south_north.values();
    for_each_volume_pair(
        backend,
        saved.west_east_velocity,
        time_levels.current_west_east_velocity,
        region.west_east_velocity_range(),
        active_y.clone(),
        half_levels.clone(),
        |index, horizontal, vertical, saved_value, current| {
            let old_current = *current;
            *saved_value = old_current;
            *current = ((coefficients.half_level_mass_multiplier[vertical]
                * saved_u_mass[horizontal]
                + coefficients.half_level_offset[vertical])
                * previous_u[index]
                - (coefficients.half_level_mass_multiplier[vertical] * current_u_mass[horizontal]
                    + coefficients.half_level_offset[vertical])
                    * old_current)
                / u_map[horizontal];
        },
    )?;

    let previous_v = time_levels.previous_south_north_velocity.values();
    let saved_v_mass = mass_outputs.saved_south_north.values();
    let current_v_mass = mass_inputs.south_north.values();
    let inverse_v_map = maps.inverse_south_north_momentum_west_east.values();
    for_each_volume_pair(
        backend,
        saved.south_north_velocity,
        time_levels.current_south_north_velocity,
        active_x.clone(),
        region.south_north_velocity_range(),
        half_levels.clone(),
        |index, horizontal, vertical, saved_value, current| {
            let old_current = *current;
            *saved_value = old_current;
            *current = ((coefficients.half_level_mass_multiplier[vertical]
                * saved_v_mass[horizontal]
                + coefficients.half_level_offset[vertical])
                * previous_v[index]
                - (coefficients.half_level_mass_multiplier[vertical] * current_v_mass[horizontal]
                    + coefficients.half_level_offset[vertical])
                    * old_current)
                * inverse_v_map[horizontal];
        },
    )?;

    let previous_t = time_levels.previous_potential_temperature.values();
    let saved_mass = mass_outputs.saved_mass_point.values();
    let full_mass = mass_inputs.full.values();
    for_each_volume_pair(
        backend,
        saved.potential_temperature,
        time_levels.current_potential_temperature,
        active_x.clone(),
        active_y.clone(),
        half_levels,
        |index, horizontal, vertical, saved_value, current| {
            let old_current = *current;
            *saved_value = old_current;
            *current = (coefficients.half_level_mass_multiplier[vertical] * saved_mass[horizontal]
                + coefficients.half_level_offset[vertical])
                * previous_t[index]
                - (coefficients.half_level_mass_multiplier[vertical] * full_mass[horizontal]
                    + coefficients.half_level_offset[vertical])
                    * old_current;
        },
    )?;

    let previous_w = time_levels.previous_vertical_velocity.values();
    let mass_map = maps.mass_point_south_north.values();
    for_each_volume_pair(
        backend,
        saved.vertical_velocity,
        time_levels.current_vertical_velocity,
        active_x.clone(),
        active_y.clone(),
        full_levels.clone(),
        |index, horizontal, vertical, saved_value, current| {
            let old_current = *current;
            *saved_value = old_current;
            *current = ((coefficients.full_level_mass_multiplier[vertical]
                * saved_mass[horizontal]
                + coefficients.full_level_offset[vertical])
                * previous_w[index]
                - (coefficients.full_level_mass_multiplier[vertical] * full_mass[horizontal]
                    + coefficients.full_level_offset[vertical])
                    * old_current)
                / mass_map[horizontal];
        },
    )?;

    let previous_ph = time_levels.previous_geopotential.values();
    for_each_volume_pair(
        backend,
        saved.geopotential,
        time_levels.current_geopotential,
        active_x.clone(),
        active_y.clone(),
        full_levels.clone(),
        |index, _, _, saved_value, current| {
            let old_current = *current;
            *saved_value = old_current;
            *current = previous_ph[index] - old_current;
        },
    )?;

    copy_volume(
        backend,
        saved.omega,
        diagnostics.omega,
        active_x,
        active_y,
        full_levels,
    )
}

fn copy_volume(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    input: &CpuField<f32>,
    west_east: Range<usize>,
    south_north: Range<usize>,
    bottom_top: Range<usize>,
) -> AcousticStepPreparationResult<()> {
    assign_volume(
        backend,
        output,
        west_east,
        south_north,
        bottom_top,
        |index, _| input.values()[index],
    )
}

fn assign_volume<Operation>(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    west_east: Range<usize>,
    south_north: Range<usize>,
    bottom_top: Range<usize>,
    operation: Operation,
) -> AcousticStepPreparationResult<()>
where
    Operation: Fn(usize, usize) -> f32 + Send + Sync,
{
    let shape = output.shape();
    let west_east_points = shape.west_east_points();
    let bottom_top_points = shape.bottom_top_points();
    backend
        .try_for_each_output_block(output.values_mut(), west_east_points, |line_index, row| {
            let y = line_index / bottom_top_points;
            let k = line_index % bottom_top_points;
            if south_north.contains(&y) && bottom_top.contains(&k) {
                let row_start = line_index * west_east_points;
                for x in west_east.clone() {
                    row[x] = operation(row_start + x, k);
                }
            }
            Ok::<(), Infallible>(())
        })
        .map_err(map_parallel_error)
}

#[allow(clippy::too_many_arguments)]
fn for_each_volume_pair<Operation>(
    backend: &CpuBackend,
    first: &mut CpuField<f32>,
    second: &mut CpuField<f32>,
    west_east: Range<usize>,
    south_north: Range<usize>,
    bottom_top: Range<usize>,
    operation: Operation,
) -> AcousticStepPreparationResult<()>
where
    Operation: Fn(usize, usize, usize, &mut f32, &mut f32) + Send + Sync,
{
    let shape = first.shape();
    let x_points = shape.west_east_points();
    let z_points = shape.bottom_top_points();
    backend
        .try_for_each_output_pair_block(
            first.values_mut(),
            second.values_mut(),
            x_points,
            |line, first_row, second_row| {
                let y = line / z_points;
                let k = line % z_points;
                if south_north.contains(&y) && bottom_top.contains(&k) {
                    let row_start = line * x_points;
                    let horizontal_start = y * x_points;
                    for x in west_east.clone() {
                        operation(
                            row_start + x,
                            horizontal_start + x,
                            k,
                            &mut first_row[x],
                            &mut second_row[x],
                        );
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}
