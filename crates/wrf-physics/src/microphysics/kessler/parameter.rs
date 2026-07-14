use std::fmt;

/// Scalar Kessler parameter named in validation failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KesslerMicrophysicsParameter {
    /// Physics update duration.
    TimeStep,
    /// Latent heat of vaporization.
    LatentHeatOfVaporization,
    /// Dry-air specific heat at constant pressure.
    DryAirSpecificHeat,
    /// Dry-air to water-vapor gas-constant ratio.
    VaporGasConstantRatio,
    /// Saturation-vapor-pressure scale.
    SaturationVaporPressureScale,
    /// Saturation-vapor-pressure exponential coefficient.
    SaturationVaporPressureExponent,
    /// Saturation-vapor-pressure temperature offset.
    SaturationVaporPressureTemperatureOffset,
    /// Saturation-vapor-pressure reference temperature.
    SaturationVaporPressureReferenceTemperature,
    /// Liquid-water density.
    LiquidWaterDensity,
}

impl fmt::Display for KesslerMicrophysicsParameter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TimeStep => formatter.write_str("time step"),
            Self::LatentHeatOfVaporization => formatter.write_str("latent heat of vaporization"),
            Self::DryAirSpecificHeat => formatter.write_str("dry-air specific heat"),
            Self::VaporGasConstantRatio => formatter.write_str("vapor gas-constant ratio"),
            Self::SaturationVaporPressureScale => {
                formatter.write_str("saturation-vapor-pressure scale")
            }
            Self::SaturationVaporPressureExponent => {
                formatter.write_str("saturation-vapor-pressure exponent")
            }
            Self::SaturationVaporPressureTemperatureOffset => {
                formatter.write_str("saturation-vapor-pressure temperature offset")
            }
            Self::SaturationVaporPressureReferenceTemperature => {
                formatter.write_str("saturation-vapor-pressure reference temperature")
            }
            Self::LiquidWaterDensity => formatter.write_str("liquid-water density"),
        }
    }
}
