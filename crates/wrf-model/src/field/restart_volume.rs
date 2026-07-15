use std::fmt;

/// Registry-backed restart fields produced or consumed by accepted stages.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(usize)]
pub enum ArwRestartVolumeField {
    /// Coupled vertical mass flux (`ww`).
    VerticalMassFlux,
    /// Time-averaged coupled vertical mass flux (`ww_m`).
    AverageVerticalMassFlux,
    /// Pressure-point geopotential (`php`).
    PressurePointGeopotential,
    /// Microphysics latent-heating tendency (`h_diabatic`).
    DiabaticHeating,
    /// Microphysics water-vapor tendency (`qv_diabatic`).
    WaterVaporDiabaticTendency,
    /// Microphysics cloud-water tendency (`qc_diabatic`).
    CloudWaterDiabaticTendency,
    /// Dry-air density (`rho`).
    DryAirDensity,
    /// Dry perturbation potential temperature (`th_phy_m_t0`).
    PerturbationDryPotentialTemperature,
}

impl ArwRestartVolumeField {
    pub(crate) const COUNT: usize = 8;
    /// All typed fields in storage order.
    pub const ALL: [Self; Self::COUNT] = [
        Self::VerticalMassFlux,
        Self::AverageVerticalMassFlux,
        Self::PressurePointGeopotential,
        Self::DiabaticHeating,
        Self::WaterVaporDiabaticTendency,
        Self::CloudWaterDiabaticTendency,
        Self::DryAirDensity,
        Self::PerturbationDryPotentialTemperature,
    ];

    pub(crate) const fn registry_name(self) -> &'static str {
        match self {
            Self::VerticalMassFlux => "ww",
            Self::AverageVerticalMassFlux => "ww_m",
            Self::PressurePointGeopotential => "php",
            Self::DiabaticHeating => "h_diabatic",
            Self::WaterVaporDiabaticTendency => "qv_diabatic",
            Self::CloudWaterDiabaticTendency => "qc_diabatic",
            Self::DryAirDensity => "rho",
            Self::PerturbationDryPotentialTemperature => "th_phy_m_t0",
        }
    }
}

impl fmt::Display for ArwRestartVolumeField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.registry_name())
    }
}
