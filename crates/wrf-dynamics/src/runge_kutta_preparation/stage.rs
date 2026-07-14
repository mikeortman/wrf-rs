use std::fmt;

/// Stage identity used when integrated grid geometry is inconsistent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RungeKuttaPreparationStage {
    /// Full and horizontally staggered dry-air column mass.
    ColumnMass,
    /// Dry-air omega diagnosis.
    OmegaDiagnosis,
    /// Momentum-staggered moisture coefficients.
    MoistureCoefficients,
    /// Full inverse density.
    InverseDensity,
    /// Pressure-point geopotential.
    PressurePointGeopotential,
}

impl fmt::Display for RungeKuttaPreparationStage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ColumnMass => formatter.write_str("column mass"),
            Self::OmegaDiagnosis => formatter.write_str("omega diagnosis"),
            Self::MoistureCoefficients => formatter.write_str("moisture coefficients"),
            Self::InverseDensity => formatter.write_str("inverse density"),
            Self::PressurePointGeopotential => formatter.write_str("pressure-point geopotential"),
        }
    }
}
