use std::fmt;

/// Semantic role of a field supplied to acoustic-step preparation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticStepPreparationField {
    /// Previous west-east velocity.
    PreviousWestEastVelocity,
    /// Current west-east velocity.
    CurrentWestEastVelocity,
    /// Previous south-north velocity.
    PreviousSouthNorthVelocity,
    /// Current south-north velocity.
    CurrentSouthNorthVelocity,
    /// Previous vertical velocity.
    PreviousVerticalVelocity,
    /// Current vertical velocity.
    CurrentVerticalVelocity,
    /// Previous potential temperature.
    PreviousPotentialTemperature,
    /// Current potential temperature.
    CurrentPotentialTemperature,
    /// Previous geopotential.
    PreviousGeopotential,
    /// Current geopotential.
    CurrentGeopotential,
    /// Previous perturbation column mass.
    PreviousColumnMass,
    /// Current perturbation column mass.
    CurrentColumnMass,
    /// Base-state column mass.
    BaseColumnMass,
    /// Column mass on west-east momentum points.
    WestEastColumnMass,
    /// Column mass on south-north momentum points.
    SouthNorthColumnMass,
    /// Full mass-point column mass.
    FullColumnMass,
    /// Saved west-east staggered column mass.
    SavedWestEastColumnMass,
    /// Saved south-north staggered column mass.
    SavedSouthNorthColumnMass,
    /// Saved mass-point column mass.
    SavedMassPointColumnMass,
    /// External-mode divergence-damping state.
    DivergenceDamping,
    /// Saved west-east velocity.
    SavedWestEastVelocity,
    /// Saved south-north velocity.
    SavedSouthNorthVelocity,
    /// Saved vertical velocity.
    SavedVerticalVelocity,
    /// Saved potential temperature.
    SavedPotentialTemperature,
    /// Saved geopotential.
    SavedGeopotential,
    /// Saved perturbation column mass.
    SavedColumnMass,
    /// Saved omega.
    SavedOmega,
    /// Pressure/inverse-density coefficient (`c2a`).
    PressureCoefficient,
    /// Base-state pressure.
    BasePressure,
    /// Perturbation pressure.
    PerturbationPressure,
    /// Full inverse density.
    InverseDensity,
    /// Dry-air omega diagnostic.
    Omega,
    /// West-east momentum south-north map factor.
    WestEastMomentumMapFactor,
    /// Inverse south-north momentum west-east map factor.
    InverseSouthNorthMomentumMapFactor,
    /// Mass-point south-north map factor.
    MassPointMapFactor,
}

impl fmt::Display for AcousticStepPreparationField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::PreviousWestEastVelocity => "previous west-east velocity",
            Self::CurrentWestEastVelocity => "current west-east velocity",
            Self::PreviousSouthNorthVelocity => "previous south-north velocity",
            Self::CurrentSouthNorthVelocity => "current south-north velocity",
            Self::PreviousVerticalVelocity => "previous vertical velocity",
            Self::CurrentVerticalVelocity => "current vertical velocity",
            Self::PreviousPotentialTemperature => "previous potential temperature",
            Self::CurrentPotentialTemperature => "current potential temperature",
            Self::PreviousGeopotential => "previous geopotential",
            Self::CurrentGeopotential => "current geopotential",
            Self::PreviousColumnMass => "previous column mass",
            Self::CurrentColumnMass => "current column mass",
            Self::BaseColumnMass => "base column mass",
            Self::WestEastColumnMass => "west-east column mass",
            Self::SouthNorthColumnMass => "south-north column mass",
            Self::FullColumnMass => "full column mass",
            Self::SavedWestEastColumnMass => "saved west-east column mass",
            Self::SavedSouthNorthColumnMass => "saved south-north column mass",
            Self::SavedMassPointColumnMass => "saved mass-point column mass",
            Self::DivergenceDamping => "divergence damping",
            Self::SavedWestEastVelocity => "saved west-east velocity",
            Self::SavedSouthNorthVelocity => "saved south-north velocity",
            Self::SavedVerticalVelocity => "saved vertical velocity",
            Self::SavedPotentialTemperature => "saved potential temperature",
            Self::SavedGeopotential => "saved geopotential",
            Self::SavedColumnMass => "saved column mass",
            Self::SavedOmega => "saved omega",
            Self::PressureCoefficient => "pressure coefficient",
            Self::BasePressure => "base pressure",
            Self::PerturbationPressure => "perturbation pressure",
            Self::InverseDensity => "inverse density",
            Self::Omega => "omega",
            Self::WestEastMomentumMapFactor => "west-east momentum map factor",
            Self::InverseSouthNorthMomentumMapFactor => "inverse south-north momentum map factor",
            Self::MassPointMapFactor => "mass-point map factor",
        };
        formatter.write_str(name)
    }
}
