use crate::{KesslerMicrophysicsError, KesslerMicrophysicsParameter, KesslerMicrophysicsResult};

/// Validated scalar inputs shared by every column in one Kessler update.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KesslerMicrophysicsParameters {
    time_step: f32,
    latent_heat_of_vaporization: f32,
    dry_air_specific_heat: f32,
    vapor_gas_constant_ratio: f32,
    saturation_vapor_pressure_scale: f32,
    saturation_vapor_pressure_exponent: f32,
    saturation_vapor_pressure_temperature_offset: f32,
    saturation_vapor_pressure_reference_temperature: f32,
    liquid_water_density: f32,
}

impl KesslerMicrophysicsParameters {
    /// Creates parameters from the constants passed to WRF's `kessler` routine.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        time_step: f32,
        latent_heat_of_vaporization: f32,
        dry_air_specific_heat: f32,
        vapor_gas_constant_ratio: f32,
        saturation_vapor_pressure_scale: f32,
        saturation_vapor_pressure_exponent: f32,
        saturation_vapor_pressure_temperature_offset: f32,
        saturation_vapor_pressure_reference_temperature: f32,
        liquid_water_density: f32,
    ) -> KesslerMicrophysicsResult<Self> {
        validate_positive(KesslerMicrophysicsParameter::TimeStep, time_step)?;
        validate_positive(
            KesslerMicrophysicsParameter::LatentHeatOfVaporization,
            latent_heat_of_vaporization,
        )?;
        validate_positive(
            KesslerMicrophysicsParameter::DryAirSpecificHeat,
            dry_air_specific_heat,
        )?;
        validate_positive(
            KesslerMicrophysicsParameter::VaporGasConstantRatio,
            vapor_gas_constant_ratio,
        )?;
        validate_positive(
            KesslerMicrophysicsParameter::SaturationVaporPressureScale,
            saturation_vapor_pressure_scale,
        )?;
        validate_positive(
            KesslerMicrophysicsParameter::SaturationVaporPressureExponent,
            saturation_vapor_pressure_exponent,
        )?;
        validate_positive(
            KesslerMicrophysicsParameter::SaturationVaporPressureTemperatureOffset,
            saturation_vapor_pressure_temperature_offset,
        )?;
        validate_positive(
            KesslerMicrophysicsParameter::SaturationVaporPressureReferenceTemperature,
            saturation_vapor_pressure_reference_temperature,
        )?;
        validate_positive(
            KesslerMicrophysicsParameter::LiquidWaterDensity,
            liquid_water_density,
        )?;

        Ok(Self {
            time_step,
            latent_heat_of_vaporization,
            dry_air_specific_heat,
            vapor_gas_constant_ratio,
            saturation_vapor_pressure_scale,
            saturation_vapor_pressure_exponent,
            saturation_vapor_pressure_temperature_offset,
            saturation_vapor_pressure_reference_temperature,
            liquid_water_density,
        })
    }

    /// Creates the constants used by WRF v4.7.1 with a caller-provided time step.
    pub fn try_from_wrf_defaults(time_step: f32) -> KesslerMicrophysicsResult<Self> {
        Self::try_new(
            time_step,
            2.5e6_f32,
            7.0_f32 * 287.0_f32 / 2.0_f32,
            287.0_f32 / 461.6_f32,
            0.6112_f32,
            17.67_f32,
            29.65_f32,
            273.15_f32,
            1000.0_f32,
        )
    }

    pub(crate) const fn time_step(self) -> f32 {
        self.time_step
    }

    pub(crate) const fn latent_heat_of_vaporization(self) -> f32 {
        self.latent_heat_of_vaporization
    }

    pub(crate) const fn dry_air_specific_heat(self) -> f32 {
        self.dry_air_specific_heat
    }

    pub(crate) const fn vapor_gas_constant_ratio(self) -> f32 {
        self.vapor_gas_constant_ratio
    }

    pub(crate) const fn saturation_vapor_pressure_scale(self) -> f32 {
        self.saturation_vapor_pressure_scale
    }

    pub(crate) const fn saturation_vapor_pressure_exponent(self) -> f32 {
        self.saturation_vapor_pressure_exponent
    }

    pub(crate) const fn saturation_vapor_pressure_temperature_offset(self) -> f32 {
        self.saturation_vapor_pressure_temperature_offset
    }

    pub(crate) const fn saturation_vapor_pressure_reference_temperature(self) -> f32 {
        self.saturation_vapor_pressure_reference_temperature
    }

    pub(crate) const fn liquid_water_density(self) -> f32 {
        self.liquid_water_density
    }
}

fn validate_positive(
    parameter: KesslerMicrophysicsParameter,
    value: f32,
) -> KesslerMicrophysicsResult<()> {
    if !value.is_finite() || value <= 0.0 {
        return Err(KesslerMicrophysicsError::InvalidParameter { parameter, value });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrf_defaults_use_model_constants_and_validate_time_step() {
        let parameters = KesslerMicrophysicsParameters::try_from_wrf_defaults(30.0).unwrap();

        assert_eq!(parameters.time_step(), 30.0);
        assert_eq!(parameters.latent_heat_of_vaporization(), 2.5e6);
        assert_eq!(parameters.dry_air_specific_heat(), 1004.5);
        assert_eq!(parameters.saturation_vapor_pressure_scale(), 0.6112);
        assert_eq!(parameters.saturation_vapor_pressure_exponent(), 17.67);
        assert_eq!(
            KesslerMicrophysicsParameters::try_from_wrf_defaults(0.0),
            Err(KesslerMicrophysicsError::InvalidParameter {
                parameter: KesslerMicrophysicsParameter::TimeStep,
                value: 0.0,
            })
        );
    }

    #[test]
    fn try_new_rejects_non_finite_constants() {
        let result = KesslerMicrophysicsParameters::try_new(
            30.0,
            f32::NAN,
            1004.0,
            0.622,
            0.6112,
            17.67,
            29.65,
            273.15,
            1000.0,
        );

        assert!(matches!(
            result,
            Err(KesslerMicrophysicsError::InvalidParameter {
                parameter: KesslerMicrophysicsParameter::LatentHeatOfVaporization,
                value,
            }) if value.is_nan()
        ));
    }
}
