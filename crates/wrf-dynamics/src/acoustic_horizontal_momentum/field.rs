use std::fmt;

/// Scientific role of a field in acoustic horizontal-momentum advancement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticHorizontalMomentumField {
    /// West-east momentum (`u`).
    WestEastMomentum,
    /// South-north momentum (`v`).
    SouthNorthMomentum,
    /// West-east large-step tendency (`ru_tend`).
    WestEastTendency,
    /// South-north large-step tendency (`rv_tend`).
    SouthNorthTendency,
    /// Perturbation pressure (`p`).
    PressurePerturbation,
    /// Base-state pressure (`pb`).
    BasePressure,
    /// Full-level perturbation geopotential (`ph`).
    GeopotentialPerturbation,
    /// Pressure-point perturbation geopotential (`php`).
    PressurePointGeopotential,
    /// Full inverse density (`alt`).
    FullInverseDensity,
    /// Perturbation inverse density (`al`).
    InverseDensityPerturbation,
    /// Perturbation column mass (`mu`).
    PerturbationColumnMass,
    /// West-east staggered full column mass (`muu`).
    WestEastColumnMass,
    /// South-north staggered full column mass (`muv`).
    SouthNorthColumnMass,
    /// Divergence-damping column mass (`mudf`).
    DivergenceDampingMass,
    /// West-east moisture coefficient (`cqu`).
    WestEastMoistureCoefficient,
    /// South-north moisture coefficient (`cqv`).
    SouthNorthMoistureCoefficient,
    /// West-east X map factor (`msfux`).
    WestEastXMapFactor,
    /// West-east Y map factor (`msfuy`).
    WestEastYMapFactor,
    /// South-north X map factor (`msfvx`).
    SouthNorthXMapFactor,
    /// Inverse south-north X map factor (`msfvx_inv`).
    InverseSouthNorthXMapFactor,
    /// South-north Y map factor (`msfvy`).
    SouthNorthYMapFactor,
}

impl fmt::Display for AcousticHorizontalMomentumField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WestEastMomentum => "west-east momentum",
            Self::SouthNorthMomentum => "south-north momentum",
            Self::WestEastTendency => "west-east tendency",
            Self::SouthNorthTendency => "south-north tendency",
            Self::PressurePerturbation => "pressure perturbation",
            Self::BasePressure => "base pressure",
            Self::GeopotentialPerturbation => "geopotential perturbation",
            Self::PressurePointGeopotential => "pressure-point geopotential",
            Self::FullInverseDensity => "full inverse density",
            Self::InverseDensityPerturbation => "inverse-density perturbation",
            Self::PerturbationColumnMass => "perturbation column mass",
            Self::WestEastColumnMass => "west-east column mass",
            Self::SouthNorthColumnMass => "south-north column mass",
            Self::DivergenceDampingMass => "divergence-damping mass",
            Self::WestEastMoistureCoefficient => "west-east moisture coefficient",
            Self::SouthNorthMoistureCoefficient => "south-north moisture coefficient",
            Self::WestEastXMapFactor => "west-east X map factor",
            Self::WestEastYMapFactor => "west-east Y map factor",
            Self::SouthNorthXMapFactor => "south-north X map factor",
            Self::InverseSouthNorthXMapFactor => "inverse south-north X map factor",
            Self::SouthNorthYMapFactor => "south-north Y map factor",
        })
    }
}
