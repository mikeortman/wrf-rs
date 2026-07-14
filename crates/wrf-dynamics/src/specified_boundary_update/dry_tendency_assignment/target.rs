use std::fmt;

/// Dry field receiving one scalar specified-boundary tendency assignment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DryBoundaryTendencyTarget {
    /// West-east momentum.
    WestEastMomentum,
    /// South-north momentum.
    SouthNorthMomentum,
    /// Perturbation geopotential.
    PerturbationGeopotential,
    /// Potential temperature.
    PotentialTemperature,
    /// Horizontal perturbation column mass.
    PerturbationColumnMass,
    /// Nested vertical momentum.
    VerticalMomentum,
}

impl fmt::Display for DryBoundaryTendencyTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WestEastMomentum => "west-east momentum",
            Self::SouthNorthMomentum => "south-north momentum",
            Self::PerturbationGeopotential => "perturbation geopotential",
            Self::PotentialTemperature => "potential temperature",
            Self::PerturbationColumnMass => "perturbation column mass",
            Self::VerticalMomentum => "vertical momentum",
        })
    }
}
