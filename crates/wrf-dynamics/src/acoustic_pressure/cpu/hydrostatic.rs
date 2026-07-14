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
    diagnose_pressure_and_inverse_density(
        backend,
        state,
        thermodynamics,
        masses,
        coefficients,
        parameters,
        region,
    )?;
    integrate_geopotential(
        backend,
        state,
        thermodynamics,
        masses,
        coefficients,
        vertical_metrics,
        region,
    )
}

#[allow(clippy::too_many_arguments)]
fn diagnose_pressure_and_inverse_density(
    backend: &CpuBackend,
    state: &mut AcousticPressureState<'_, CpuField<f32>>,
    thermodynamics: &AcousticPressureThermodynamics<'_, CpuField<f32>>,
    masses: &AcousticPressureMasses<'_, CpuField<f32>>,
    coefficients: AcousticPressureCoefficients<'_>,
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
                        let pressure = perturbation_mass[horizontal]
                            * coefficients.hydrostatic_pressure_multiplier[k];
                        pressure_row[x] = pressure;
                        let perturbation_mass_weighted = coefficients.half_level_mass_multiplier[k]
                            * perturbation_mass[horizontal];
                        let full_mass_weighted = coefficients.half_level_mass_multiplier[k]
                            * full_mass[horizontal]
                            + coefficients.half_level_mass_offset[k];
                        inverse_density_row[x] = full_inverse_density[index]
                            * (current_temperature[index]
                                - perturbation_mass_weighted * reference_temperature[index])
                            / (full_mass_weighted
                                * (parameters.reference_temperature
                                    + reference_temperature[index]))
                            - pressure / pressure_coefficient[index];
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

#[allow(clippy::too_many_arguments)]
fn integrate_geopotential(
    backend: &CpuBackend,
    state: &mut AcousticPressureState<'_, CpuField<f32>>,
    thermodynamics: &AcousticPressureThermodynamics<'_, CpuField<f32>>,
    masses: &AcousticPressureMasses<'_, CpuField<f32>>,
    coefficients: AcousticPressureCoefficients<'_>,
    vertical_metrics: AcousticPressureVerticalMetrics<'_>,
    region: &AcousticPressureRegion,
) -> AcousticPressureResult<()> {
    let shape = region.shape();
    let x_points = shape.west_east_points();
    let z_points = shape.bottom_top_points();
    let plane_points = x_points * z_points;
    let active_x = region.active_west_east();
    let active_y = region.active_south_north();
    let active_k = region.active_half_levels();
    let inverse_density = state.inverse_density_perturbation.values();
    let full_inverse_density = thermodynamics.full_inverse_density.values();
    let perturbation_mass = masses.perturbation.values();
    let full_mass = masses.full.values();
    backend
        .try_for_each_output_block(
            state.geopotential_perturbation.values_mut(),
            plane_points,
            |y, geopotential_plane| {
                if active_y.contains(&y) {
                    let plane_start = y * plane_points;
                    let horizontal_start = y * x_points;
                    for k in active_k.clone() {
                        for x in active_x.clone() {
                            let horizontal = horizontal_start + x;
                            let local_index = k * x_points + x;
                            let index = plane_start + local_index;
                            let perturbation_mass_weighted = coefficients
                                .half_level_mass_multiplier[k]
                                * perturbation_mass[horizontal];
                            let full_mass_weighted = coefficients.half_level_mass_multiplier[k]
                                * full_mass[horizontal]
                                + coefficients.half_level_mass_offset[k];
                            geopotential_plane[local_index + x_points] = geopotential_plane
                                [local_index]
                                - vertical_metrics.eta_thickness[k]
                                    * (full_mass_weighted * inverse_density[index]
                                        + perturbation_mass_weighted * full_inverse_density[index]);
                        }
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}
