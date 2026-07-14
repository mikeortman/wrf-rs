use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField};

use crate::{
    AcousticPressureCoefficients, AcousticPressureMasses, AcousticPressureParameters,
    AcousticPressureRegion, AcousticPressureResult, AcousticPressureState,
    AcousticPressureThermodynamics, AcousticPressureVerticalMetrics,
};

use super::map_parallel_error;

#[allow(clippy::too_many_arguments)]
pub(super) fn diagnose(
    backend: &CpuBackend,
    state: &mut AcousticPressureState<'_, CpuField<f32>>,
    thermodynamics: &AcousticPressureThermodynamics<'_, CpuField<f32>>,
    masses: &AcousticPressureMasses<'_, CpuField<f32>>,
    coefficients: AcousticPressureCoefficients<'_>,
    vertical_metrics: AcousticPressureVerticalMetrics<'_>,
    parameters: AcousticPressureParameters,
    region: &AcousticPressureRegion,
) -> AcousticPressureResult<()> {
    let shape = region.shape();
    let x_points = shape.west_east_points();
    let z_points = shape.bottom_top_points();
    let active_x = region.active_west_east();
    let active_y = region.active_south_north();
    let active_k = region.active_half_levels();
    let full_inverse_density = thermodynamics.full_inverse_density.values();
    let current_temperature = thermodynamics.current_temperature.values();
    let reference_temperature = thermodynamics.reference_temperature_perturbation.values();
    let pressure_coefficient = thermodynamics.pressure_coefficient.values();
    let geopotential = state.geopotential_perturbation.values();
    let perturbation_mass = masses.perturbation.values();
    let full_mass = masses.full.values();
    backend
        .try_for_each_output_pair_block(
            state.inverse_density_perturbation.values_mut(),
            state.pressure_perturbation.values_mut(),
            x_points,
            |line, inverse_density_row, pressure_row| {
                let y = line / z_points;
                let k = line % z_points;
                if active_y.contains(&y) && active_k.contains(&k) {
                    let row_start = line * x_points;
                    let horizontal_start = y * x_points;
                    for x in active_x.clone() {
                        let index = row_start + x;
                        let horizontal = horizontal_start + x;
                        let perturbation_mass_weighted = coefficients.half_level_mass_multiplier[k]
                            * perturbation_mass[horizontal];
                        let full_mass_weighted = coefficients.half_level_mass_multiplier[k]
                            * full_mass[horizontal]
                            + coefficients.half_level_mass_offset[k];
                        let inverse_density = -1.0 / full_mass_weighted
                            * (full_inverse_density[index] * perturbation_mass_weighted
                                + vertical_metrics.inverse_eta_thickness[k]
                                    * (geopotential[index + x_points] - geopotential[index]));
                        inverse_density_row[x] = inverse_density;
                        pressure_row[x] = pressure_coefficient[index]
                            * (full_inverse_density[index]
                                * (current_temperature[index]
                                    - perturbation_mass_weighted * reference_temperature[index])
                                / (full_mass_weighted
                                    * (parameters.reference_temperature
                                        + reference_temperature[index]))
                                - inverse_density);
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}
