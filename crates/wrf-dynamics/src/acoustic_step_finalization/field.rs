use std::fmt;

/// Field role named by acoustic-finalization shape diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticStepFinalizationField {
    /// Current west-east velocity (`u_2`).
    WestEastVelocity,
    /// Current south-north velocity (`v_2`).
    SouthNorthVelocity,
    /// Current vertical velocity (`w_2`).
    VerticalVelocity,
    /// Current potential temperature (`t_2`).
    PotentialTemperature,
    /// Current perturbation geopotential (`ph_2`).
    PerturbationGeopotential,
    /// Current vertical mass flux (`ww`).
    VerticalMassFlux,
    /// Current perturbation column mass (`mu_2`).
    PerturbationColumnMass,
    /// Large-step full column mass (`mut`).
    LargeStepFullColumnMass,
    /// Final full column mass (`muts`).
    FinalFullColumnMass,
    /// Large-step west-east column mass (`muu`).
    LargeStepWestEastColumnMass,
    /// Final west-east column mass (`muus`).
    FinalWestEastColumnMass,
    /// Large-step south-north column mass (`muv`).
    LargeStepSouthNorthColumnMass,
    /// Final south-north column mass (`muvs`).
    FinalSouthNorthColumnMass,
    /// Saved west-east velocity (`u_save`).
    SavedWestEastVelocity,
    /// Saved south-north velocity (`v_save`).
    SavedSouthNorthVelocity,
    /// Saved vertical velocity (`w_save`).
    SavedVerticalVelocity,
    /// Saved potential temperature (`t_save`).
    SavedPotentialTemperature,
    /// Saved perturbation geopotential (`ph_save`).
    SavedPerturbationGeopotential,
    /// Saved perturbation column mass (`mu_save`).
    SavedPerturbationColumnMass,
    /// Saved vertical mass flux (`ww1`).
    SavedVerticalMassFlux,
    /// Diabatic heating (`h_diabatic`).
    DiabaticHeating,
    /// West-east velocity map factor (`msfuy`).
    WestEastVelocityMapFactor,
    /// South-north velocity map factor (`msfvx`).
    SouthNorthVelocityMapFactor,
    /// Mass-point south-north map factor (`msfty`).
    MassPointSouthNorthMapFactor,
}

impl fmt::Display for AcousticStepFinalizationField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WestEastVelocity => "west-east velocity",
            Self::SouthNorthVelocity => "south-north velocity",
            Self::VerticalVelocity => "vertical velocity",
            Self::PotentialTemperature => "potential temperature",
            Self::PerturbationGeopotential => "perturbation geopotential",
            Self::VerticalMassFlux => "vertical mass flux",
            Self::PerturbationColumnMass => "perturbation column mass",
            Self::LargeStepFullColumnMass => "large-step full column mass",
            Self::FinalFullColumnMass => "final full column mass",
            Self::LargeStepWestEastColumnMass => "large-step west-east column mass",
            Self::FinalWestEastColumnMass => "final west-east column mass",
            Self::LargeStepSouthNorthColumnMass => "large-step south-north column mass",
            Self::FinalSouthNorthColumnMass => "final south-north column mass",
            Self::SavedWestEastVelocity => "saved west-east velocity",
            Self::SavedSouthNorthVelocity => "saved south-north velocity",
            Self::SavedVerticalVelocity => "saved vertical velocity",
            Self::SavedPotentialTemperature => "saved potential temperature",
            Self::SavedPerturbationGeopotential => "saved perturbation geopotential",
            Self::SavedPerturbationColumnMass => "saved perturbation column mass",
            Self::SavedVerticalMassFlux => "saved vertical mass flux",
            Self::DiabaticHeating => "diabatic heating",
            Self::WestEastVelocityMapFactor => "west-east velocity map factor",
            Self::SouthNorthVelocityMapFactor => "south-north velocity map factor",
            Self::MassPointSouthNorthMapFactor => "mass-point south-north map factor",
        })
    }
}
