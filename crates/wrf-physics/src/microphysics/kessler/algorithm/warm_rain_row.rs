use std::ops::Range;

use crate::KesslerMicrophysicsParameters;

const AUTOCONVERSION_THRESHOLD: f32 = 0.001;
const AUTOCONVERSION_RATE: f32 = 0.001;
const ACCRETION_RATE: f32 = 2.2;
const ACCRETION_EXPONENT: f32 = 0.875;

/// Mutable view of one south-north row during warm-rain conversion.
pub(super) struct KesslerWarmRainRow<'a> {
    potential_temperature: &'a mut [f32],
    water_vapor_mixing_ratio: &'a mut [f32],
    cloud_water_mixing_ratio: &'a mut [f32],
    rain_water_mixing_ratio: &'a mut [f32],
    dry_air_density: &'a [f32],
    exner_function: &'a [f32],
    production: &'a [f32],
    west_east_points: usize,
}

impl<'a> KesslerWarmRainRow<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        potential_temperature: &'a mut [f32],
        water_vapor_mixing_ratio: &'a mut [f32],
        cloud_water_mixing_ratio: &'a mut [f32],
        rain_water_mixing_ratio: &'a mut [f32],
        dry_air_density: &'a [f32],
        exner_function: &'a [f32],
        production: &'a [f32],
        west_east_points: usize,
    ) -> Self {
        Self {
            potential_temperature,
            water_vapor_mixing_ratio,
            cloud_water_mixing_ratio,
            rain_water_mixing_ratio,
            dry_air_density,
            exner_function,
            production,
            west_east_points,
        }
    }

    pub(super) fn apply(
        &mut self,
        west_east_range: Range<usize>,
        bottom_top_range: Range<usize>,
        parameters: KesslerMicrophysicsParameters,
    ) {
        let saturation_temperature_factor = parameters.saturation_vapor_pressure_exponent()
            * (parameters.saturation_vapor_pressure_reference_temperature()
                - parameters.saturation_vapor_pressure_temperature_offset())
            * parameters.latent_heat_of_vaporization()
            / parameters.dry_air_specific_heat();

        for bottom_top_index in bottom_top_range {
            for west_east_index in west_east_range.clone() {
                self.apply_point(
                    west_east_index,
                    bottom_top_index,
                    saturation_temperature_factor,
                    parameters,
                );
            }
        }
    }

    fn apply_point(
        &mut self,
        west_east_index: usize,
        bottom_top_index: usize,
        saturation_temperature_factor: f32,
        parameters: KesslerMicrophysicsParameters,
    ) {
        let linear_index = self.linear_index(west_east_index, bottom_top_index);
        let rain_water = self.rain_water_mixing_ratio[linear_index];
        let conversion_factor = 1.0_f32
            / (1.0_f32
                + ACCRETION_RATE
                    * parameters.time_step()
                    * rain_water.max(0.0).powf(ACCRETION_EXPONENT));
        let cloud_water = self.cloud_water_mixing_ratio[linear_index];
        let rain_production = cloud_water * (1.0_f32 - conversion_factor)
            + conversion_factor
                * AUTOCONVERSION_RATE
                * parameters.time_step()
                * (cloud_water - AUTOCONVERSION_THRESHOLD).max(0.0);
        let density_scale = 0.001_f32 * self.dry_air_density[linear_index];

        self.cloud_water_mixing_ratio[linear_index] = (cloud_water - rain_production).max(0.0);
        let rain_with_sedimentation = rain_water + self.production[linear_index];
        self.rain_water_mixing_ratio[linear_index] = rain_with_sedimentation - rain_water;
        self.rain_water_mixing_ratio[linear_index] =
            (self.rain_water_mixing_ratio[linear_index] + rain_production).max(0.0);

        let exner = self.exner_function[linear_index];
        let temperature = exner * self.potential_temperature[linear_index];
        let pressure = 1.0e5_f32 * exner.powf(1004.0_f32 / 287.0_f32);
        let condensation_heating = 2.5e6_f32 / (1004.0_f32 * exner);
        let vapor_pressure_exponent = parameters.saturation_vapor_pressure_exponent()
            * (temperature - parameters.saturation_vapor_pressure_reference_temperature())
            / (temperature - parameters.saturation_vapor_pressure_temperature_offset());
        let saturation_vapor_pressure = 1000.0_f32
            * parameters.saturation_vapor_pressure_scale()
            * vapor_pressure_exponent.exp();
        let saturation_mixing_ratio = parameters.vapor_gas_constant_ratio()
            * saturation_vapor_pressure
            / (pressure - saturation_vapor_pressure);
        let saturation_adjustment_denominator = 1.0_f32
            + pressure / (pressure - saturation_vapor_pressure)
                * saturation_mixing_ratio
                * saturation_temperature_factor
                / (temperature - parameters.saturation_vapor_pressure_temperature_offset()).powi(2);
        let saturation_adjustment = (self.water_vapor_mixing_ratio[linear_index]
            - saturation_mixing_ratio)
            / saturation_adjustment_denominator;
        let evaporation = self.calculate_evaporation(
            linear_index,
            density_scale,
            pressure,
            saturation_mixing_ratio,
            saturation_adjustment,
            parameters.time_step(),
        );
        let condensed_water =
            saturation_adjustment.max(-self.cloud_water_mixing_ratio[linear_index]);

        self.potential_temperature[linear_index] +=
            condensation_heating * (condensed_water - evaporation);
        self.water_vapor_mixing_ratio[linear_index] =
            (self.water_vapor_mixing_ratio[linear_index] - condensed_water + evaporation).max(0.0);
        self.cloud_water_mixing_ratio[linear_index] += condensed_water;
        self.rain_water_mixing_ratio[linear_index] -= evaporation;
    }

    fn calculate_evaporation(
        &self,
        linear_index: usize,
        density_scale: f32,
        pressure: f32,
        saturation_mixing_ratio: f32,
        saturation_adjustment: f32,
        time_step: f32,
    ) -> f32 {
        let rain_mass = density_scale * self.rain_water_mixing_ratio[linear_index];
        let ventilation = 1.6_f32 + 124.9_f32 * rain_mass.powf(0.2046_f32);
        let rain_surface = rain_mass.powf(0.525_f32);
        let resistance = 2.55e8_f32 / (pressure * saturation_mixing_ratio) + 5.4e5_f32;
        let vapor_deficit =
            (saturation_mixing_ratio - self.water_vapor_mixing_ratio[linear_index]).max(0.0);
        let evaporation_rate = (ventilation * rain_surface) / resistance;
        let requested_evaporation = time_step
            * evaporation_rate
            * (vapor_deficit / (density_scale * saturation_mixing_ratio));
        let saturation_limit =
            (-saturation_adjustment - self.cloud_water_mixing_ratio[linear_index]).max(0.0);
        requested_evaporation
            .min(saturation_limit)
            .min(self.rain_water_mixing_ratio[linear_index])
    }

    fn linear_index(&self, west_east_index: usize, bottom_top_index: usize) -> usize {
        bottom_top_index * self.west_east_points + west_east_index
    }
}
