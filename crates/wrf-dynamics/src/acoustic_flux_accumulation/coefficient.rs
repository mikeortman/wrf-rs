use std::fmt;

/// Vertical coefficient used by acoustic mass-flux accumulation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticFluxCoefficient {
    /// Column-mass multiplier at half levels (`c1h`).
    ColumnMassMultiplier,
    /// Column-mass offset at half levels (`c2h`).
    ColumnMassOffset,
}

impl fmt::Display for AcousticFluxCoefficient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ColumnMassMultiplier => "column-mass multiplier",
            Self::ColumnMassOffset => "column-mass offset",
        })
    }
}
