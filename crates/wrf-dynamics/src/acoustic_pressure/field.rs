use std::fmt;

/// Scientific role of a field in acoustic pressure diagnosis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticPressureField {
    /// Perturbation inverse density (`al`).
    InverseDensityPerturbation,
    /// Perturbation pressure (`p`).
    PressurePerturbation,
    /// Perturbation geopotential (`ph`).
    GeopotentialPerturbation,
    /// Previous perturbation pressure used by divergence damping (`pm1`).
    PreviousPressurePerturbation,
    /// Full inverse density (`alt`).
    FullInverseDensity,
    /// Current acoustic temperature variable (`t_2`).
    CurrentTemperature,
    /// Reference temperature perturbation (`t_1`).
    ReferenceTemperaturePerturbation,
    /// Linearized pressure coefficient (`c2a`).
    PressureCoefficient,
    /// Perturbation column mass (`mu`).
    PerturbationColumnMass,
    /// Full column mass (`mut`).
    FullColumnMass,
}

impl fmt::Display for AcousticPressureField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InverseDensityPerturbation => "inverse-density perturbation",
            Self::PressurePerturbation => "pressure perturbation",
            Self::GeopotentialPerturbation => "geopotential perturbation",
            Self::PreviousPressurePerturbation => "previous pressure perturbation",
            Self::FullInverseDensity => "full inverse density",
            Self::CurrentTemperature => "current temperature",
            Self::ReferenceTemperaturePerturbation => "reference temperature perturbation",
            Self::PressureCoefficient => "pressure coefficient",
            Self::PerturbationColumnMass => "perturbation column mass",
            Self::FullColumnMass => "full column mass",
        })
    }
}
