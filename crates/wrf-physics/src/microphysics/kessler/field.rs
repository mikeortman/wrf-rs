use std::fmt;

/// Field participating in one Kessler microphysics update.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KesslerMicrophysicsField {
    /// Potential temperature updated in place.
    PotentialTemperature,
    /// Water-vapor mixing ratio updated in place.
    WaterVaporMixingRatio,
    /// Cloud-water mixing ratio updated in place.
    CloudWaterMixingRatio,
    /// Rain-water mixing ratio updated in place.
    RainWaterMixingRatio,
    /// Dry-air density.
    DryAirDensity,
    /// Exner function.
    ExnerFunction,
    /// Geopotential height at mass levels.
    Height,
    /// Vertical layer thickness on W levels.
    VerticalLayerThickness,
    /// Non-convective accumulated precipitation.
    AccumulatedPrecipitation,
    /// Non-convective precipitation for the current update.
    StepPrecipitation,
}

impl fmt::Display for KesslerMicrophysicsField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PotentialTemperature => formatter.write_str("potential temperature"),
            Self::WaterVaporMixingRatio => formatter.write_str("water-vapor mixing ratio"),
            Self::CloudWaterMixingRatio => formatter.write_str("cloud-water mixing ratio"),
            Self::RainWaterMixingRatio => formatter.write_str("rain-water mixing ratio"),
            Self::DryAirDensity => formatter.write_str("dry-air density"),
            Self::ExnerFunction => formatter.write_str("Exner function"),
            Self::Height => formatter.write_str("height"),
            Self::VerticalLayerThickness => formatter.write_str("vertical layer thickness"),
            Self::AccumulatedPrecipitation => formatter.write_str("accumulated precipitation"),
            Self::StepPrecipitation => formatter.write_str("step precipitation"),
        }
    }
}
