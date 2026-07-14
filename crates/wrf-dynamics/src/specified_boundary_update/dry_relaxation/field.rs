use std::fmt;

/// Semantic field role in dry boundary-relaxation validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DryBoundaryRelaxationField {
    /// Coupled west-east momentum state (`ru`).
    WestEastMomentum,
    /// Coupled south-north momentum state (`rv`).
    SouthNorthMomentum,
    /// Perturbation geopotential state (`ph`).
    PerturbationGeopotential,
    /// Perturbation potential-temperature state (`t`).
    PotentialTemperature,
    /// Perturbation column mass (`mu`).
    PerturbationColumnMass,
    /// Full column mass used for mass weighting (`mut`).
    FullColumnMass,
    /// Optional nested vertical velocity (`w`).
    VerticalVelocity,
    /// West-east momentum output tendency.
    WestEastMomentumTendency,
    /// South-north momentum output tendency.
    SouthNorthMomentumTendency,
    /// Perturbation geopotential output tendency.
    PerturbationGeopotentialTendency,
    /// Potential-temperature output tendency.
    PotentialTemperatureTendency,
    /// Perturbation column-mass output tendency.
    PerturbationColumnMassTendency,
    /// Optional nested vertical-momentum output tendency.
    VerticalMomentumTendency,
    /// Caller-owned tile-halo mass-weighting workspace.
    MassWeightingWorkspace,
}

impl fmt::Display for DryBoundaryRelaxationField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WestEastMomentum => "west-east momentum",
            Self::SouthNorthMomentum => "south-north momentum",
            Self::PerturbationGeopotential => "perturbation geopotential",
            Self::PotentialTemperature => "potential temperature",
            Self::PerturbationColumnMass => "perturbation column mass",
            Self::FullColumnMass => "full column mass",
            Self::VerticalVelocity => "vertical velocity",
            Self::WestEastMomentumTendency => "west-east momentum tendency",
            Self::SouthNorthMomentumTendency => "south-north momentum tendency",
            Self::PerturbationGeopotentialTendency => "perturbation geopotential tendency",
            Self::PotentialTemperatureTendency => "potential-temperature tendency",
            Self::PerturbationColumnMassTendency => "perturbation column-mass tendency",
            Self::VerticalMomentumTendency => "vertical-momentum tendency",
            Self::MassWeightingWorkspace => "mass-weighting workspace",
        })
    }
}
