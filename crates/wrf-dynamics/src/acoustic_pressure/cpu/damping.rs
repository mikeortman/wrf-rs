use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField};

use crate::{
    AcousticPressureDampingPhase, AcousticPressureParameters, AcousticPressureRegion,
    AcousticPressureResult, AcousticPressureState,
};

use super::map_parallel_error;

pub(super) fn apply(
    backend: &CpuBackend,
    state: &mut AcousticPressureState<'_, CpuField<f32>>,
    parameters: AcousticPressureParameters,
    phase: AcousticPressureDampingPhase,
    region: &AcousticPressureRegion,
) -> AcousticPressureResult<()> {
    let shape = region.shape();
    let x_points = shape.west_east_points();
    let z_points = shape.bottom_top_points();
    let active_x = region.active_west_east();
    let active_y = region.active_south_north();
    let active_k = region.active_half_levels();
    backend
        .try_for_each_output_pair_block(
            state.pressure_perturbation.values_mut(),
            state.previous_pressure_perturbation.values_mut(),
            x_points,
            |line, pressure_row, previous_pressure_row| {
                let y = line / z_points;
                let k = line % z_points;
                if active_y.contains(&y) && active_k.contains(&k) {
                    for x in active_x.clone() {
                        match phase {
                            AcousticPressureDampingPhase::Initialize => {
                                previous_pressure_row[x] = pressure_row[x];
                            }
                            AcousticPressureDampingPhase::Advance => {
                                let undamped_pressure = pressure_row[x];
                                pressure_row[x] = pressure_row[x]
                                    + parameters.divergence_damping
                                        * (pressure_row[x] - previous_pressure_row[x]);
                                previous_pressure_row[x] = undamped_pressure;
                            }
                        }
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}
