use std::fmt;

/// Scientific role of a field used by the vertical acoustic solve.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerticalAcousticCoefficientField {
    /// Lower tridiagonal coefficient (`a`).
    LowerDiagonal,
    /// Reciprocal diagonal after forward elimination (`alpha`).
    InverseEliminatedDiagonal,
    /// Upper coefficient after forward elimination (`gamma`).
    UpperEliminationFactor,
    /// Full dry column mass (`mut`).
    FullColumnMass,
    /// Moisture correction at W points (`cqw`).
    VerticalMoistureCoefficient,
    /// Linearized pressure coefficient (`c2a`).
    PressureCoefficient,
}

impl fmt::Display for VerticalAcousticCoefficientField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::LowerDiagonal => "lower diagonal",
            Self::InverseEliminatedDiagonal => "inverse eliminated diagonal",
            Self::UpperEliminationFactor => "upper elimination factor",
            Self::FullColumnMass => "full column mass",
            Self::VerticalMoistureCoefficient => "vertical moisture coefficient",
            Self::PressureCoefficient => "pressure coefficient",
        })
    }
}
