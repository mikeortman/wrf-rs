use std::fmt;

/// Field role used in column-mass staggering shape errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColumnMassStaggeringField {
    /// Perturbation dry-air column mass input.
    PerturbationMass,
    /// Base-state dry-air column mass input.
    BaseMass,
    /// Already-combined full dry-air column mass input.
    FullMass,
    /// Output at west-east momentum points.
    WestEastMomentumMass,
    /// Output at south-north momentum points.
    SouthNorthMomentumMass,
}

impl fmt::Display for ColumnMassStaggeringField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PerturbationMass => formatter.write_str("perturbation mass"),
            Self::BaseMass => formatter.write_str("base mass"),
            Self::FullMass => formatter.write_str("full mass"),
            Self::WestEastMomentumMass => formatter.write_str("west-east momentum mass"),
            Self::SouthNorthMomentumMass => formatter.write_str("south-north momentum mass"),
        }
    }
}
