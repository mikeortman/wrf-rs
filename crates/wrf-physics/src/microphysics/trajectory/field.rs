use std::fmt;

/// ARW field named by trajectory shape diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArwMicrophysicsField {
    /// Perturbation potential temperature (`t_2`).
    PerturbationPotentialTemperature,
    /// Perturbation inverse density (`al`).
    PerturbationInverseDensity,
    /// Base inverse density (`alb`).
    BaseInverseDensity,
    /// Perturbation pressure (`p`).
    PerturbationPressure,
    /// Base pressure (`pb`).
    BasePressure,
    /// Perturbation W-level geopotential (`ph_2`).
    PerturbationGeopotential,
    /// Base W-level geopotential (`phb`).
    BaseGeopotential,
    /// Accumulated non-convective precipitation (`RAINNC`).
    AccumulatedPrecipitation,
    /// Current-step non-convective precipitation (`RAINNCV`).
    StepPrecipitation,
    /// Reusable trajectory workspace.
    Workspace,
}

impl fmt::Display for ArwMicrophysicsField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::PerturbationPotentialTemperature => "perturbation potential temperature",
            Self::PerturbationInverseDensity => "perturbation inverse density",
            Self::BaseInverseDensity => "base inverse density",
            Self::PerturbationPressure => "perturbation pressure",
            Self::BasePressure => "base pressure",
            Self::PerturbationGeopotential => "perturbation W-level geopotential",
            Self::BaseGeopotential => "base W-level geopotential",
            Self::AccumulatedPrecipitation => "accumulated precipitation",
            Self::StepPrecipitation => "step precipitation",
            Self::Workspace => "trajectory workspace",
        })
    }
}
